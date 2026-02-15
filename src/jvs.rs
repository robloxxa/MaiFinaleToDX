use std::io::{BufReader, BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use jvs_packets::jvs::{RequestPacket, ResponsePacket};
use jvs_packets::{Packet, ReadPacket, WritePacket};
use log::{error, info};

use crate::config;
use crate::config::Input;
use crate::error::Result;
use crate::helper_funcs::bit_read;
use crate::keyboard::Keyboard;
use crate::port::{Port, RealPort};

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
    // pub const CONVEY_ID: u8 = 0x15;
    pub const READ_DIGITAL: u8 = 0x20;
}

static BROADCAST: u8 = 0xFF;

pub struct Jvs {
    pub writer: BufWriter<Box<dyn Port>>,
    pub reader: BufReader<Box<dyn Port>>,
    keyboard: Keyboard,
    input: Input,
    req_packet: RequestPacket<16>,
    res_packet: ResponsePacket<128>,
}

impl Jvs {
    pub fn new(port_name: impl AsRef<str>, input: &Input) -> Result<Self> {
        let port = RealPort::open(port_name.as_ref(), 115_200)?;

        port.discard_buffers()?;

        let writer_port = port.try_clone()?;
        let reader_port = port.try_clone()?;

        Ok(Self {
            writer: BufWriter::with_capacity(512, writer_port),
            reader: BufReader::with_capacity(512, reader_port),
            keyboard: Keyboard::new(),
            input: input.clone(),
            req_packet: RequestPacket::default(),
            res_packet: ResponsePacket::default(),
        })
    }

    /// Writes a request packet to Jvs Com port and immediately wait for a response, muting self.res_packet
    fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        self.writer
            .write_packet(self.req_packet.set_dest(dest).set_data(data))?;

        self.writer.flush()?;

        self.reader.read_packet(&mut self.res_packet)?;
        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        self.req_packet
            .set_dest(BROADCAST)
            .set_data(&[Cmd::RESET, Cmd::RESET_ARGUMENT]);

        self.writer.write_packet(&self.req_packet)?;
        self.writer.write_packet(&self.req_packet)?;

        self.writer.flush()?;

        Ok(())
    }

    pub fn try_init(&mut self, retry_count: i64, board: u8) -> Result<()> {
        self.reader
            .get_mut()
            .set_read_timeout(Duration::from_secs(5))?;
        self.writer
            .get_mut()
            .set_write_timeout(Duration::from_secs(5))?;

        for c in 0..retry_count {
            info!("Trying to initialize Jvs. Attempt {}", c + 1);
            match self.init(board) {
                Ok(()) => {
                    self.reader
                        .get_mut()
                        .set_read_timeout(Duration::from_secs(2))?;
                    self.writer
                        .get_mut()
                        .set_write_timeout(Duration::from_secs(2))?;
                    return Ok(());
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    error!("Jvs initialization timed out")
                }
                Err(e) => {
                    error!("Jvs initialization failed: {}", e);
                }
            }
        }

        error!("Init failed");
        Err(io::Error::from(io::ErrorKind::TimedOut).into())
    }

    pub fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Jvs: Initializing");
        self.reset()?;

        info!("Jvs: Reset sent");
        // Wait a little before sending the next command
        thread::sleep(Duration::from_millis(500));

        self.cmd(BROADCAST, &[Cmd::ASSIGN_ADDRESS, board])?;
        info!("Jvs: Assigned address {}", board,);

        self.cmd(board, &[Cmd::IDENTIFY])?;
        info!(
            "Jvs: Board Info: {}",
            std::str::from_utf8(self.res_packet.data())
                .map_err(|_| io::Error::from(io::ErrorKind::Other))?
        );

        self.cmd(board, &[Cmd::COMMAND_REVISION])?;
        info!(
            "Jvs: Command Version Revision: REV{}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::JVS_VERSION])?;
        info!(
            "Jvs: Jvs Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::COMMS_VERSION])?;
        info!(
            "Jvs: Communications Version: {}.{}",
            self.res_packet.data()[0] / 10,
            self.res_packet.data()[0] % 10
        );

        self.cmd(board, &[Cmd::CAPABILITIES])?;
        info!("Jvs: Feature check: {:02X?}", self.res_packet.data());

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
            .key(self.input.p2_btn1, !bit_read(data[4], 2))?;
        self.keyboard
            .key(self.input.p2_btn2, !bit_read(data[4], 3))?;

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

pub fn setup(
    settings: &config::Jvs,
    should_exit: Arc<AtomicBool>,
    _shared_state: Option<crate::state::SharedState>,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut jvs = Jvs::new(&settings.port, &settings.input)?;

    jvs.try_init(settings.init_retry_count.unwrap_or(i64::MAX), 1)?;

    let handle = thread::Builder::new()
        .name("Finale Jvs Thread".to_string())
        .spawn(move || -> Result<()> {
            while !should_exit.load(Ordering::Acquire) {
                match jvs.read_digital(1) {
                    Ok(()) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        error!("Failed to read digital. Error: {}", e);
                    }
                }
            }
            Ok(())
        })?;

    Ok(vec![handle])
}
