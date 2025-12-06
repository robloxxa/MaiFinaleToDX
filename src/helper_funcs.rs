use log::error;
use std::io;
use std::io::{Read, Write};

pub static SYNC: u8 = 0xE0;
pub static MARK: u8 = 0xD0;

pub trait ReadExt: Read + Sized {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
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

pub fn bit_read(input: u8, n: usize) -> bool {
    input & (1 << n) != 0
}

pub fn log_error<E: std::fmt::Display>(err: E) -> E {
    error!("{}", err);
    err
}

// pub trait LogResultErr<R, E: std::fmt::Display>: Sized {
//     fn log_err<S: std::fmt::Display>(self, msg: S) -> Result<R, E>;
// }
// 
// impl<R, E: std::fmt::Display> LogResultErr<R, E> for Result<R, E> {
//     fn log_err<S: std::fmt::Display>(self, msg: S) -> Result<R, E> {
//         match self {
//             Ok(r) => Ok(r),
//             Err(e) => {
//                 error!("{}: {}", msg, e);
//                 Err(e)
//             }
//         }
//     }
// }