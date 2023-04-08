use std::io::{Read, Write};
use std::ops::DerefMut;
use std::time::Duration;
use std::{io, thread};

use log::info;
use serialport::SerialPort;
use winapi::ctypes::c_int;
use winapi::um::winuser::{
    VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD4, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
};

use crate::config;
use crate::helper_funcs::{bit_read, read_byte, SerialExt};
use crate::keyboard::Keyboard;

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

type InputMapping = [[Option<c_int>; 7]; 4];

pub struct RingEdge2 {
    pub port: Box<dyn SerialPort>,
    keyboard: Keyboard,

    service_key: c_int,
    test_key: c_int,
    input_map: InputMapping,

    data_buffer: [u8; 512],
}

impl RingEdge2 {
    pub fn new(
        port_name: String,
        input_settings: config::Input,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, 115_200).open()?;
        port.set_timeout(Duration::from_millis(0))?;
        let input_map = map_input_settings(&input_settings);
        Ok(Self {
            port,
            keyboard: Keyboard::new(),
            service_key: input_settings.service,
            test_key: input_settings.test,
            input_map,
            data_buffer: [0; 512],
        })
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

    fn cmd(&mut self, dest: u8, data: &[u8]) -> io::Result<(usize, u8)> {
        self.write_packet(dest, data)?;

        loop {
            if self.port.read_byte()? != SYNC {
                continue;
            }
            if self.port.read_byte()? != 00 {
                continue;
            }
            break;
        }

        let size = self.port.read_byte()? as usize;
        let status = self.port.read_byte()?;

        let mut counter: usize = 0;

        while counter < size - 1 {
            let mut b = self.port.read_byte()?;
            if b == MARK {
                b = self.port.read_byte()? + 1;
            }
            self.data_buffer[counter] = b;
            counter += 1;
        }

        Ok((counter, status))
    }

    fn reset(&mut self) -> io::Result<()> {
        let data = [CMD_RESET, CMD_RESET_ARGUMENT];
        self.write_packet(BROADCAST, &data)?;
        self.write_packet(BROADCAST, &data)?;
        Ok(())
    }

    pub fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Initializing JVS");

        self.reset()?;
        info!("Reset sent");
        thread::sleep(Duration::from_secs(1));

        let (size, status) = self.cmd(BROADCAST, &[CMD_ASSIGN_ADDRESS, board])?;
        info!(
            "[Status: {}] Assigned address {}. Data: {:?}",
            status,
            board,
            &self.data_buffer[..size],
        );

        let (size, status) = self.cmd(board, &[CMD_IDENTIFY])?;
        info!(
            "[Status: {}] Board Info: {}",
            status,
            std::str::from_utf8(&self.data_buffer[..size]).unwrap()
        );

        let (size, status) = self.cmd(board, &[CMD_COMMAND_REVISION])?;
        info!(
            "[Status: {}] Command Version Revision: {:?}",
            status,
            &self.data_buffer[..size]
        );

        let (size, status) = self.cmd(board, &[CMD_JVS_VERSION])?;
        info!(
            "[Status: {}] JVS Version: {:?}",
            status,
            &self.data_buffer[..size]
        );

        let (size, status) = self.cmd(board, &[CMD_COMMS_VERSION])?;
        info!(
            "[Status: {}] Communications Version: {:?}",
            status,
            &self.data_buffer[..size]
        );

        let (size, status) = self.cmd(board, &[CMD_CAPABILITIES])?;
        info!(
            "[Status: {}] Feature check: {:?}",
            status,
            &self.data_buffer[..size]
        );

        Ok(())
    }

    fn read_digital(&mut self, board: u8) {
        let (size, _) = self.cmd(board, &[CMD_READ_DIGITAL, 0x02, 0x02]).unwrap();
        let data = &self.data_buffer[..size];
        if bit_read(&data[1], 7) {
            self.keyboard.key_down(&self.service_key);
        } else {
            self.keyboard.key_up(&self.service_key);
        }

        if bit_read(&data[1], 6) {
            self.keyboard.key_down(&self.test_key);
        } else {
            self.keyboard.key_up(&self.test_key);
        }

        for (i, bit) in data[2..6].iter().enumerate() {
            for bit_pos in 0..5 {
                if let Some(key) = self.input_map[i][bit_pos] {
                    if bit_read(bit, bit_pos) {
                        self.keyboard.key_up(&key);
                    } else {
                        self.keyboard.key_down(&key);
                    }
                }
            }
        }
    }
}

fn map_input_settings(settings: &config::Input) -> InputMapping {
    [
        [
            Some(settings.p1_btn3),
            Some(settings.p1_btn1),
            Some(settings.p1_btn2),
            None,
            None,
            None,
            None,
        ],
        [
            None,
            None,
            Some(settings.p1_btn8),
            Some(settings.p1_btn7),
            Some(settings.p1_btn6),
            Some(settings.p1_btn5),
            Some(settings.p1_btn4),
        ],
        [
            Some(settings.p2_btn3),
            Some(settings.p2_btn1),
            Some(settings.p2_btn2),
            None,
            None,
            None,
            None,
        ],
        [
            None,
            None,
            Some(settings.p2_btn8),
            Some(settings.p2_btn7),
            Some(settings.p2_btn6),
            Some(settings.p2_btn5),
            Some(settings.p2_btn4),
        ],
    ]
}

pub fn spawn_thread() {}
