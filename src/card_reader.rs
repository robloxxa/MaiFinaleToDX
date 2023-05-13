use log::{debug, error, info, warn};
use serialport;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, io, thread};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use serialport::{Error, SerialPort};
use winapi::um::winuser::VK_RETURN;

use crate::config::Config;
use crate::keyboard::Keyboard;

use crate::packets::rs232c;
use crate::packets::rs232c::Packet;

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

static RESET: u8 = 0x62;
static CMD_GETFIRMWARE: u8 = 0x30;
static CMD_GETHARDWARE: u8 = 0x32;
static CMD_RADIO_ON: u8 = 0x40;
static CMD_RADIO_OFF: u8 = 0x41;
static CMD_POLL: u8 = 0x42;

pub struct CardReader {
    re2_port: serialport::COMPort,
    alls_port: serialport::COMPort,
    req_packet: rs232c::RequestPacket<128>,
    res_packet: rs232c::ResponsePacket<128>,
}

impl CardReader {
    pub fn new(re2_port_name: String, alls_port_name: String) -> Result<Self, serialport::Error> {
        let mut re2_port = serialport::new(re2_port_name, 38_400).open_native()?;
        let mut alls_port = serialport::new(alls_port_name, 115_200).open_native()?;
        re2_port.set_timeout(Duration::from_millis(1000))?;
        alls_port.set_timeout(Duration::from_millis(0))?;

        Ok(Self {
            re2_port,
            alls_port,
            req_packet: rs232c::RequestPacket::default(),
            res_packet: rs232c::ResponsePacket::default(),
        })
    }

    pub fn init(&mut self, dest: u8) -> io::Result<()> {
        info!("Initializing Readers...");
        self.cmd(dest, RESET, &[00])?;
        self.cmd(dest, RESET, &[00])?;
        info!("Reset sent");
        self.cmd(dest, CMD_GETFIRMWARE, &[00])?;
        info!(
            "Firmware Version: {}",
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );
        self.cmd(dest, CMD_GETHARDWARE, &[00])?;
        info!(
            "Hardware Version: {}",
            std::str::from_utf8(self.res_packet.data()).unwrap()
        );
        info!("Reader successfully initialized");
        Ok(())
    }

    pub fn cmd(&mut self, dest: u8, cmd: u8, data: &[u8]) -> io::Result<()> {
        self.req_packet
            .set_dest(dest)
            .set_cmd(cmd)
            .set_data(data)
            .write(&mut self.re2_port)?;
        self.res_packet.read(&mut self.re2_port)?;

        Ok(())
    }
}

// fn read_aime_request(reader: &mut dyn SerialPort, buf: &mut [u8]) -> io::Result<usize> {
//     reader.read_u8()?;
//     Ok(0)
// }

// fn write_aime_request()

pub fn spawn_thread(
    config: &Config,
    running: Arc<AtomicBool>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    let mut reader = CardReader::new(
        config.settings.reader_re2_com.clone(),
        config.settings.reader_alls_com.clone(),
    )?;
    if let None = config.settings.reader_device_file {
        return Err(io::Error::new(io::ErrorKind::InvalidData,"The reader_device_file is empty, NFC reader is disabled."))
    }
    let path = config.settings.reader_device_file.clone().unwrap();
    debug!("{path}");
    // reader.init(0x00)?;
    Ok(thread::spawn(move || -> io::Result<()> {
        let mut kb = Keyboard::new();
        reader.init(00).expect("Init failed");
        reader.cmd(00, CMD_RADIO_ON, &[01, 03])?;
        while running.load(Ordering::SeqCst) {
            if let Err(e) = reader.cmd(00, CMD_POLL, &[00]) {
                debug!("timeout")
            }
            if reader.res_packet.data().len() == 20 {
                let mut f = OpenOptions::new().write(true).open(&path).expect("Cannot read file");
                let mut id = String::new();
                for &b in &reader.res_packet.data()[4..=11] {
                    id.push_str(&format!("{:02X}", b));
                }
                f.write(id.as_bytes()).unwrap();
                kb.key_down(&VK_RETURN);
                thread::sleep(Duration::from_secs(2));
                kb.key_up(&VK_RETURN);
            }
        }
        Ok(())
    }))
}
