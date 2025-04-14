use crate::config::Settings;
use crate::keyboard::Keyboard;
use anyhow::{Context, Error, Result};
use jvs_packets::jvs_modified::{ModifiedPacket, RequestPacket, ResponsePacket};
use jvs_packets::{Packet, ReadPacket, WritePacket};
use log::{debug, info};
use serial2::SerialPort;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};
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
    pub const LED_RESET: u8 = 0x10;
    pub const GET_FIRMWARE: u8 = 0x30;
    pub const GET_HARDWARE: u8 = 0x32;
    pub const RADIO_ON: u8 = 0x40;
    pub const RADIO_OFF: u8 = 0x41;
    pub const POLL: u8 = 0x42;
    pub const RESET: u8 = 0x62;
}

pub struct CardReader {
    port: SerialPort,

    path: PathBuf,

    req_packet: RequestPacket<128>,
    res_packet: ResponsePacket<128>,
}

impl CardReader {
    pub fn new(finale_port_name: impl AsRef<str>, reader_file: impl Into<PathBuf>) -> Result<Self> {
        let mut finale_port = SerialPort::open(finale_port_name.as_ref(), 38_400)?;
        finale_port.set_read_timeout(Duration::from_millis(5000))?;

        Ok(Self {
            port: finale_port,
            path: reader_file.into(),
            req_packet: RequestPacket::default(),
            res_packet: ResponsePacket::default(),
        })
    }

    pub fn init(&mut self, dest: u8) -> io::Result<()> {
        info!("Initializing Readers...");
        self.cmd(dest, Cmd::RESET, &[00])?;
        self.cmd(dest, Cmd::RESET, &[00])?;
        info!("Reset sent");
        self.cmd(dest, Cmd::GET_FIRMWARE, &[00])?;
        info!(
            "Firmware Version: {}",
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );
        self.cmd(dest, Cmd::GET_HARDWARE, &[00])?;
        info!(
            "Hardware Version: {}",
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );
        info!("Reader successfully initialized");
        Ok(())
    }

    pub fn cmd(&mut self, dest: u8, cmd: u8, data: &[u8]) -> io::Result<()> {
        self.req_packet.set_dest(dest).set_cmd(cmd).set_data(data);

        self.port.write_packet(&self.req_packet)?;

        self.port.read_packet(&mut self.res_packet)?;

        Ok(())
    }
}

// fn read_aime_request(reader: &mut dyn SerialPort, buf: &mut [u8]) -> io::Result<usize> {
//     reader.read_u8()?;
//     Ok(0)
// }

// fn write_aime_request()

pub fn spawn_thread(
    mut reader: CardReader,
    exit_sig: Arc<AtomicBool>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    thread::Builder::new()
        .name("Card Reader Thread".to_string())
        .spawn(move || -> io::Result<()> {
            let mut kb = Keyboard::new();
            reader.cmd(00, Cmd::RADIO_ON, &[0x01, 0x03])?;
            while !exit_sig.load(Ordering::Relaxed) {
                match reader.cmd(00, Cmd::POLL, &[00]) {
                    Ok(()) => {
                        if reader.res_packet.data().len() == 20 {
                            let mut f = OpenOptions::new().write(true).open(&reader.path)?;
                            let mut id = String::new();
                            for &b in &reader.res_packet.data()[4..=11] {
                                id.push_str(&format!("{:02X}", b));
                            }
                            f.write_all(id.as_bytes())?;
                            kb.key_down(VK_RETURN)?;
                            thread::sleep(Duration::from_secs(2));
                            kb.key_up(VK_RETURN)?;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                    Err(e) => return Err(e),
                }
                thread::sleep(Duration::from_millis(250));
            }
            Ok(())
        })
}

pub fn setup(
    cfg: &Settings,
    handles: &mut Vec<JoinHandle<io::Result<()>>>,
    exit_sig: Arc<AtomicBool>,
) -> Result<()> {
    let file_path = cfg.reader_device_file.as_ref().map_or_else(
        || {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The reader_device_file is empty, NFC reader is disabled.",
            ))
        },
        |p| Ok(p.to_owned()),
    )?;

    let mut reader = CardReader::new(&cfg.reader_port, file_path)?;

    reader.init(00)?;

    handles.push(spawn_thread(reader, exit_sig.clone())?);

    Ok(())
}
