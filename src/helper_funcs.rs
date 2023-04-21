use log::debug;
use serialport;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::num::Wrapping;
use serialport::SerialPort;

pub static SYNC: u8 = 0xE0;
pub static MARK: u8 = 0xD0;

pub trait SerialExt: serialport::SerialPort {
    fn read_byte(&mut self) -> Result<u8, serialport::Error> {
        let mut read_buf: [u8; 1] = [0];
        self.read_exact(read_buf.as_mut())?;
        return Ok(read_buf[0]);
    }

    fn read_jvs_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let sync = self.read_byte()?;
        let dest = self.read_byte()?;
        let size = self.read_byte()? as usize;
        let status = self.read_byte()?; // TODO: return error if status is wrong
        let mut counter: usize = 0;

        while counter < size - 1 {
            let mut b = self.read_byte()?;
            if b == MARK {
                b = self.read_byte()? + 1;
            }
            buf[counter] = b;
            counter += 1;
        }
        debug!(
            "Read: {:X?} {:X?} {:X?} {:X?} {:X?}",
            sync,
            dest,
            size,
            status,
            &buf[..counter]
        );
        Ok(counter - 1)
    }

    fn write_jvs_packet(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        let mut writer = BufWriter::with_capacity(64, self);
        let size: u8 = data.len() as u8 + 1;
        let mut sum: u8 = dest.wrapping_add(size);

        writer.write(&[SYNC, dest, size])?;

        for &b in data.iter() {
            if b == SYNC || b == MARK {
                writer.write(&[MARK, b - 1])?;
            } else {
                writer.write(&[b])?;
            }

            sum = sum.wrapping_add(b);
        }

        if sum == SYNC || sum == MARK {
            writer.write(&[MARK, sum - 1])?;
        } else {
            writer.write(&[sum])?;
        }
        writer.flush()?;
        debug!(
            "Write: {:X?} {:X?} {:X?} {:X?} {:X?}",
            SYNC, dest, size, &data, sum
        );
        Ok(())
    }

    fn read_aime_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.read_byte() {
            Ok(x) => {
                if x != 0xE0 {
                    return Ok(0);
                }
            }
            Err(err) => return Err(io::Error::from(err)),
        }

        let size = self.read_byte()? as usize;
        self.read_byte()?;
        self.read_byte()?;
        let cmd = self.read_byte()?;
        let report = self.read_byte()?;
        let mut counter = 0;
        while counter < size - 4 {
            let mut b = self.read_byte()?;
            if b == MARK {
                b = self.read_byte()? + 1;
            }
            buf[counter] = b;
            counter += 1;
        }
        debug!(
            "CMD: {}, Report: {}. Data: {:?}",
            cmd,
            report,
            &buf[..counter]
        );
        Ok(counter - 1)
    }
    fn write_aime_packet(&mut self, dest: u8, seq_num: &mut u8, buf: &[u8]) -> io::Result<()> {
        let size: u8 = buf.len() as u8 + 3;
        let mut sum = dest.wrapping_add(size).wrapping_add(*seq_num);

        self.write_all(&[SYNC, size, dest, *seq_num])?;
        *seq_num = (*seq_num + 1) % 32;

        for &b in buf.iter() {
            if b == SYNC || b == MARK {
                self.write(&[MARK, b - 1])?;
            } else {
                self.write(&[b])?;
            }

            sum = sum.wrapping_add(b);
        }

        if sum == SYNC || sum == MARK {
            self.write(&[MARK, sum - 1])?;
        } else {
            self.write(&[sum])?;
        }
        Ok(())
    }
}

pub trait ReadExt: Read {
    fn read_u8(&mut self) -> io::Result<u8> {
        let buf = &mut [0u8; 1];
        self.read(buf)?;
        Ok(buf[0])
    }
}

impl ReadExt for dyn Read {}

impl ReadExt for dyn SerialPort {}

pub fn bit_read(input: &u8, n: usize) -> bool {
    input & (1 << n) != 0
}
