use std::time::Duration;
use std::{io, thread};

use std::thread::JoinHandle;

use log::{error, info};
use serialport::{ClearBuffer, FlowControl, SerialPort};
use winapi::ctypes::c_int;

use crate::config;
use crate::config::Config;
use crate::helper_funcs::{bit_read, SerialExt, MARK, SYNC};
use crate::keyboard::Keyboard;

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

    data_buffer: [u8; 512],
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
            data_buffer: [0; 512],
        })
    }

    fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<usize> {
        // self.port.write_jvs_packet(dest, data)?;
        //
        // // FIXME: for some reason it could just stop reading anything
        // self.port.read_jvs_packet(&mut self.data_buffer)
        Ok(0)
    }

    fn reset(&mut self) -> io::Result<()> {
        let data = [CMD_RESET, CMD_RESET_ARGUMENT];
        // self.port.write_jvs_packet(BROADCAST, &data)?;
        // self.port.write_jvs_packet(BROADCAST, &data)?;
        Ok(())
    }

    pub fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Initializing JVS");

        self.reset()?;
        info!("Reset sent");
        thread::sleep(Duration::from_millis(500));
        let size = self.cmd(BROADCAST, &[CMD_ASSIGN_ADDRESS, board])?;
        info!(
            "Assigned address {}. Data: {:?}",
            board,
            &self.data_buffer[..size],
        );

        let size = self.cmd(board, &[CMD_IDENTIFY])?;
        info!(
            "Board Info: {}",
            std::str::from_utf8(&self.data_buffer[..size]).unwrap()
        );

        let size = self.cmd(board, &[CMD_COMMAND_REVISION])?;
        info!(
            "Command Version Revision: {}{}",
            "REV",
            *&self.data_buffer[..size][0] as f32 / 10.0
        );

        let size = self.cmd(board, &[CMD_JVS_VERSION])?;
        info!("JVS Version: {:?}", &self.data_buffer[..size]);

        let size = self.cmd(board, &[CMD_COMMS_VERSION])?;
        info!("Communications Version: {:?}", &self.data_buffer[..size]);

        let size = self.cmd(board, &[CMD_CAPABILITIES])?;
        info!("Feature check: {:?}", &self.data_buffer[..size]);

        Ok(())
    }

    fn read_digital(&mut self, board: u8) -> io::Result<()> {
        let size = self.cmd(board, &[CMD_READ_DIGITAL, 0x02, 0x02])?;
        let data = &self.data_buffer[..size];

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
    done_recv: crossbeam_channel::Receiver<()>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    let mut jvs = RingEdge2::new(args.settings.jvs_re2_com.clone(), args.input.clone())?;
    jvs.init(1)?;

    Ok(thread::spawn(move || -> io::Result<()> {
        loop {
            if let Err(err) = done_recv.try_recv() {
                break;
            }
            if let Err(E) = jvs.read_digital(1) {
                error!("Jvs error: {}", E);
            };

            // thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }))
}
