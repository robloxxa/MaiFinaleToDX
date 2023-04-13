use serialport;

pub static SYNC: u8 = 0xE0;
pub static MARK: u8 = 0xD0;

pub trait SerialExt: serialport::SerialPort {
    fn read_byte(&mut self) -> Result<u8, serialport::Error> {
        let mut read_buf: [u8; 1] = [0];
        self.read_exact(read_buf.as_mut())?;
        return Ok(read_buf[0]);
    }
}

impl SerialExt for dyn serialport::SerialPort {}

pub fn bit_read(input: &u8, n: usize) -> bool {
    input & (1 << n) != 0
}
