use crate::config::{self};
use crate::error::Result;
use crate::keyboard::Keyboard;
use crate::port::{Port, RealPort};
use crate::runtime::Module;
use anyhow::anyhow;
use arrayvec::ArrayVec;
use jvs_packets::jvs_modified::{ModifiedPacket, RequestPacket, ResponsePacket};
use jvs_packets::{Packet, ReadPacket, WritePacket};
use tracing::{error, info};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};
use winapi::um::winuser::VK_RETURN;

pub(crate) mod state;

pub use state::State;

// #[derive(Debug)]
// #[repr(u8)]
// enum Command {
//     LEDReset = 0x10,
//     GetFirmware = 0x30,
//     GetHardware = 0x32,
//     RadioOn = 0x40,
//     RadioOff = 0x41,
//     Poll = 0x42,
//     Reset = 0x62,
// }
// 

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

    reader_file: File,
    destinations: ArrayVec<u8, 4>,
    retry_count: i64,
    exit_sig: Arc<AtomicBool>,

    req_packet: RequestPacket<128>,
    res_packet: ResponsePacket<128>,
}

impl CardReader {
    pub fn new(cfg: &config::Reader, exit_sig: Arc<AtomicBool>) -> Result<Self> {
        let mut finale_port = RealPort::open(&cfg.port, 38_400)?;

        finale_port.set_read_timeout(Duration::from_millis(5000))?;

        Ok(Self {
            port: Box::new(finale_port),
            keyboard: Keyboard::new(),
            reader_file: OpenOptions::new().read(true).write(true).open(
                cfg.device_file
                    .as_ref()
                    .ok_or_else(|| anyhow!("Device file not specified"))?,
            )?,
            destinations: cfg.destinations.clone(),
            retry_count: cfg.init_retry_count.unwrap_or(i64::MAX),
            exit_sig,
            req_packet: RequestPacket::default(),
            res_packet: ResponsePacket::default(),
        })
    }

    fn init_impl(&mut self) -> io::Result<()> {
        let retry_count = self.retry_count;
        let mut destinations = ArrayVec::<u8, 4>::new();

        for i in 0..self.destinations.len() {
            let destination = self.destinations[i];

            for r in 0..retry_count {
                if self.exit_sig.load(Ordering::Acquire) {
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
                        std::str::from_utf8(self.res_packet.data()).unwrap()
                    );

                    self.cmd(destination, Cmd::GET_HARDWARE, &[00])?;
                    info!(
                        "(DEST: {}) Hardware Version: {}",
                        destination,
                        std::str::from_utf8(self.res_packet.data()).unwrap()
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

    pub fn cmd(&mut self, dest: u8, cmd: u8, data: &[u8]) -> io::Result<()> {
        self.req_packet.set_dest(dest).set_cmd(cmd).set_data(data);

        self.port.write_packet(&self.req_packet)?;

        self.port.read_packet(&mut self.res_packet)?;

        Ok(())
    }

    fn poll_readers(&mut self) -> io::Result<()> {
        for i in 0..self.destinations.len() {
            match self.cmd(self.destinations[i], Cmd::POLL, &[00]) {
                Ok(()) => {
                    if self.res_packet.data().len() == 19 {
                        let mut id = String::new();
                        for &b in &self.res_packet.data()[3..=10] {
                            id.push_str(&format!("{:02X}", b));
                        }

                        self.reader_file.write_all(id.as_bytes())?;

                        self.keyboard.key_down(VK_RETURN)?;
                        thread::sleep(Duration::from_secs(2));
                        self.keyboard.key_up(VK_RETURN)?;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => {
                    error!("Card Reader Error: {}", e);
                }
            }
        }
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
    }
}

impl Module for CardReader {
    fn init(&mut self) -> Result<()> {
        self.init_impl()?;
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        let _ = self.poll_readers();
        thread::sleep(Duration::from_millis(250));
        Ok(())
    }
}

pub fn setup(
    cfg: config::reader::Reader,
    exit_sig: Arc<AtomicBool>,
    _shared_state: crate::state::SharedState,
) -> Result<Box<dyn Module>> {
    let reader = CardReader::new(&cfg, exit_sig)?;
    Ok(Box::new(reader))
}
