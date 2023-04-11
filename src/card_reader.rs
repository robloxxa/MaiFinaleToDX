use std::{io, thread};
use std::io::Write;
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, info};
use serialport::SerialPort;

use crate::config::Config;
use crate::helper_funcs::{MARK, SerialExt, SYNC};

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
    seq_num: u8,
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
            data_buffer: [0; 512],
            seq_num: 0,
        })
    }

    pub fn init(&mut self) -> io::Result<()> {
        info!("Initializing Readers...");
        self.reset()?;
        info!("Reset sent");
        let mut n = self.cmd(00, &[CMD_GETFIRMWARE, 00])?;
        info!("Firmware Version: {}", std::str::from_utf8(&self.data_buffer[..n]).unwrap());
        n = self.cmd(00, &[CMD_GETHARDWARE, 00])?;
        info!("Hardware Version: {}", std::str::from_utf8(&self.data_buffer[..n]).unwrap());
        info!("Readers successfully initialized");
        Ok(())
    }

    pub fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<usize> {
        self.write_packet(dest, data)?;

        match self.re2_port.read_byte() {
            Ok(x) => if x != 0xE0 { return Ok(0) },
            Err(err) => return Err(io::Error::from(err)),
        }

        let size = self.re2_port.read_byte()? as usize;
        self.re2_port.read_byte()?;
        self.re2_port.read_byte()?;
        let cmd = self.re2_port.read_byte()?;
        let report = self.re2_port.read_byte()?;
        let mut counter = 0;
        while counter < size - 4 {
            let mut b = self.re2_port.read_byte()?;
            if b == MARK {
                b = self.re2_port.read_byte()? + 1;
            }
            self.data_buffer[counter] = b;
            counter += 1;
        }
        debug!("CMD: {}, Report: {}. Data: {:?}",
            cmd, report, &self.data_buffer[..counter]);
        Ok(counter-1)
    }

    pub fn write_packet(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        let size: u8 = data.len() as u8 + 3;
        let mut sum = dest as u32 + size as u32 + self.seq_num as u32;

        self.re2_port.write(&[SYNC, size, dest, self.seq_num])?;
        self.seq_num = self.seq_num + 1 % 32;

        for &b in data.iter() {
            if b == SYNC || b == MARK {
                self.re2_port.write(&[MARK, b - 1])?;
            } else {
                self.re2_port.write(&[b])?;
            }

            sum = (sum + b as u32) % 256;
        }
        self.re2_port.write(&[sum as u8])?;
        self.re2_port.flush()?;
        Ok(())
    }

    pub fn reset(&mut self) -> io::Result<()> {
        let _ = self.cmd(0, &[RESET, 00])?;
        thread::sleep(Duration::from_secs(2));
        let _ = self.cmd(0, &[RESET, 00])?;
        Ok(())
    }

    pub fn read_re2(&mut self) {}
}

pub fn spawn_thread(config: &Config) -> JoinHandle<()> {
    let mut reader = CardReader::new(
        config.settings.reader_re2_com.clone(),
        config.settings.reader_alls_com.clone(),
    ).unwrap();


    thread::spawn(move || {
        reader.init().expect("cannot init");
        let _ = reader.cmd(00, &[CMD_RADIO_ON, 00]);

        loop {
            let n = reader.cmd(00, &[CMD_POLL, 00]).unwrap();
            thread::sleep(Duration::from_millis(2000));
        }
    })
}