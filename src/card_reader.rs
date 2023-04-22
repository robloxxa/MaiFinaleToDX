use log::{debug, info, warn};
use serialport::{COMPort, SerialPort};
use std::io::{BufReader, Read, Write};
use std::os::windows::io::AsRawHandle;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};
use winapi::um::winbase::PACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA;

use crate::config::Config;
use crate::helper_funcs::{ReadExt, SerialExt, MARK, SYNC};

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

    data_buffer: [u8; 512],
    id_buffer: [u8; 64],
    seq_num: u8,
}

impl CardReader {
    pub fn new(re2_port_name: String, alls_port_name: String) -> Result<Self, serialport::Error> {
        let mut re2_port = serialport::new(re2_port_name, 115_200).open()?;
        let mut alls_port = serialport::new(alls_port_name, 115_200).open()?;
        re2_port.set_timeout(Duration::from_millis(0))?;
        alls_port.set_timeout(Duration::from_millis(0))?;

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
        // self.re2_port .write_aime_packet(dest, &mut self.seq_num, data)?;
        // read_aime_request(self.alls_port.as_mut(), &mut self.data_buffer);
        // self.re2_port.read_aime_packet(&mut self.data_buffer)

        Ok(0)
    }

    pub fn poll_nfc(&mut self, dest: u8) -> io::Result<usize> {
        let n = self.cmd(dest, &[CMD_POLL, 00])?;
        Ok(n)
    }
}

// fn read_aime_request(reader: &mut dyn SerialPort, buf: &mut [u8]) -> io::Result<usize> {
//     reader.read_u8()?;
//     Ok(0)
// }

// fn write_aime_request()

pub fn spawn_thread(
    config: &Config,
    done_recv: crossbeam_channel::Receiver<()>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    let mut reader = CardReader::new(
        config.settings.reader_re2_com.clone(),
        config.settings.reader_alls_com.clone(),
    )?;
    // reader.init(0x00)?;
    Ok(thread::spawn(move || -> io::Result<()> {
        // let _ = reader.cmd(0x00, &[CMD_RADIO_ON, 01, 03])?;
        // TODO: Write a proxy
        loop {
            if let Err(_) = done_recv.try_recv() {
                break;
            }

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
