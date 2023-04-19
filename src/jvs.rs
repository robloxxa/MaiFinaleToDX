use std::time::{Duration, Instant};
use std::{io, thread};

use std::thread::JoinHandle;

use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio_serial;
use tokio_serial::SerialPortBuilderExt;
use winapi::ctypes::c_int;

use crate::config;
use crate::config::Config;
use crate::helper_funcs::{bit_read, MARK, SYNC};
use crate::keyboard::Keyboard;

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
type InputMapping = [[Option<c_int>; 8]; 4];

pub struct RingEdge2 {
    pub port: tokio_serial::SerialStream,
    pub keyboard: Keyboard,

    service_key: c_int,
    test_key: c_int,
    input_map: InputMapping,

    data_buffer: [u8; 512],
}

impl RingEdge2 {
    pub fn new(
        port_name: String,
        input_settings: config::Input,
    ) -> Result<Self, tokio_serial::Error> {
        let mut port = tokio_serial::new(port_name, 115_200).open_native_async()?;
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

    async fn read_jvs_response(&mut self, buf: &mut [u8]) -> tokio_serial::Result<usize> {
        let mut d = [0u8; 4];
        self.port.read_exact(&mut d).await?;
        let (dest, size, status) = (d[1], d[2] as usize, d[3]);

        let mut counter: usize = 0;

        while counter < size - 1 {
            let mut b = self.port.read_u8().await?;
            if b == MARK {
                b = self.port.read_u8().await? + 1;
            }
            buf[counter] = b;
            counter += 1;
        }

        // debug!(
        //     "JVS Read: {:X?} {:X?} {:X?} {:X?} {:X?}",
        //     SYNC,
        //     dest,
        //     size,
        //     status,
        //     &buf[..counter]
        // );

        Ok(counter)
    }

    async fn write_jvs_request(&mut self, dest: u8, data: &[u8]) -> tokio_serial::Result<()> {
        let mut writer = BufWriter::with_capacity(512, &mut self.port);
        let size: u8 = data.len() as u8 + 1;
        let mut sum: u8 = dest.wrapping_add(size);

        writer.write(&[SYNC, dest, size]).await?;

        for &b in data.iter() {
            if b == SYNC || b == MARK {
                writer.write(&[MARK, b - 1]).await?;
            } else {
                writer.write_u8(b).await?;
            }

            sum = sum.wrapping_add(b);
        }

        if sum == SYNC || sum == MARK {
            writer.write_all(&[MARK, sum - 1]).await?;
        } else {
            writer.write_u8(sum).await?;
        }
        self.port.flush().await?;
        debug!(
            "Write: {:X?} {:X?} {:X?} {:X?} {:X?}",
            SYNC, dest, size, &data, sum
        );
        Ok(())
    }

    async fn cmd(&mut self, dest: u8, data: &[u8]) -> tokio_serial::Result<usize> {
        {
            self.write_jvs_request(dest, data).await?;
        }
        let size = self.read_jvs_response(&mut self.data_buffer).await?;

        Ok(size)
    }

    async fn reset(&mut self) -> tokio_serial::Result<()> {
        let data = [CMD_RESET, CMD_RESET_ARGUMENT];
        self.write_jvs_request(BROADCAST, &data).await?;
        self.write_jvs_request(BROADCAST, &data).await?;
        Ok(())
    }

    pub async fn init(&mut self, board: u8) -> io::Result<()> {
        info!("Initializing JVS");

        self.reset().await?;
        info!("Reset sent");
        thread::sleep(Duration::from_millis(500));
        let _ = self.cmd(BROADCAST, &[CMD_ASSIGN_ADDRESS, board]).await?;
        info!("Assigned address {}", board,);

        let size = self.cmd(board, &[CMD_IDENTIFY]).await?;
        info!(
            "Board Info: {}",
            std::str::from_utf8(&self.data_buffer[1..size]).unwrap()
        );

        let size = self.cmd(board, &[CMD_COMMAND_REVISION]).await?;
        info!(
            "Command Version Revision: {}",
            *&self.data_buffer[..size][1] as f32 / 10.0
        );

        let size = self.cmd(board, &[CMD_JVS_VERSION]).await?;
        info!(
            "JVS Version: {:?}",
            *&self.data_buffer[..size][1] as f32 / 10.0
        );

        let size = self.cmd(board, &[CMD_COMMS_VERSION]).await?;
        info!(
            "Communications Version: {:?}",
            *&self.data_buffer[..size][1] as f32 / 10.0
        );

        let size = self.cmd(board, &[CMD_CAPABILITIES]).await?;
        info!("Feature check: {:?}", &self.data_buffer[..size]);

        Ok(())
    }

    async fn read_digital(&mut self, board: u8) -> io::Result<()> {
        let size = self.cmd(board, &[CMD_READ_DIGITAL, 0x02, 0x02]).await?;
        let data = &self.data_buffer[..size];

        if bit_read(&data[2], 6) {
            self.keyboard.key_down(&self.test_key);
        } else {
            self.keyboard.key_up(&self.test_key);
        }

        if bit_read(&data[1], 7) {
            self.keyboard.key_down(&self.service_key);
        } else {
            self.keyboard.key_up(&self.service_key);
        }

        for (i, bit) in data[2..=5].iter().enumerate() {
            for bit_pos in 0..=7 {
                if let Some(key) = self.input_map[i][bit_pos] {
                    if !bit_read(bit, bit_pos) {
                        self.keyboard.key_down(&key);
                    } else {
                        self.keyboard.key_up(&key);
                    }
                }
            }
        }
        Ok(())
    }
}

fn map_input_settings(settings: &config::Input) -> InputMapping {
    [
        [
            Some(settings.p1_btn3),
            None,
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
            None,
            Some(settings.p1_btn8),
            Some(settings.p1_btn7),
            Some(settings.p1_btn6),
            Some(settings.p1_btn5),
            Some(settings.p1_btn4),
        ],
        [
            Some(settings.p2_btn3),
            None,
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
            None,
            Some(settings.p2_btn8),
            Some(settings.p2_btn7),
            Some(settings.p2_btn6),
            Some(settings.p2_btn5),
            Some(settings.p2_btn4),
        ],
    ]
}

pub fn spawn_thread(args: &Config) -> tokio_serial::Result<JoinHandle<tokio_serial::Result<()>>> {
    // let mut jvs = RingEdge2::new(args.settings.jvs_re2_com.clone(), args.input.clone())?;
    // jvs.init(1)?;
    // Ok(thread::spawn(move || -> tokio_serial::Result<()> {
    //     loop {
    //         if let Err(e) = jvs.read_digital(1) {
    //             error!("Jvs error: {}", e);
    //         };
    //         // thread::sleep(Duration::from_millis(10));
    //     }
    // }))
    todo!()
}
