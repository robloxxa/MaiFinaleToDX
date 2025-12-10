use std::io::{BufReader, BufWriter, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};

use std::thread::JoinHandle;

use jvs_packets::jvs::{RequestPacket, ResponsePacket};
use jvs_packets::{Packet, ReadPacket, WritePacket};
use log::{error, info};
use serial2::SerialPort;
use winapi::ctypes::c_int;

use crate::config;
use crate::config::Input;
use crate::helper_funcs::bit_read;
use crate::keyboard::Keyboard;

#[non_exhaustive]
pub struct Cmd;

impl Cmd {
    pub const RESET: u8 = 0xF0;
    pub const RESET_ARGUMENT: u8 = 0xD9;
    pub const ASSIGN_ADDRESS: u8 = 0xF1;
    pub const IDENTIFY: u8 = 0x10;
    pub const COMMAND_REVISION: u8 = 0x11;
    pub const JVS_VERSION: u8 = 0x12;
    pub const COMMS_VERSION: u8 = 0x13;
    pub const CAPABILITIES: u8 = 0x14;
    pub const CONVEY_ID: u8 = 0x15;
    pub const READ_DIGITAL: u8 = 0x20;
}

const UNUSED_MAPPING: c_int = -1;
static BROADCAST: u8 = 0xFF;

// type InputMapping = [[c_int; 8]; 4];

pub struct JVS {
    pub writer: BufWriter<SerialPort>,
    pub reader: BufReader<SerialPort>,
    keyboard: Keyboard,
    input: Input,
    req_packet: RequestPacket<16>,
    res_packet: ResponsePacket<128>,
}

impl JVS {
    pub fn new(port_name: impl AsRef<str>, input: &Input) -> Result<Self> {
        let mut port = SerialPort::open(port_name.as_ref(), 115_200)?;
        
        port.set_read_timeout(Duration::from_millis(500))?;
        
        Ok(Self {
            writer: BufWriter::with_capacity(512, port.try_clone()?),
            reader: BufReader::with_capacity(512, port.try_clone()?),
            keyboard: Keyboard::new(),
            input: input.clone(),
            req_packet: RequestPacket::default(),
            res_packet: ResponsePacket::default(),
        })
    }

    /// Writes a request packet to JVS Com port and immediately wait for a response, muting self.res_packet
    fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        self.writer
            .write_packet(self.req_packet.set_dest(dest).set_data(data))?;
        self.reader.read_packet(&mut self.res_packet)?;
        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        self.req_packet
            .set_dest(BROADCAST)
            .set_data(&[Cmd::RESET, Cmd::RESET_ARGUMENT]);

        self.writer.write_packet(&self.req_packet)?;
        self.writer.write_packet(&self.req_packet)?;

        Ok(())
    }

    pub fn init(&mut self, board: u8) -> Result<()> {
        const RETRY_COUNT: u8 = 5;

        self.reader
            .get_mut()
            .set_read_timeout(Duration::from_secs(2))?;
        self.reader
            .get_mut()
            .set_write_timeout(Duration::from_secs(2))?;

        for c in 0..RETRY_COUNT {
            info!("Trying to initialize JVS. Attempt {}", c + 1);
            match self.send_init(board) {
                Ok(()) => {
                    self.reader
                        .get_mut()
                        .set_read_timeout(Duration::from_secs(0))?;
                    self.reader
                        .get_mut()
                        .set_write_timeout(Duration::from_secs(0))?;
                    return Ok(());
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e.into()),
            }
        }

        error!("Init failed");
        Err(io::Error::from(io::ErrorKind::TimedOut).into())
    }

    pub fn send_init(&mut self, board: u8) -> io::Result<()> {
        info!("JVS: Initializing");

        self.reset()?;
        info!("JVS: Reset sent");
        thread::sleep(Duration::from_millis(500));

        self.cmd(BROADCAST, &[Cmd::ASSIGN_ADDRESS, board])?;
        info!("JVS: Assigned address {}", board,);

        self.cmd(board, &[Cmd::IDENTIFY])?;
        info!(
            "JVS: Board Info: {}",
            std::str::from_utf8(self.res_packet.data())
                .map_err(|e| anyhow::Error::from(e))
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?
        );

        self.cmd(board, &[Cmd::COMMAND_REVISION])?;
        info!(
            "JVS: Command Version Revision: REV{}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::JVS_VERSION])?;
        info!(
            "JVS: JVS Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::COMMS_VERSION])?;
        info!(
            "JVS: Communications Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::CAPABILITIES])?;
        info!("JVS: Feature check: {:02X?}", self.res_packet.data());

        Ok(())
    }

    fn read_digital(&mut self, board: u8) -> io::Result<()> {
        self.cmd(board, &[Cmd::READ_DIGITAL, 0x02, 0x02])?;

        let data = self.res_packet.data();

        // Input and Service Buttons
        self.keyboard.key(self.input.test, bit_read(data[2], 6))?;
        self.keyboard
            .key(self.input.service, bit_read(data[1], 7))?;

        // Player 1 buttons
        self.keyboard
            .key(self.input.p1_btn3, !bit_read(data[2], 0))?;
        self.keyboard
            .key(self.input.p1_btn1, !bit_read(data[2], 2))?;
        self.keyboard
            .key(self.input.p1_btn2, !bit_read(data[2], 3))?;

        self.keyboard
            .key(self.input.p1_btn8, !bit_read(data[3], 3))?;
        self.keyboard
            .key(self.input.p1_btn7, !bit_read(data[3], 4))?;
        self.keyboard
            .key(self.input.p1_btn6, !bit_read(data[3], 5))?;
        self.keyboard
            .key(self.input.p1_btn5, !bit_read(data[3], 6))?;
        self.keyboard
            .key(self.input.p1_btn4, !bit_read(data[3], 7))?;

        // Player 2 Buttons
        self.keyboard
            .key(self.input.p2_btn3, !bit_read(data[4], 0))?;
        self.keyboard
            .key(self.input.p2_btn1, !bit_read(data[4], 4))?;
        self.keyboard
            .key(self.input.p2_btn4, !bit_read(data[4], 3))?;

        self.keyboard
            .key(self.input.p2_btn8, !bit_read(data[5], 3))?;
        self.keyboard
            .key(self.input.p2_btn7, !bit_read(data[5], 4))?;
        self.keyboard
            .key(self.input.p2_btn6, !bit_read(data[5], 5))?;
        self.keyboard
            .key(self.input.p2_btn5, !bit_read(data[5], 6))?;
        self.keyboard
            .key(self.input.p2_btn4, !bit_read(data[5], 7))?;

        Ok(())
    }
}

pub fn init(
    settings: &config::JVS,
    handles: &mut Vec<JoinHandle<Result<()>>>,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let mut jvs = JVS::new(&settings.port, &settings.input)?;
    
    jvs.init(0)?;
    
    handles.push(
        thread::Builder::new()
            .name("Finale JVS Thread".to_string())
            .spawn(move || -> Result<()> {
                while !running.load(Ordering::Acquire) {
                    match jvs.read_digital(1) {
                        Ok(()) => {}
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                        Err(e) => {
                            error!("Failed to read digital. Error: {}", e);

                            return Err(e.into());
                        }
                    }
                }
                Ok(())
            })?,
    );

    Ok(())
}
