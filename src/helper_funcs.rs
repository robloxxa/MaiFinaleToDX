use log::debug;
use serialport;

use std::io;
use std::io::{BufWriter, Read, Write};


pub static SYNC: u8 = 0xE0;
pub static MARK: u8 = 0xD0;

pub trait ReadExt: Read {
    fn read_u8(&mut self) -> io::Result<u8> {
        let buf = &mut [0u8; 1];
        self.read(buf)?;
        Ok(buf[0])
    }

    fn read_u8_escaped(&mut self) -> io::Result<u8> {
        let mut b = self.read_u8()?;

        if b == MARK {
            b = self.read_u8()?.wrapping_add(1);
        }

        Ok(b)
    }
}

impl<R: Read> ReadExt for R {}

pub trait WriteExt: Write {
    fn write_u8(&mut self, b: u8) -> io::Result<()> {
        self.write(&[b])?;
        Ok(())
    }

    fn write_u8_escaped(&mut self, b: u8) -> io::Result<()> {
        if b == SYNC || b == MARK {
            let _ = self.write(&[MARK, b - 1])?;
            Ok(())
        } else {
            self.write_u8(b)
        }
    }
}

impl<W: Write> WriteExt for W {}

pub fn bit_read(input: &u8, n: usize) -> bool {
    input & (1 << n) != 0
}
