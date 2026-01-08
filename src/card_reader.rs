use crate::config::reader::Reader;
use crate::config::{self, reader};
use crate::error::Result;
use crate::keyboard::Keyboard;
use anyhow::{anyhow, Context};
use jvs_packets::jvs_modified::{ModifiedPacket, RequestPacket, ResponsePacket};
use jvs_packets::{Packet, ReadPacket, WritePacket};
use log::{debug, error, info};
use serial2::SerialPort;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Index;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{i64, io, thread};
use winapi::um::winuser::VK_RETURN;

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
    port: SerialPort,

    keyboard: Keyboard,

    reader_file: File,
    destinations: Vec<u8>,
    retry_count: i64,

    req_packet: RequestPacket<128>,
    res_packet: ResponsePacket<128>,
}

impl CardReader {
    pub fn new(cfg: &config::Reader) -> Result<Self> {
        let mut finale_port = SerialPort::open(&cfg.port, 38_400)?;

        finale_port.set_read_timeout(Duration::from_millis(5000))?;

        Ok(Self {
            port: finale_port,
            keyboard: Keyboard::new(),
            reader_file: OpenOptions::new().read(true).write(true).open(
                cfg.device_file
                    .as_ref()
                    .ok_or_else(|| anyhow!("Device file not specified"))?,
            )?,
            retry_count: cfg.init_retry_count.unwrap_or_else(|| i64::MAX),
            destinations: cfg.destinations.clone(),
            req_packet: RequestPacket::default(),
            res_packet: ResponsePacket::default(),
        })
    }

    pub fn init(&mut self, dest: u8) -> io::Result<()> {
        self.cmd(dest, Cmd::RESET, &[00])?;
        self.cmd(dest, Cmd::RESET, &[00])?;
        info!("(DEST: {}) Reset sent", dest);

        thread::sleep(Duration::from_secs(2));

        self.cmd(dest, Cmd::GET_FIRMWARE, &[00])?;
        info!(
            "(DEST: {}) Firmware Version: {}",
            dest,
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );

        self.cmd(dest, Cmd::GET_HARDWARE, &[00])?;
        info!(
            "(DEST: {}) Hardware Version: {}",
            dest,
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );

        self.cmd(dest, Cmd::RADIO_ON, &[0x01, 0x03])?;
        info!("(DEST: {}) Radio On", dest);

        info!("Reader at destination {} successfully initialized", dest);

        Ok(())
    }

    pub fn try_init(&mut self, retry_count: i64) -> io::Result<()> {
        let mut destinations: Vec<u8> = Vec::new();

        for i in 0..self.destinations.len() {
            for r in 0..retry_count {
                let destination = self.destinations[i];

                info!(
                    "Initializing Card Reader at destination {}. Attempt {}",
                    destination, r
                );
                match self.init(destination) {
                    Ok(()) => {
                        destinations.push(destination);
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                        error!(
                            "Timeout occurred during initialization at destionation {}: {}",
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
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to initialize Card Reader",
            ))
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

    pub fn poll(&mut self) -> io::Result<()> {
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
            self.cmd(self.destinations[i], Cmd::RADIO_OFF, &[0x01, 0x03])
                .unwrap();
        }
    }
}

pub fn setup(
    cfg: &config::reader::Reader,
    handles: &mut Vec<JoinHandle<Result<()>>>,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let mut reader = CardReader::new(cfg)?;

    reader.try_init(cfg.init_retry_count.unwrap_or_else(|| i64::MAX))?;

    handles.push(
        thread::Builder::new()
            .name("Card Reader Thread".to_string())
            .spawn(move || -> Result<()> {
                while !running.load(Ordering::Relaxed) {
                    let _ = reader.poll();
                    thread::sleep(Duration::from_millis(250));
                }

                Ok(())
            })
            .with_context(|| format!("Card Reader thread failed to spawn"))?,
    );

    Ok(())
}
