
use std::io::Read;
use std::time::Duration;
use log::debug;

use serialport::{ClearBuffer, SerialPort};


pub struct MessageCmd {
    pub player_num: usize,
    pub cmd: TouchMasterCommand,
}

#[repr(u8)]
#[derive(Debug)]
pub enum TouchMasterCommand {
    // { R S E T } Tells Touchscreen to reset, have no idea what to do with it
    Reset = b'E',
    // { H A L T } Tells Touchscreen to stop sending data
    Halt = b'L',
    // { S T A T } Tells Touchscreen to start sending data
    Stat = b'A',
    // { L/R TouchArea k Threshold }
    Sens(u8, u8, u8) = b'k',
    // There is also Ratio, but its useless on an actual cabinet (todo: verify this)
    Ratio(u8, u8, u8) = b'r',
    Unknown,
}

impl TouchMasterCommand {
    pub fn from_buf(buf: &[u8]) -> TouchMasterCommand {
        match buf[3] {
            b'E' => TouchMasterCommand::Reset,
            b'L' => TouchMasterCommand::Halt,
            b'A' => TouchMasterCommand::Stat,
            b'k' => TouchMasterCommand::Sens(buf[1], buf[2], buf[4]),
            b'r' => TouchMasterCommand::Ratio(buf[1], buf[2], buf[4]),
            _ => {
                TouchMasterCommand::Unknown
            },
        }
    }
}

pub struct Deluxe {
    pub port: serialport::COMPort,
    player_num: usize,
    read_buffer: [u8; 6],
    pub sender_channel: crossbeam_channel::Sender<MessageCmd>,
}

impl Deluxe {
    pub fn new(
        port_name: String,
        player_num: usize,
        sender_channel: crossbeam_channel::Sender<MessageCmd>,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, 115_200).open_native()?;
        port.set_timeout(Duration::from_millis(1))?;
        port.clear(ClearBuffer::All)?;
        Ok(Self {
            port,
            player_num,
            read_buffer: [0; 6],
            sender_channel,
        })
    }

    pub fn read(&mut self) {
        if let Err(err) = self.port.read_exact(self.read_buffer.as_mut()) {
            if err.kind() == std::io::ErrorKind::TimedOut {
                return;
            } else {
                panic!("{}", err);
            }
        }
        let cmd = TouchMasterCommand::from_buf(self.read_buffer.as_ref());

        self.sender_channel
            .send(MessageCmd {
                player_num: self.player_num,
                cmd,
            })
            .unwrap();
    }
}
