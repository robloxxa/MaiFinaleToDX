// Packets structures for JVS protocol
pub trait Packet {
    fn buf_len(&self) -> usize;
    fn data(&mut self) -> &mut [u8];
    fn dest(&self) -> u8;
    fn raw_size(&self) -> u8;
    fn set_dest(&mut self, dest: u8) -> &mut Self;
    fn set_raw_size(&mut self, raw_size: u8) -> &mut Self;
    fn set_data(&mut self, new_data: &[u8]) -> &mut Self;
}

pub mod rs232 {
    use super::Packet;
    use crate::helper_funcs::{MARK, SYNC};
    use log::debug;
    use std::hash::Hasher;
    use std::io;
    use std::io::Write;
    use std::pin::Pin;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    #[derive(Debug)]
    pub enum ResponseStatus {
        Ok,
        UnknownCommand,
        ChecksumError,
        AcknowledgeOverflow,
        UnknownStatus,
    }

    impl From<u8> for ResponseStatus {
        fn from(value: u8) -> Self {
            match value {
                1 => Self::Ok,
                2 => Self::UnknownCommand,
                3 => Self::ChecksumError,
                4 => Self::AcknowledgeOverflow,
                _ => Self::UnknownStatus,
            }
        }
    }

    impl Into<u8> for ResponseStatus {
        fn into(self) -> u8 {
            match self {
                Self::Ok => 1,
                Self::UnknownCommand => 2,
                Self::ChecksumError => 3,
                Self::AcknowledgeOverflow => 4,
                Self::UnknownStatus => 0xFF,
            }
        }
    }

    /// A struct for RS232 RequestPacket.
    /// Note that RequestPacket.buf does not include SYNC and SUM bytes,
    /// since they are only necessary when writing/reading to a port.
    ///
    /// The buf array look like this
    /// [destination, size, data_0, ..., data_N]
    /// N = 0..255
    ///
    /// The only difference from ResponsePacket that RequestPacket doesn't have STATUS byte
    #[derive(Debug, Copy, Clone)]
    pub struct RequestPacket {
        buf: [u8; 257],
    }

    impl Packet for RequestPacket {
        // +2 for DEST and SIZE bytes and -1 for SUM byte
        fn buf_len(&self) -> usize {
            self.raw_size() as usize + 2 - 1
        }

        fn data(&mut self) -> &mut [u8] {
            let len = self.buf_len();
            &mut self.buf[3..len + 1]
        }

        fn dest(&self) -> u8 {
            self.buf[0]
        }

        fn set_dest(&mut self, dest: u8) -> &mut Self {
            self.buf[0] = dest;
            self
        }

        fn raw_size(&self) -> u8 {
            self.buf[1]
        }

        fn set_raw_size(&mut self, raw_size: u8) -> &mut Self {
            self.buf[1] = raw_size;
            self
        }

        fn set_data(&mut self, new_data: &[u8]) -> &mut Self {
            self.set_raw_size(new_data.len() as u8 + 1);
            self.data().copy_from_slice(new_data);
            self
        }
    }

    impl RequestPacket {
        pub fn new(dest: u8, data: &[u8]) -> Self {
            let mut packet = Self { buf: [0; 257] };
            packet.set_dest(dest).set_data(data);

            packet
        }

        pub async fn write(&mut self, mut writer: Pin<&mut impl AsyncWrite>) -> io::Result<()> {
            let mut sum = 0u8;
            writer.write_u8(SYNC).await?;
            for &b in &self.buf[..self.buf_len()] {
                if b == MARK || b == SYNC {
                    writer.write_u8(MARK).await?;
                    writer.write_u8(b - 1).await?;
                } else {
                    writer.write_u8(b).await?;
                }

                sum = sum.wrapping_add(b);
            }

            if sum == MARK || sum == SYNC {
                writer.write_u8(MARK).await?;
                writer.write_u8(sum - 1).await?;
            } else {
                writer.write_u8(sum).await?;
            }

            writer.flush().await?;

            debug!(
                "RS232 Packet Sent: {:02X?} {:02X?} {:02X?} {:02X?} {:02X?}",
                SYNC,
                self.dest(),
                self.raw_size(),
                &self.buf[..self.buf_len()],
                sum
            );

            Ok(())
        }

        pub async fn read(&mut self, mut reader: Pin<&mut impl AsyncRead>) -> io::Result<()> {
            let mut sum = 0u8;
            reader.read_u8().await?;

            self.set_dest(reader.read_u8().await?)
                .set_raw_size(reader.read_u8().await?);

            let counter = 0usize;
            let data = self.data();
            let mut b: u8;

            while counter < data.len() {
                b = reader.read_u8().await?;
                if b == MARK {
                    b = reader.read_u8().await? + 1;
                }
                data[counter] = b;

                sum = sum.wrapping_add(b);
            }

            sum = sum.wrapping_add(self.dest()).wrapping_add(self.raw_size());

            if sum != reader.read_u8().await? {
                todo!()
            }

            Ok(())
        }
    }

    /// A struct for RS232 ResponsePacket.
    /// Note that ResponsePacket.buf does not include SYNC and SUM bytes,
    /// since they are only necessary when writing to a port.
    ///
    /// The buf array look like this
    /// [destination, size, status, data_0, ..., data_N]
    /// N = 0..255
    ///
    /// The only difference from RequestPacket that ResponsePacket have STATUS byte
    #[derive(Debug, Copy, Clone)]
    pub struct ResponsePacket {
        buf: [u8; 257],
    }

    impl Packet for ResponsePacket {
        fn buf_len(&self) -> usize {
            self.raw_size() as usize + 2 - 2
        }

        fn data(&mut self) -> &mut [u8] {
            let len = self.buf_len();
            &mut self.buf[4..len]
        }

        fn dest(&self) -> u8 {
            self.buf[0]
        }

        fn set_dest(&mut self, dest: u8) -> &mut Self {
            self.buf[0] = dest;
            self
        }

        fn raw_size(&self) -> u8 {
            self.buf[1]
        }

        fn set_raw_size(&mut self, raw_size: u8) -> &mut Self {
            self.buf[1] = raw_size;
            self
        }

        fn set_data(&mut self, new_data: &[u8]) -> &mut Self {
            self.set_raw_size(new_data.len() as u8 + 2); // +2 for SUM and STATUS bytes
            let len = self.buf_len();
            self.buf[4..len].copy_from_slice(new_data);
            self
        }
    }

    impl ResponsePacket {
        pub fn new(dest: u8, data: &[u8]) -> Self {
            let mut packet = Self { buf: [0; 257] };
            packet.set_dest(dest).set_data(data);
            packet.set_status(ResponseStatus::Ok);
            packet
        }

        pub fn status(&self) -> ResponseStatus {
            ResponseStatus::from(self.buf[3])
        }

        pub fn set_status(&mut self, status: ResponseStatus) -> &mut Self {
            self.buf[3] = status.into();

            self
        }

        pub async fn send(&mut self, mut writer: Pin<&mut impl AsyncWrite>) -> io::Result<()> {
            let mut sum = 0u8;
            writer.write_u8(SYNC).await?;
            for &b in &self.buf[..self.buf_len()] {
                if b == MARK || b == SYNC {
                    writer.write_u8(MARK).await?;
                    writer.write_u8(b - 1).await?;
                } else {
                    writer.write_u8(b).await?;
                }

                sum = sum.wrapping_add(b);
            }

            if sum == MARK || sum == SYNC {
                writer.write_u8(MARK).await?;
                writer.write_u8(sum - 1).await?;
            } else {
                writer.write_u8(sum).await?;
            }

            writer.flush().await?;

            debug!(
                "RS232 Packet Sent: {:02X?} {:02X?} {:02X?} {:02X?} {:02X?}",
                SYNC,
                self.dest(),
                self.raw_size(),
                &self.buf[..self.buf_len()],
                sum
            );

            Ok(())
        }

        pub async fn read(&mut self, mut reader: Pin<&mut impl AsyncRead>) -> io::Result<()> {
            let mut sum = 0u8;
            reader.read_u8().await?;

            self.set_dest(reader.read_u8().await?)
                .set_raw_size(reader.read_u8().await?);

            let counter = 0usize;
            let data = &mut self.data();
            let mut b: u8;

            while counter < data.len() {
                b = reader.read_u8().await?;
                if b == MARK {
                    b = reader.read_u8().await? + 1;
                }
                data[counter] = b;

                sum = sum.wrapping_add(b);
            }

            sum = sum.wrapping_add(self.dest()).wrapping_add(self.raw_size());

            if sum != reader.read_u8().await? {
                todo!()
            }

            Ok(())
        }
    }
}

pub mod rs232c {
    use super::Packet;
    use tokio_serial::new;

    /// A struct for RS232C RequestPacket.
    /// It works pretty same as RS232 but has some difference in structure.
    /// Note that RequestPacket.buf does not include SYNC and SUM bytes,
    /// since they are only necessary when writing/reading to a port.
    ///
    /// The buf array look like this
    /// [size, destination, sequence, command, data_0, ..., data_N]
    /// N = 0..255
    ///
    /// The only difference from ResponsePacket that RequestPacket doesn't have STATUS byte
    #[derive(Debug, Copy, Clone)]
    pub struct RequestPacket {
        buf: [u8; 512],
    }

    impl Packet for RequestPacket {
        fn buf_len(&self) -> usize {
            self.buf[0] as usize - 1
        }

        fn data(&mut self) -> &mut [u8] {
            let len = self.buf_len();
            &mut self.buf[3..len]
        }

        fn dest(&self) -> u8 {
            self.buf[1]
        }

        fn raw_size(&self) -> u8 {
            self.buf[0]
        }

        fn set_dest(&mut self, dest: u8) -> &mut Self {
            self.buf[1] = dest;
            self
        }

        fn set_raw_size(&mut self, raw_size: u8) -> &mut Self {
            self.buf[0] = raw_size;

            self
        }

        fn set_data(&mut self, new_data: &[u8]) -> &mut Self {
            self.set_raw_size(new_data.len() as u8 + 2); // +2 for SUM and STATUS bytes
            let len = self.buf_len();
            self.buf[3..len].copy_from_slice(new_data);
            self
        }
    }

    #[derive(Debug)]
    pub struct ResponsePacket {
        size: usize,
        dest: u8,
        seq_num: u8,
        cmd: u8,
        report: u8,
        data: [u8; 512],
    }

    impl ResponsePacket {}
}

#[cfg(test)]
mod test {
    use super::*;
    use std::pin::Pin;
    use tokio::io::AsyncWrite;
    use tokio::io::BufWriter;
    #[tokio::test]
    async fn test_rs232_req_packet() -> tokio_serial::Result<()> {
        let mut d = Vec::<u8>::with_capacity(64);
        let mut packet = rs232::RequestPacket::new(0xFF, &[0xF0, 0xD9]);
        let mut buf_writer = BufWriter::new(d);
        println!("{:?}", packet.data());
        assert_eq!(packet.data(), &[0xF0, 0xD9]);
        packet.write(Pin::new(&mut buf_writer)).await?;
        assert_eq!(
            buf_writer.get_ref().as_slice(),
            &[0xE0, 0xFF, 3, 0xF0, 0xD9, 171]
        );

        Ok(())
    }
}
