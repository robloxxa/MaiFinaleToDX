use serial2::SerialPort;

use log::{error, warn};
use std::io::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
// pub struct MessageCmd {
//     pub player_num: usize,
//     pub cmd: MasterCommand,
// }

// #[repr(u8)]
// #[derive(Debug)]
// pub enum MasterCommand {
//     // { R S E T } Tells Touchscreen to reset, have no idea what to do with it
//     Reset = b'E',
//     // { H A L T } Tells Touchscreen to stop sending data
//     Halt = b'L',
//     // { S T A T } Tells Touchscreen to start sending data
//     Stat = b'A',
//     // { L/R TouchArea k Threshold }
//     Sens(u8, u8, u8) = b'k',
//     // There is also Ratio, but its useless on an actual cabinet (todo: verify this)
//     Ratio(u8, u8, u8) = b'r',
//     Unknown,
// }

// impl MasterCommand {
//     pub fn from_buf(buf: &[u8]) -> MasterCommand {
//         match buf[3] {
//             b'E' => MasterCommand::Reset,
//             b'L' => MasterCommand::Halt,
//             b'A' => MasterCommand::Stat,
//             b'k' => MasterCommand::Sens(buf[1], buf[2], buf[4]),
//             b'r' => MasterCommand::Ratio(buf[1], buf[2], buf[4]),
//             _ => MasterCommand::Unknown,
//         }
//     }
// }

enum ResponseFrame {
    MasterRequest
}

pub struct FrameParser<const MAX_SIZE: usize = TOUCH_MAX_SIZE> {
    inner: [u8; MAX_SIZE],
    idx: usize,
    in_frame: bool,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            idx: 0,
            in_frame: false,
        }
    }

    pub fn push(&mut self, b: u8) -> ParsedPacket<'_> {
        match b {
            b'(' => {
                self.in_frame = true;
                self.idx = 0;
                ParsedPacket::Incompleted
            }

            b')' => {
                if !self.in_frame {
                    return ParsedPacket::Incompleted;
                }

                self.in_frame = false;

                // валидные длины: 4 или 12
                match self.idx {
                    4 => ParsedPacket::Data(&self.inner[..self.idx]),
                    12 => ParsedPacket::Input(&self.inner[..self.idx]),
                    _ => ParsedPacket::Incompleted,
                }
            }

            _ => {
                if !self.in_frame {
                    return ParsedPacket::Incompleted; // мусор вне фрейма
                }

                // пока собираем содержимое скобок
                if self.idx < 12 {
                    self.inner[self.idx] = b;
                    self.idx += 1;
                } else {
                    // переполнение → сброс, ищем новый '('
                    self.in_frame = false;
                    self.idx = 0;
                }

                ParsedPacket::Incompleted
            }
        }
    }
}

pub struct Deluxe {
    num: u8,
    pub port: SerialPort,
    pub active: Arc<AtomicBool>,
}

impl Deluxe {
    pub fn new(port_name: impl Into<String>, num: u8) -> Result<Self> {
        let port_name = port_name.into();
        let mut port = SerialPort::open(&port_name, 115_200).map_err(|e| {
            error!(
                "Cannot open serial port for Deluxe P{} Touchscreen: {}",
                num, e
            );
            e
        })?;

        port.set_read_timeout(Duration::from_millis(0))?;

        Ok(Self {
            num,
            port,
            active: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut read_buffer: [u8; 6] = [0; 6];
        match self.port.read_exact(&mut read_buffer) {
            Ok(_) => {
                match read_buffer[3] {
                    b'E' => self.active.store(false, Ordering::Relaxed),
                    b'L' => {
                        self.active.store(false, Ordering::Relaxed);
                        self.port.set_read_timeout(Duration::from_millis(0))?;
                    }
                    b'A' => {
                        self.active.store(true, Ordering::Relaxed);
                        self.port.set_read_timeout(Duration::from_millis(1000))?;
                    }
                    b'k' | b'r' => {
                        read_buffer[0] = b'(';
                        read_buffer[5] = b')';
                        self.send(&mut read_buffer)?;
                    }
                    _ => {
                        panic!("Unknown command: {:?}", &read_buffer);
                    }
                }
                Ok(())
            }
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub(crate) fn send(&self, buf: &[u8]) -> Result<()> {
        match self.port.write_all(buf) {
            Ok(()) => Ok(()),
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => {
                warn!("Write to Deluxe P{} timed out. This is probably due to MaiMai being closed", self.num);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            num: self.num,
            port: self.port.try_clone()?,
            active: self.active.clone(),
        })
    }

    pub fn spawn_thread(
        mut deluxe_touch: Deluxe,
        exit_sig: Arc<AtomicBool>,
    ) -> Result<JoinHandle<Result<()>>> {
        thread::Builder::new()
            .name(format!("Deluxe P{} Touch Thread", deluxe_touch.num))
            .spawn(move || {
                while !exit_sig.load(Ordering::Relaxed) {
                    deluxe_touch.process()?;
                }

                Ok(())
            })
    }
}
