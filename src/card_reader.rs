use log::{debug, info, warn};
use serialport::{COMPort, SerialPort};
use std::io::{BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use crate::config::Config;
use crate::helper_funcs::{ReadExt, SerialExt, MARK, SYNC};
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
    re2_port: Box<dyn SerialPort>,
    alls_port: Box<dyn SerialPort>,
    req_packet: rs232c::RequestPacket<128>,
    res_packet: rs232c::ResponsePacket<128>,
    id_buffer: [u8; 64],
}

impl CardReader {
    pub fn new(re2_port_name: String, alls_port_name: String) -> Result<Self, serialport::Error> {
        let mut re2_port = serialport::new(re2_port_name, 38_400).open()?;
        let mut alls_port = serialport::new(alls_port_name, 115_200).open()?;
        re2_port.set_timeout(Duration::from_millis(0))?;
        alls_port.set_timeout(Duration::from_millis(0))?;

        Ok(Self {
            re2_port,
            alls_port,
            req_packet: rs232c::RequestPacket::default(),
            res_packet: rs232c::ResponsePacket::default(),
            id_buffer: [0; 64],
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
    // reader.init(0x00)?;
    Ok(thread::spawn(move || -> io::Result<()> {
        // let _ = reader.cmd(0x00, &[CMD_RADIO_ON, 01, 03])?;
        // TODO: Write a proxy
        while running.load(Ordering::SeqCst) {
            reader.req_packet.read(&mut reader.alls_port)?;

            reader.req_packet.write(&mut reader.re2_port)?;

            reader.res_packet.read(&mut reader.re2_port)?;

            reader.res_packet.write(&mut reader.alls_port)?;
            // let _ = reader.cmd(0x00, &[CMD_RADIO_ON, 01, 03])?;
            // if let Ok(n) = reader.poll_nfc(0x00) {
            //     if n > 2 {
            //         info!("P1: Got card! {:02X?}", &reader.data_buffer[..n]);
            //     }
            //     // warn!("00: {:?}", &reader.data_buffer[..n]);
            // }
            // let _ = reader.cmd(0x00, &[CMD_RADIO_OFF, 00])?;
            //
            // let _ = reader.cmd(0x01, &[CMD_RADIO_ON, 01, 03])?;
            // if let Ok(n) = reader.poll_nfc(0x01) {
            //     if n > 2 {
            //         info!("P2: Got card! {:02X?}", &reader.data_buffer[..n]);
            //     }
            //     // warn!("00: {:?}", &reader.data_buffer[..n]);
            // }
            //
            // let _ = reader.cmd(0x01, &[CMD_RADIO_OFF, 00])?;
            // thread::sleep(Duration::from_millis(200));
        }
        Ok(())
    }))
}
