use log::{debug, info};
use serialport::SerialPort;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};
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
// static CMD_RADIO_OFF: u8 = 0x41;
static CMD_POLL: u8 = 0x42;

pub struct CardReader {
    buf_writer: BufWriter<serialport::COMPort>,
    req_packet: rs232c::RequestPacket<128>,
    res_packet: rs232c::ResponsePacket<128>,
}

impl CardReader {
    pub fn new(re2_port_name: String) -> Result<Self, serialport::Error> {
        let mut re2_port = serialport::new(re2_port_name, 38_400).open_native()?;
        re2_port.set_timeout(Duration::from_millis(5000))?;

        Ok(Self {
            buf_writer: BufWriter::new(re2_port),
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
            .write(&mut self.buf_writer)?;
        self.res_packet.read(self.buf_writer.get_mut())?;
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
    let mut reader = CardReader::new(config.settings.reader_re2_com.clone())?;
    if config.settings.reader_device_file.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "The reader_device_file is empty, NFC reader is disabled.",
        ));
    }
    let path = config.settings.reader_device_file.clone().unwrap();
    // reader.init(0x00)?;
    let reader_handle = thread::Builder::new()
        .name("Card Reader Thread".to_string())
        .spawn(move || -> io::Result<()> {
            let mut kb = Keyboard::new();
            reader.init(00).expect("Init failed");
            reader.cmd(00, CMD_RADIO_ON, &[0x01, 0x03])?;
            while running.load(Ordering::Acquire) {
                if reader.cmd(00, CMD_POLL, &[00]).is_err() {
                    // TODO: handle error
                    debug!("timeout")
                }
                if reader.res_packet.data().len() == 20 {
                    let mut f = OpenOptions::new()
                        .write(true)
                        .open(&path)
                        .expect("Cannot read file");
                    let mut id = String::new();
                    for &b in &reader.res_packet.data()[4..=11] {
                        id.push_str(&format!("{:02X}", b));
                    }
                    f.write_all(id.as_bytes()).unwrap();
                    kb.key_down(&VK_RETURN);
                    thread::sleep(Duration::from_secs(2));
                    kb.key_up(&VK_RETURN);
                }
                thread::sleep(Duration::from_millis(250));
            }
            Ok(())
        })
        .unwrap();
    Ok(reader_handle)
}
