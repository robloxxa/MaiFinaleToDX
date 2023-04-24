use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};

use std::thread::JoinHandle;

use log::{debug, error, info};
use serialport::{ClearBuffer, FlowControl, SerialPort};
use winapi::ctypes::c_int;

use crate::config;
use crate::config::Config;
use crate::helper_funcs::{bit_read, SerialExt, MARK, SYNC};
use crate::keyboard::Keyboard;
use crate::packets::rs232;
use crate::packets::rs232::Packet;

static BROADCAST: u8 = 0xFF;

static CMD_RESET: u8 = 0xF0;
static CMD_RESET_ARGUMENT: u8 = 0xD9;
static CMD_ASSIGN_ADDRESS: u8 = 0xF1;

static CMD_IDENTIFY: u8 = 0x10;
static CMD_COMMAND_REVISION: u8 = 0x11;
static CMD_JVS_VERSION: u8 = 0x12;
static CMD_COMMS_VERSION: u8 = 0x13;
static CMD_CAPABILITIES: u8 = 0x14;
static CMD_CONVEY_ID: u8 = 0x15;
static CMD_READ_DIGITAL: u8 = 0x20;
type InputMapping = [[Option<c_int>; 8]; 4];

pub struct RingEdge2 {
    pub port: serialport::COMPort,
    keyboard: Keyboard,

    service_key: c_int,
    test_key: c_int,
    input_map: InputMapping,

    req_packet: rs232::RequestPacket<16>,
    res_packet: rs232::ResponsePacket<32>,
}

impl RingEdge2 {
    pub fn new(
        port_name: String,
        input_settings: config::Input,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, 115_200).open_native()?;
        port.set_timeout(Duration::from_millis(500))?;
        let input_map = map_input_settings(&input_settings);
        Ok(Self {
            port,
            keyboard: Keyboard::new(),
            service_key: input_settings.service,
            test_key: input_settings.test,
            input_map,
            req_packet: rs232::RequestPacket::default(),
            res_packet: rs232::ResponsePacket::default(),
        })
    }

    /// Writes a request packet to JVS Com port and immediately wait for a response, muting self.res_packet
    fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        self.req_packet
            .set_dest(dest)
            .set_data(data)
            .write(&mut self.port)?;
        self.res_packet.read(&mut self.port)?;
        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        self.req_packet
            .set_dest(0xFF)
            .set_data(&[CMD_RESET, CMD_RESET_ARGUMENT]);

        self.req_packet.write(&mut self.port)?;
        self.req_packet.write(&mut self.port)?;

        Ok(())
    }

    pub fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Initializing JVS");

        self.reset()?;
        info!("Reset sent");
        thread::sleep(Duration::from_millis(500));

        self.cmd(BROADCAST, &[CMD_ASSIGN_ADDRESS, board])?;
        info!(
            "Assigned address {}. Data: {:?}",
            board,
            self.res_packet.data(),
        );

        self.cmd(board, &[CMD_IDENTIFY])?;
        info!(
            "Board Info: {}",
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );

        self.cmd(board, &[CMD_COMMAND_REVISION])?;
        info!(
            "Command Version Revision: REV{}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[CMD_JVS_VERSION])?;
        info!(
            "JVS Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[CMD_COMMS_VERSION])?;
        info!(
            "Communications Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[CMD_CAPABILITIES])?;
        info!("Feature check: {:02X?}", self.res_packet.data());

        Ok(())
    }

    fn read_digital(&mut self, board: u8) -> io::Result<()> {
        self.cmd(board, &[CMD_READ_DIGITAL, 0x02, 0x02])?;
        let data = self.res_packet.data();

        debug!("{:02X?}", data);

        if bit_read(&data[2], 6) {
            self.keyboard.key_down(&self.test_key);
        } else {
            self.keyboard.key_up(&self.test_key);
        }

        if bit_read(&data[1], 7) {
            self.keyboard.key_down(&self.service_key);
        } else {
            self.keyboard.key_up(&self.service_key);
        }

        for (i, bit) in data[2..=5].iter().enumerate() {
            for bit_pos in 0..=7 {
                if let Some(key) = self.input_map[i][bit_pos] {
                    if !bit_read(bit, bit_pos) {
                        self.keyboard.key_down(&key);
                    } else {
                        self.keyboard.key_up(&key);
                    }
                }
            }
        }
        Ok(())
    }
}

fn map_input_settings(settings: &config::Input) -> InputMapping {
    [
        [
            Some(settings.p1_btn3),
            None,
            Some(settings.p1_btn1),
            Some(settings.p1_btn2),
            None,
            None,
            None,
            None,
        ],
        [
            None,
            None,
            None,
            Some(settings.p1_btn8),
            Some(settings.p1_btn7),
            Some(settings.p1_btn6),
            Some(settings.p1_btn5),
            Some(settings.p1_btn4),
        ],
        [
            Some(settings.p2_btn3),
            None,
            Some(settings.p2_btn1),
            Some(settings.p2_btn2),
            None,
            None,
            None,
            None,
        ],
        [
            None,
            None,
            None,
            Some(settings.p2_btn8),
            Some(settings.p2_btn7),
            Some(settings.p2_btn6),
            Some(settings.p2_btn5),
            Some(settings.p2_btn4),
        ],
    ]
}

pub fn spawn_thread(
    args: &Config,
    running: Arc<AtomicBool>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    let mut jvs = RingEdge2::new(args.settings.jvs_re2_com.clone(), args.input.clone())?;
    jvs.init(1)?;

    Ok(thread::spawn(move || -> io::Result<()> {
        while running.load(Ordering::SeqCst) {
            if let Err(E) = jvs.read_digital(1) {
                error!("Jvs error: {}", E);
            };

            // thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }))
}
