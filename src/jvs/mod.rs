use std::io::{self, Read, Write};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::{error, info};

use crate::config;
use crate::config::jvs::JvsMode;
use crate::config::Input;
use crate::error::{Error, Result};
use crate::exit_signal::ExitSignal;
use crate::helper_funcs::bit_read;
use crate::keyboard::Keyboard;
use crate::port::{MockPort, Port, RealPort};
use crate::runtime::Module;

pub mod emulator;
pub mod packet;
pub mod state;

pub use state::State;

use emulator::JvsEmulator;
use packet::{Builder, JvsPacket, Parser};

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
    pub const READ_DIGITAL: u8 = 0x20;
}

static BROADCAST: u8 = 0xFF;

pub struct Jvs {
    pub port: Box<dyn Port>,
    keyboard: Keyboard,
    input: Input,
    retry_count: i64,
    exit_sig: ExitSignal,
    board: u8,
    parser: Parser,
    builder: Builder,
    buf: [u8; 64],
    waiting_response: bool,
    jvs_state: Arc<State>,
    pub emu_handle: Option<JoinHandle<Result<()>>>,
}

impl Jvs {
    pub fn new(
        port: Box<dyn Port>,
        jvs_state: Arc<State>,
        input: &Input,
        retry_count: i64,
        board: u8,
        exit_sig: ExitSignal,
    ) -> Result<Self> {
        Ok(Self {
            port,
            keyboard: Keyboard::new(),
            input: input.clone(),
            retry_count,
            exit_sig,
            board,
            parser: Parser::new(),
            builder: Builder::new(),
            buf: [0; 64],
            waiting_response: false,
            jvs_state,
            emu_handle: None,
        })
    }

    fn write_packet(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        self.builder.build(dest, data);
        self.port.write_all(self.builder.as_slice())
    }

    /// Sends a packet and blocks until a complete response arrives or the port times out.
    /// Stores the response payload (status byte stripped) into `self.res_data`.
    fn cmd(&mut self, dest: u8, data: &[u8]) -> Result<Option<&[u8]>> {
        self.write_packet(dest, data)?;

        loop {
            if self.exit_sig.is_set() {
                return Err(Error::ModuleStopped);
            }
            
            let n = self.port.read(&mut self.buf)?;

            for i in 0..n {
                if let Some(pkt) = self.parser.push(self.buf[i]) {
                    return match pkt {
                        JvsPacket::Valid { data, .. } => {
                            // data[0] is the outer JVS status byte; strip it to match
                            // the payload layout expected by callers.
                            let payload = data.get(1..).unwrap_or(&[]);
                            let len = payload.len().min(self.res_data.len());
                            self.res_data[..len].copy_from_slice(&payload[..len]);
                            self.res_len = len;
                            Ok(())
                        }
                        JvsPacket::Invalid => {
                            Err(io::Error::new(io::ErrorKind::InvalidData, "Checksum mismatch"))
                        }
                    };
                }
            }
        }
    }

    fn reset(&mut self) -> io::Result<()> {
        self.builder.build(BROADCAST, &[Cmd::RESET, Cmd::RESET_ARGUMENT]);
        self.port.write_all(self.builder.as_slice())?;
        self.port.write_all(self.builder.as_slice())
    }

    fn init_impl(&mut self) -> Result<()> {
        let retry_count = self.retry_count;
        let board = self.board;

        self.port.set_read_timeout(Duration::from_secs(5))?;
        self.port.set_write_timeout(Duration::from_secs(5))?;

        for c in 0..retry_count {
            if self.exit_sig.is_set() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled").into());
            }
            info!("Trying to initialize Jvs. Attempt {}", c + 1);
            match (|| -> io::Result<()> {
                self.reset()?;
                thread::sleep(Duration::from_millis(500));

                self.cmd(BROADCAST, &[Cmd::ASSIGN_ADDRESS, board])?;
                info!("Jvs: Assigned address {}", board);

                self.cmd(board, &[Cmd::IDENTIFY])?;
                info!(
                    "Jvs: Board Info: {}",
                    std::str::from_utf8(&self.res_data[..self.res_len])
                        .map_err(|_| io::Error::from(io::ErrorKind::Other))?
                );

                self.cmd(board, &[Cmd::COMMAND_REVISION])?;
                info!(
                    "Jvs: Command Version Revision: REV{}.{}",
                    self.res_data[0] / 10,
                    self.res_data[0] % 10
                );

                self.cmd(board, &[Cmd::JVS_VERSION])?;
                info!(
                    "Jvs: Jvs Version: {}.{}",
                    self.res_data[0] / 10,
                    self.res_data[0] % 10
                );

                self.cmd(board, &[Cmd::COMMS_VERSION])?;
                info!(
                    "Jvs: Communications Version: {}.{}",
                    self.res_data[0] / 10,
                    self.res_data[0] % 10
                );

                self.cmd(board, &[Cmd::CAPABILITIES])?;
                info!("Jvs: Feature check: {:02X?}", &self.res_data[..self.res_len]);

                Ok(())
            })() {
                Ok(()) => {
                    self.port.set_read_timeout(Duration::from_millis(1))?;
                    self.port.set_write_timeout(Duration::from_secs(2))?;
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

    fn poll_impl(&mut self) -> Result<()> {
        if !self.waiting_response {
            let board = self.board;
            self.write_packet(board, &[Cmd::READ_DIGITAL, 0x02, 0x02])?;
            self.waiting_response = true;
            return Ok(());
        }

        match self.port.read(&mut self.buf) {
            Ok(n) => {
                for i in 0..n {
                    if let Some(pkt) = self.parser.push(self.buf[i]) {
                        self.waiting_response = false;
                        if let JvsPacket::Valid { data, .. } = pkt {
                            let payload = data.get(1..).unwrap_or(&[]);
                            let len = payload.len().min(self.res_data.len());
                            self.res_data[..len].copy_from_slice(&payload[..len]);
                            self.res_len = len;
                            self.process_digital()?;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    fn process_digital(&mut self) -> io::Result<()> {
        if self.res_len < 5 {
            return Ok(());
        }

        // Copy to avoid borrow conflicts when calling keyboard methods.
        let d = [
            self.res_data[0],
            self.res_data[1],
            self.res_data[2],
            self.res_data[3],
            self.res_data[4],
        ];

        let gui = self.jvs_state.load_buttons();

        self.keyboard.key(self.input.test,    bit_read(d[1], 6) || gui.test);
        self.keyboard.key(self.input.service, bit_read(d[0], 7) || gui.service);

        self.keyboard.key(self.input.p1_btn1, !bit_read(d[1], 2) || gui.p1[0]);
        self.keyboard.key(self.input.p1_btn2, !bit_read(d[1], 3) || gui.p1[1]);
        self.keyboard.key(self.input.p1_btn3, !bit_read(d[1], 0) || gui.p1[2]);
        self.keyboard.key(self.input.p1_btn4, !bit_read(d[2], 7) || gui.p1[3]);
        self.keyboard.key(self.input.p1_btn5, !bit_read(d[2], 6) || gui.p1[4]);
        self.keyboard.key(self.input.p1_btn6, !bit_read(d[2], 5) || gui.p1[5]);
        self.keyboard.key(self.input.p1_btn7, !bit_read(d[2], 4) || gui.p1[6]);
        self.keyboard.key(self.input.p1_btn8, !bit_read(d[2], 3) || gui.p1[7]);

        self.keyboard.key(self.input.p2_btn1, !bit_read(d[3], 2) || gui.p2[0]);
        self.keyboard.key(self.input.p2_btn2, !bit_read(d[3], 3) || gui.p2[1]);
        self.keyboard.key(self.input.p2_btn3, !bit_read(d[3], 0) || gui.p2[2]);
        self.keyboard.key(self.input.p2_btn4, !bit_read(d[4], 7) || gui.p2[3]);
        self.keyboard.key(self.input.p2_btn5, !bit_read(d[4], 6) || gui.p2[4]);
        self.keyboard.key(self.input.p2_btn6, !bit_read(d[4], 5) || gui.p2[5]);
        self.keyboard.key(self.input.p2_btn7, !bit_read(d[4], 4) || gui.p2[6]);
        self.keyboard.key(self.input.p2_btn8, !bit_read(d[4], 3) || gui.p2[7]);

        if let Err(e) = self.keyboard.flush() {
            error!("JVS keyboard flush error: {}", e);
        }
        Ok(())
    }
}

impl Drop for Jvs {
    fn drop(&mut self) {
        if let Some(handle) = self.emu_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Module for Jvs {
    fn init(&mut self) -> Result<()> {
        self.init_impl()
    }

    fn poll(&mut self) -> Result<()> {
        if let Err(e) = self.poll_impl() {
            error!("Jvs poll error: {}", e);
        }
        Ok(())
    }
}

pub fn setup(
    cfg: config::Jvs,
    exit_sig: ExitSignal,
    shared_state: crate::state::SharedState,
) -> Result<Box<dyn Module>> {
    let (port, emu_handle) = match cfg.mode {
        JvsMode::Emulated => {
            let mock = MockPort::new();
            let mock_clone = mock.try_clone()?;
            let emu = JvsEmulator::new(mock, shared_state.jvs.clone(), exit_sig.clone(), 1);
            let handle = emu.spawn_thread()?;
            (mock_clone, Some(handle))
        }
        JvsMode::Hardware => {
            let port = RealPort::open(&cfg.port, 115_200)?;
            port.discard_buffers()?;
            (Box::new(port) as Box<dyn Port>, None)
        }
    };

    let mut jvs = Jvs::new(
        port,
        shared_state.jvs.clone(),
        &cfg.input,
        cfg.init_retry_count.unwrap_or(i64::MAX),
        1,
        exit_sig,
    )?;
    jvs.emu_handle = emu_handle;
    Ok(Box::new(jvs))
}
