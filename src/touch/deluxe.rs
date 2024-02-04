use anyhow::{Context, Error, Result};
use serial2::SerialPort;

use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

pub struct Deluxe {
    pub port: SerialPort,
    read_buffer: [u8; 6],
    active: Arc<AtomicBool>,
}

impl Deluxe {
    pub fn new(port_name: impl Into<String>, active: &Arc<AtomicBool>) -> Result<Self> {
        let port_name = port_name.into();
        let mut port = SerialPort::open(&port_name, 115_200)
            .with_context(|| format!("Failed to open port {}", port_name))?;
        port.set_read_timeout(Duration::from_millis(1000))?;
        Ok(Self {
            port,
            read_buffer: [0; 6],
            active: active.clone(),
        })
    }

    pub fn process(&mut self) -> Result<()> {
        match self.port.read_exact(&mut self.read_buffer) {
            Ok(_) => {
                match self.read_buffer[3] {
                    b'E' => self.active.store(false, Ordering::Relaxed),
                    b'L' => self.active.store(false, Ordering::Relaxed),
                    b'A' => self.active.store(true, Ordering::Relaxed),
                    b'k' | b'r' => {
                        self.read_buffer[0] = b'(';
                        self.read_buffer[5] = b')';
                        self.port.write_all(&self.read_buffer)?;
                    }
                    _ => {
                        panic!("Unknown command: {:?}", &self.read_buffer);
                    }
                }
                Ok(())
            }
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}
