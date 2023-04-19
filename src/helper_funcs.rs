use log::debug;
use std::io;
use std::io::Write;
use tokio_serial;

pub static SYNC: u8 = 0xE0;
pub static MARK: u8 = 0xD0;

pub fn is_escape_byte(b: u8) -> bool {
    b == SYNC || b == MARK
}

pub struct Response<'a> {
    size: usize,
    dest: u8,
    seq_num: u8,
    cmd: u8,
    status: &'a [u8],
}

// pub trait SerialExt: tokio_serial::SerialPort {
//     fn read_aime_response(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         match self.read_byte() {
//             Ok(x) => {
//                 if x != 0xE0 {
//                     return Ok(0);
//                 }
//             }
//             Err(err) => return Err(io::Error::from(err)),
//         }
//
//         let size = self.read_byte()? as usize;
//         self.read_byte()?;
//         self.read_byte()?;
//         let cmd = self.read_byte()?;
//         let report = self.read_byte()?;
//         let mut counter = 0;
//         while counter < size - 4 {
//             let mut b = self.read_byte()?;
//             if b == MARK {
//                 b = self.read_byte()? + 1;
//             }
//             buf[counter] = b;
//             counter += 1;
//         }
//         debug!(
//             "CMD: {}, Report: {}. Data: {:?}",
//             cmd,
//             report,
//             &buf[..counter]
//         );
//         Ok(counter - 1)
//     }
//     fn write_aime_request(&mut self, dest: u8, seq_num: &mut u8, buf: &[u8]) -> io::Result<()> {
//         let size: u8 = buf.len() as u8 + 3;
//         let mut sum = dest.wrapping_add(size).wrapping_add(*seq_num);
//
//         self.write_all(&[SYNC, size, dest, *seq_num])?;
//         *seq_num = (*seq_num + 1) % 32;
//
//         for &b in buf.iter() {
//             if b == SYNC || b == MARK {
//                 self.write(&[MARK, b - 1])?;
//             } else {
//                 self.write(&[b])?;
//             }
//
//             sum = sum.wrapping_add(b);
//         }
//
//         if sum == SYNC || sum == MARK {
//             self.write(&[MARK, sum - 1])?;
//         } else {
//             self.write(&[sum])?;
//         }
//         Ok(())
//     }
//
//     // fn read_aime_request(&mut self) -> io::Result<()> {}
// }

pub fn bit_read(input: &u8, n: usize) -> bool {
    input & (1 << n) != 0
}
