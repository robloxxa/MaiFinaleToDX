use crate::config::{self};
use crate::config::reader::ReaderMode;
use crate::error::Result;
use crate::exit_signal::ExitSignal;
use crate::keyboard::Keyboard;
use crate::port::{MockPort, Port, RealPort};
use crate::runtime::Module;
use anyhow::anyhow;
use arrayvec::ArrayVec;
use jvs_packets::jvs_modified::{ModifiedPacket, RequestPacket};
use jvs_packets::{Packet, WritePacket};
use tracing::{error, info};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{io, thread};
use winapi::um::winuser::VK_RETURN;

pub(crate) mod emulator;
pub(crate) mod packet;
pub(crate) mod state;

pub use state::State;

use emulator::CardReaderEmulator;
use packet::{ParseResult, ResponseParser};

#[non_exhaustive]
pub struct Cmd;

impl Cmd {
    // pub const LED_RESET: u8 = 0x10;
    pub const GET_FIRMWARE: u8 = 0x30;
    pub const GET_HARDWARE: u8 = 0x32;
    pub const RADIO_ON: u8 = 0x40;
    pub const RADIO_OFF: u8 = 0x41;
    pub const POLL: u8 = 0x42;
    pub const RESET: u8 = 0x62;
}

pub struct CardReader {
    port: Box<dyn Port>,

    keyboard: Keyboard,

    reader_file: Option<File>,
    reader_state: Arc<Mutex<state::State>>,
    destinations: ArrayVec<u8, 4>,
    retry_count: i64,
    exit_sig: ExitSignal,

    req_packet: RequestPacket<128>,
    parser: ResponseParser,
    /// Last successfully parsed response payload (for init logging and card handling).
    res_data: [u8; 128],
    res_len: usize,
    buf: [u8; 64],

    waiting_response: bool,
    current_dest_idx: usize,
    /// When set, ENTER is held until this instant.
    key_release_at: Option<Instant>,

    emu_handle: Option<JoinHandle<Result<()>>>,
}

impl CardReader {
    pub fn new(
        port: Box<dyn Port>,
        cfg: &config::Reader,
        exit_sig: ExitSignal,
        reader_state: Arc<Mutex<state::State>>,
    ) -> Result<Self> {
        let reader_file = match cfg.device_file.as_ref() {
            Some(path) => Some(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)
                    .map_err(|e| anyhow!("Failed to open device file '{}': {}", path, e))?,
            ),
            None => None,
        };
        Ok(Self {
            port,
            keyboard: Keyboard::new(),
            reader_file,
            reader_state,
            destinations: cfg.destinations.clone(),
            retry_count: cfg.init_retry_count.unwrap_or(i64::MAX),
            exit_sig,
            req_packet: RequestPacket::default(),
            parser: ResponseParser::new(),
            res_data: [0; 128],
            res_len: 0,
            buf: [0; 64],
            waiting_response: false,
            current_dest_idx: 0,
            key_release_at: None,
            emu_handle: None,
        })
    }

    fn init_impl(&mut self) -> io::Result<()> {
        let retry_count = self.retry_count;
        let mut destinations = ArrayVec::<u8, 4>::new();

        for i in 0..self.destinations.len() {
            let destination = self.destinations[i];

            for r in 0..retry_count {
                if self.exit_sig.is_set() {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
                }
                info!(
                    "Initializing Card Reader at destination {}. Attempt {}",
                    destination,
                    r + 1
                );
                match (|| -> io::Result<()> {
                    self.cmd(destination, Cmd::RESET, &[00])?;
                    self.cmd(destination, Cmd::RESET, &[00])?;
                    info!("(DEST: {}) Reset sent", destination);

                    thread::sleep(Duration::from_secs(2));

                    self.cmd(destination, Cmd::GET_FIRMWARE, &[00])?;
                    info!(
                        "(DEST: {}) Firmware Version: {}",
                        destination,
                        std::str::from_utf8(&self.res_data[..self.res_len]).unwrap_or("?")
                    );

                    self.cmd(destination, Cmd::GET_HARDWARE, &[00])?;
                    info!(
                        "(DEST: {}) Hardware Version: {}",
                        destination,
                        std::str::from_utf8(&self.res_data[..self.res_len]).unwrap_or("?")
                    );

                    self.cmd(destination, Cmd::RADIO_ON, &[0x01, 0x03])?;
                    info!("(DEST: {}) Radio On", destination);

                    info!("Reader at destination {} successfully initialized", destination);
                    Ok(())
                })() {
                    Ok(()) => {
                        destinations.push(destination);
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                        error!(
                            "Timeout occurred during initialization at destination {}: {}",
                            destination, e
                        );
                    }
                    Err(e) => {
                        error!(
                            "Initialization failed at destination {}: {}",
                            destination, e
                        );
                    }
                }
            }
        }

        if destinations.is_empty() {
            Err(io::Error::other("Failed to initialize Card Reader"))
        } else {
            self.destinations = destinations;
            Ok(())
        }
    }

    /// Blocking cmd used during init (port has a long read timeout at that point).
    /// Stores the response data payload into `self.res_data`/`self.res_len`.
    pub fn cmd(&mut self, dest: u8, cmd: u8, data: &[u8]) -> io::Result<()> {
        self.req_packet.set_dest(dest).set_cmd(cmd).set_data(data);
        self.port.write_packet(&self.req_packet)?;

        loop {
            let n = self.port.read(&mut self.buf)?;

            for i in 0..n {
                if let Some(result) = self.parser.push(self.buf[i]) {
                    return match result {
                        ParseResult::Valid { data, len } => {
                            let copy_len = len.min(self.res_data.len());
                            self.res_data[..copy_len].copy_from_slice(&data[..copy_len]);
                            self.res_len = copy_len;
                            Ok(())
                        }
                        ParseResult::Invalid => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Checksum mismatch",
                        )),
                    };
                }
            }
        }
    }

    fn poll_impl(&mut self) -> io::Result<()> {
        if self.destinations.is_empty() {
            return Ok(());
        }

        // Release the held ENTER key when the timer expires.
        if let Some(release_at) = self.key_release_at {
            if Instant::now() >= release_at {
                self.key_release_at = None;
                self.keyboard.key_up(VK_RETURN);
                self.keyboard.flush()?;
            }
            // While holding the key we skip polling.
            return Ok(());
        }

        let dest = self.destinations[self.current_dest_idx];

        if !self.waiting_response {
            self.req_packet.set_dest(dest).set_cmd(Cmd::POLL).set_data(&[0x00]);
            self.port.write_packet(&self.req_packet)?;
            self.waiting_response = true;
            return Ok(());
        }

        match self.port.read(&mut self.buf) {
            Ok(n) => {
                for i in 0..n {
                    if let Some(result) = self.parser.push(self.buf[i]) {
                        self.waiting_response = false;
                        self.current_dest_idx =
                            (self.current_dest_idx + 1) % self.destinations.len();

                        if let ParseResult::Valid { data, len } = result {
                            if len == 19 {
                                let copy_len = len.min(self.res_data.len());
                                self.res_data[..copy_len].copy_from_slice(&data[..copy_len]);
                                self.res_len = copy_len;
                                self.handle_card()?;
                            }
                        }
                        break;
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e),
        }

        Ok(())
    }

    fn handle_card(&mut self) -> io::Result<()> {
        let mut id = String::with_capacity(16);
        for &b in &self.res_data[3..=10] {
            id.push_str(&format!("{:02X}", b));
        }

        if let Some(file) = &mut self.reader_file {
            file.write_all(id.as_bytes())?;
        }

        self.reader_state.lock().unwrap().last_card_id = Some(id);

        self.keyboard.key_down(VK_RETURN);
        self.keyboard.flush()?;
        self.key_release_at = Some(Instant::now() + Duration::from_secs(2));

        Ok(())
    }
}

impl Drop for CardReader {
    fn drop(&mut self) {
        for i in 0..self.destinations.len() {
            let dest = self.destinations[i];
            if let Err(e) = self.cmd(dest, Cmd::RADIO_OFF, &[0x01, 0x03]) {
                error!("Failed to turn off radio for destination {}: {}", dest, e);
            }
        }
        if let Some(handle) = self.emu_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Module for CardReader {
    fn init(&mut self) -> Result<()> {
        self.init_impl()?;
        self.port.set_read_timeout(Duration::from_millis(1))?;
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        if let Err(e) = self.poll_impl() {
            error!("Card reader poll error: {}", e);
        }
        Ok(())
    }
}

pub fn setup(
    cfg: config::reader::Reader,
    exit_sig: ExitSignal,
    shared_state: crate::state::SharedState,
) -> Result<Box<dyn Module>> {
    let (port, emu_handle) = match cfg.mode {
        ReaderMode::Emulated => {
            let mut mock = MockPort::new();
            // init_impl uses 5-second blocking reads; the default timeout of zero
            // would cause immediate TimedOut errors before the emulator responds.
            mock.set_read_timeout(Duration::from_millis(5000))?;
            let mock_clone = mock.try_clone()?;
            let emu = CardReaderEmulator::new(mock, shared_state.reader.clone(), exit_sig.clone());
            let handle = emu.spawn_thread()?;
            (mock_clone, Some(handle))
        }
        ReaderMode::Hardware => {
            let mut port = RealPort::open(&cfg.port, 38_400)?;
            port.set_read_timeout(Duration::from_millis(5000))?;
            (Box::new(port) as Box<dyn Port>, None)
        }
    };

    let mut reader = CardReader::new(port, &cfg, exit_sig, shared_state.reader.clone())?;
    reader.emu_handle = emu_handle;
    Ok(Box::new(reader))
}
