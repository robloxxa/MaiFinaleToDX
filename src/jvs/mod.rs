use arrayvec::ArrayVec;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::{debug, error, info, warn};

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

fn apply_digital_to_state(state: &State, d: [u8; 5]) {
    state.set_hardware_button(0, bit_read(d[1], 6)); // test     (active-high)
    state.set_hardware_button(1, bit_read(d[0], 7)); // service  (active-high)
    state.set_hardware_button(2, !bit_read(d[1], 2)); // p1_btn1  (active-low)
    state.set_hardware_button(3, !bit_read(d[1], 3)); // p1_btn2
    state.set_hardware_button(4, !bit_read(d[1], 0)); // p1_btn3
    state.set_hardware_button(5, !bit_read(d[2], 7)); // p1_btn4
    state.set_hardware_button(6, !bit_read(d[2], 6)); // p1_btn5
    state.set_hardware_button(7, !bit_read(d[2], 5)); // p1_btn6
    state.set_hardware_button(8, !bit_read(d[2], 4)); // p1_btn7
    state.set_hardware_button(9, !bit_read(d[2], 3)); // p1_btn8
    state.set_hardware_button(10, !bit_read(d[3], 2)); // p2_btn1
    state.set_hardware_button(11, !bit_read(d[3], 3)); // p2_btn2
    state.set_hardware_button(12, !bit_read(d[3], 0)); // p2_btn3
    state.set_hardware_button(13, !bit_read(d[4], 7)); // p2_btn4
    state.set_hardware_button(14, !bit_read(d[4], 6)); // p2_btn5
    state.set_hardware_button(15, !bit_read(d[4], 5)); // p2_btn6
    state.set_hardware_button(16, !bit_read(d[4], 4)); // p2_btn7
    state.set_hardware_button(17, !bit_read(d[4], 3)); // p2_btn8
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
    /// Returns the response payload (status byte stripped) as a stack-allocated ArrayVec.
    fn cmd(&mut self, dest: u8, data: &[u8]) -> Result<ArrayVec<u8, 253>> {
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
                            let payload = data.get(1..).unwrap_or(&[]);
                            Ok(ArrayVec::try_from(payload).unwrap_or_default())
                        }
                        JvsPacket::Invalid(data) => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Checksum mismatch: {:#04X?}", data),
                        )
                        .into()),
                    };
                }
            }
        }
    }

    fn reset(&mut self) -> io::Result<()> {
        self.builder
            .build(BROADCAST, &[Cmd::RESET, Cmd::RESET_ARGUMENT]);
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
            match (|| -> Result<()> {
                self.reset()?;
                thread::sleep(Duration::from_millis(500));

                self.cmd(BROADCAST, &[Cmd::ASSIGN_ADDRESS, board])?;
                info!("Jvs: Assigned address {}", board);

                let data = self.cmd(board, &[Cmd::IDENTIFY])?;
                info!(
                    "Jvs: Board Info: {}",
                    std::str::from_utf8(&data)
                        .map_err(|_| io::Error::from(io::ErrorKind::Other))?
                );

                let data = self.cmd(board, &[Cmd::COMMAND_REVISION])?;
                info!(
                    "Jvs: Command Version Revision: REV{}.{}",
                    data[0] / 10,
                    data[0] % 10
                );

                let data = self.cmd(board, &[Cmd::JVS_VERSION])?;
                info!("Jvs: Jvs Version: {}.{}", data[0] / 10, data[0] % 10);

                let data = self.cmd(board, &[Cmd::COMMS_VERSION])?;
                info!(
                    "Jvs: Communications Version: {}.{}",
                    data[0] / 10,
                    data[0] % 10
                );

                let data = self.cmd(board, &[Cmd::CAPABILITIES])?;
                info!("Jvs: Feature check: {:02X?}", &data[..]);

                Ok(())
            })() {
                Ok(()) => {
                    self.port.set_read_timeout(Duration::from_millis(1))?;
                    self.port.set_write_timeout(Duration::from_secs(2))?;
                    return Ok(());
                }
                Err(Error::Io(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
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
                        match pkt {
                            JvsPacket::Valid { data, .. } => {
                                let d: Option<[u8; 5]> = if let Some(p) = data.get(1..) {
                                    (p.len() >= 6).then(|| [p[1], p[2], p[3], p[4], p[5]])
                                } else {
                                    None
                                };
                                if let Some(d) = d {
                                    self.process_digital(d)?;
                                }
                            }
                            JvsPacket::Invalid(data) => {
                                warn!("Invalid JVS packet: {:#04X?}", data);
                            }
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    fn process_digital(&mut self, d: [u8; 5]) -> io::Result<()> {
        apply_digital_to_state(&self.jvs_state, d);

        let combined = self.jvs_state.load_buttons();
        self.keyboard.key(self.input.test, combined.test);
        self.keyboard.key(self.input.service, combined.service);
        self.keyboard.key(self.input.p1_btn1, combined.p1[0]);
        self.keyboard.key(self.input.p1_btn2, combined.p1[1]);
        self.keyboard.key(self.input.p1_btn3, combined.p1[2]);
        self.keyboard.key(self.input.p1_btn4, combined.p1[3]);
        self.keyboard.key(self.input.p1_btn5, combined.p1[4]);
        self.keyboard.key(self.input.p1_btn6, combined.p1[5]);
        self.keyboard.key(self.input.p1_btn7, combined.p1[6]);
        self.keyboard.key(self.input.p1_btn8, combined.p1[7]);
        self.keyboard.key(self.input.p2_btn1, combined.p2[0]);
        self.keyboard.key(self.input.p2_btn2, combined.p2[1]);
        self.keyboard.key(self.input.p2_btn3, combined.p2[2]);
        self.keyboard.key(self.input.p2_btn4, combined.p2[3]);
        self.keyboard.key(self.input.p2_btn5, combined.p2[4]);
        self.keyboard.key(self.input.p2_btn6, combined.p2[5]);
        self.keyboard.key(self.input.p2_btn7, combined.p2[6]);
        self.keyboard.key(self.input.p2_btn8, combined.p2[7]);

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
        self.poll_impl()
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
