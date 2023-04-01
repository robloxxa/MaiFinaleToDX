use crate::helper_funcs;
use log::info;
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;
use std::{io, thread};

static SYNC: u8 = 0xE0;
static MARK: u8 = 0xD0;
static BROADCAST: u8 = 0xFF;

static CMD_RESET: u8 = 0xF0;
static CMD_RESET_ARGUMENT: u8 = 0xD9;
static CMD_ASSIGN_ADDRESS: u8 = 0xF1;

static CMD_IDENTIFY: u8 = 0x10;
static CMD_COMMAND_REVISION: u8 = 0x11;
static CMD_JVS_VERSION: u8 = 0x12;
static CMD_COMMS_VERSION: u8 = 0x13;
static CMD_CAPABILITIES: u8 = 0x14;
static CMD_CONVEY_ID: u8 = 0x15;
static CMD_READ_DIGITAL: u8 = 0x20;

pub struct RingEdge2 {
    pub port: Box<dyn SerialPort>,
    keyboard: helper_funcs::Keyboard,
    read_buffer: [u8; 1],
    data_buffer: [u8; 1024],
}

impl RingEdge2 {
    pub fn new(port_name: String, baud_rate: u32) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, baud_rate).open()?;
        port.set_timeout(Duration::from_millis(0))?;

        Ok(Self {
            port,
            keyboard: helper_funcs::Keyboard::new(),
            read_buffer: [0; 1],
            data_buffer: [0; 1024],
        })
    }

    pub fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Initializing JVS");
        self.reset()?;
        info!("Reset sent");
        thread::sleep(Duration::from_secs(1));
        let (data, status) = self.cmd(BROADCAST, &[CMD_ASSIGN_ADDRESS, board])?;
        info!(
            "[Status: {}] Assigned address {}. Data: {:?}",
            status, board, data
        );
        let (ident, status) = self.cmd(board, &[CMD_IDENTIFY])?;

        info!(
            "[Status: {}] Identity: {}",
            status,
            std::str::from_utf8(ident).unwrap()
        );

        let (data, status) = self.cmd(board, &[CMD_COMMAND_REVISION])?;
        info!("[Status: {}] Command Version Revision: {:?}", status, data);

        let (data, status) = self.cmd(board, &[CMD_JVS_VERSION])?;
        info!("[Status: {}] JVS Version: {:?}", status, data,);

        let (data, status) = self.cmd(board, &[CMD_COMMS_VERSION])?;
        info!("[Status: {}] Communications Version: {:?}", status, data,);

        let (data, status) = self.cmd(board, &[CMD_CAPABILITIES])?;
        info!("[Status: {}] Feature check: {:?}", status, data,);

        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        let data = [CMD_RESET, CMD_RESET_ARGUMENT];
        self.write_packet(BROADCAST, &data)?;
        self.write_packet(BROADCAST, &data)?;
        Ok(())
    }

    pub fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<(&mut [u8], u8)> {
        self.write_packet(dest, data)?;

        loop {
            if self.read_byte()? != SYNC {
                continue;
            }
            if self.read_byte()? != 00 {
                continue;
            }
            break;
        }

        let size = self.read_byte()? as usize;
        let status = self.read_byte()?;

        let mut counter: usize = 0;

        while counter < size - 1 {
            let mut b = self.read_byte()?;
            if b == MARK {
                b = self.read_byte()? + 1;
            }
            self.data_buffer[counter] = b;
            counter += 1;
        }

        Ok((self.data_buffer[0..counter].as_mut(), status))
    }

    pub fn write_packet(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        let size: u8 = data.len() as u8 + 1;
        let mut sum = (dest + size) as u32;

        self.port.write(&[SYNC, dest, size])?;

        for &b in data.iter() {
            if b == SYNC || b == MARK {
                self.port.write(&[MARK, b - 1])?;
            } else {
                self.port.write(&[b])?;
            }

            sum = (sum + b as u32) % 256;
        }
        self.port.write(&[sum as u8])?;
        self.port.flush()?;
        Ok(())
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        self.port.read_exact(&mut self.read_buffer)?;
        return Ok(self.read_buffer[0]);
    }
}

pub fn spawn_thread() {}
