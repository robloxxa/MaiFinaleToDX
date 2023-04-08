use std::time::Duration;

use crate::serial;
use serialport::SerialPort;

use crate::touch::AllsMessageCmd;

#[derive(Debug)]
#[repr(u8)]
pub enum AllsTouchMasterCommand {
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

impl AllsTouchMasterCommand {
    pub fn from_buf(buf: &[u8]) -> AllsTouchMasterCommand {
        match buf[3] {
            b'E' => AllsTouchMasterCommand::Reset,
            b'L' => AllsTouchMasterCommand::Halt,
            b'A' => AllsTouchMasterCommand::Stat,
            b'k' => AllsTouchMasterCommand::Sens(buf[1], buf[2], buf[4]),
            b'r' => AllsTouchMasterCommand::Ratio(buf[1], buf[2], buf[4]),
            _ => AllsTouchMasterCommand::Unknown,
        }
    }
}

pub struct Alls {
    pub port: Box<dyn SerialPort>,
    player_num: usize,
    read_buffer: [u8; 6],
    pub sender_channel: crossbeam_channel::Sender<AllsMessageCmd>,
}

impl Alls {
    pub fn new(
        port_name: String,
        baud_rate: u32,
        player_num: usize,
        sender_channel: crossbeam_channel::Sender<AllsMessageCmd>,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, baud_rate).open()?;
        port.set_timeout(Duration::from_millis(1))?;
        
        Ok(Self {
            port,
            player_num,
            read_buffer: [0; 6],
            sender_channel,
        })
    }

    pub fn read(&mut self) {
        if let Err(err) = self.port.read(self.read_buffer.as_mut()) {
            if err.kind() == std::io::ErrorKind::TimedOut {
                return;
            } else {
                panic!("{}", err);
            }
        }
        let cmd = AllsTouchMasterCommand::from_buf(self.read_buffer.as_ref());

        self.sender_channel
            .send(AllsMessageCmd {
                player_num: self.player_num,
                cmd,
            })
            .unwrap();
    }
}
