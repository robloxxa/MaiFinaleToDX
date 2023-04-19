use std::thread::JoinHandle;
use std::{io, thread};

use log::info;
use tokio_serial::SerialPortBuilderExt;

use crate::config::Config;

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
    re2_port: tokio_serial::SerialStream,
    alls_port: tokio_serial::SerialStream,

    data_buffer: [u8; 512],
    id_buffer: [u8; 64],
    seq_num: u8,
}

impl CardReader {
    pub fn new(re2_port_name: String, alls_port_name: String) -> Result<Self, tokio_serial::Error> {
        let mut re2_port = tokio_serial::new(re2_port_name, 38_400).open_native_async()?;
        let mut alls_port = tokio_serial::new(alls_port_name, 115_200).open_native_async()?;

        Ok(Self {
            re2_port,
            alls_port,
            data_buffer: [0; 512],
            id_buffer: [0; 64],
            seq_num: 0,
        })
    }

    pub fn init(&mut self, dest: u8) -> io::Result<()> {
        info!("Initializing Readers...");
        let _ = self.cmd(dest, &[RESET, 00])?;
        let _ = self.cmd(dest, &[RESET, 00])?;
        info!("Reset sent");
        let mut n = self.cmd(dest, &[CMD_GETFIRMWARE, 00])?;
        info!(
            "Firmware Version: {}",
            std::str::from_utf8(&self.data_buffer[..n]).unwrap()
        );
        n = self.cmd(dest, &[CMD_GETHARDWARE, 00])?;
        info!(
            "Hardware Version: {}",
            std::str::from_utf8(&self.data_buffer[..n]).unwrap()
        );
        // let _ = self.cmd(0x09, &[0xF5, 00]);
        // let _ = self.cmd(0x09, &[0xF5, 00]);
        info!("Reader successfully initialized");
        Ok(())
    }

    pub fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<usize> {
        // self.re2_port
        //     .write_aime_request(dest, &mut self.seq_num, data)?;
        // self.re2_port.read_aime_response(&mut self.data_buffer)
        todo!()
    }

    pub fn poll_nfc(&mut self, dest: u8) -> io::Result<usize> {
        let n = self.cmd(dest, &[CMD_POLL, 00])?;
        Ok(n)
    }
}

pub fn spawn_thread(config: &Config) -> tokio_serial::Result<JoinHandle<tokio_serial::Result<()>>> {
    let mut reader = CardReader::new(
        config.settings.reader_re2_com.clone(),
        config.settings.reader_alls_com.clone(),
    )?;

    // reader.init(0x00)?;
    Ok(thread::spawn(move || -> tokio_serial::Result<()> {
        // let _ = reader.cmd(0x00, &[CMD_RADIO_ON, 01, 03])?;
        // TODO: Write a proxy
        loop {
            // let req = RequestPacket::read(reader.alls_port.as_mut())?;
            // reader.seq_num = req.seq_num;
            // req.write(reader.re2_port.as_mut())?;

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
    }))
}
