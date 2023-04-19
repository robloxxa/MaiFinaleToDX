use tokio::io::{AsyncRead, AsyncReadExt};
use crate::touch::AllsMessageCmd;
use tokio::sync::mpsc::Sender;
use tokio_serial;
use tokio_serial::SerialPortBuilderExt;
#[repr(u8)]
#[derive(Debug)]
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
    pub port: tokio_serial::SerialStream,
    player_num: usize,
    read_buffer: [u8; 6],
    pub sender_channel: Sender<AllsMessageCmd>,
}

impl Alls {
    pub fn new(
        port_name: String,
        player_num: usize,
        sender_channel: Sender<AllsMessageCmd>,
    ) -> Result<Self, tokio_serial::Error> {
        let port = tokio_serial::new(port_name, 115_200).open_native_async()?;

        Ok(Self {
            port,
            player_num,
            read_buffer: [0; 6],
            sender_channel,
        })
    }

    pub async fn read(&mut self) -> tokio_serial::Result<()> {
        self.port.read_exact(&mut self.read_buffer).await?;
        // if let Err(err) = self.port.read(self.read_buffer.as_mut()) {
        //     if err.kind() == std::io::ErrorKind::TimedOut {
        //         return;
        //     } else {
        //         panic!("{}", err);
        //     }
        // }
        let cmd = AllsTouchMasterCommand::from_buf(self.read_buffer.as_ref());

        self.sender_channel.send(AllsMessageCmd {
            player_num: self.player_num,
            cmd,
        });
        
        Ok(())
    }
}
