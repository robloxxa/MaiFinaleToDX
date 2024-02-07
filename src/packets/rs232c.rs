#![allow(dead_code)]

use crate::helper_funcs::{ReadExt, WriteExt, SYNC};

use std::io;

use super::Packet;

const SYNC_INDEX: usize = 0;
const SIZE_INDEX: usize = 1;
const DESTINATION_INDEX: usize = 2;
const SEQUENCE_INDEX: usize = 3;
const COMMAND_INDEX: usize = 4;
const REPORT_INDEX: usize = 5;

const LEN_OF_HEADER: usize = 2;

pub trait ReaderPacket: Packet {
    const SEQUENCE_INDEX: usize;
    const COMMAND_INDEX: usize;

    fn get_seq_num(&self) -> u8 {
        self.inner()[SEQUENCE_INDEX]
    }

    fn seq_num(&mut self, seq_num: u8) -> &mut Self {
        self.inner_mut()[SEQUENCE_INDEX] = seq_num;
        self
    }

    fn cmd(&self) -> u8 {
        self.get_buf()[COMMAND_INDEX]
    }

    fn set_cmd(&mut self, cmd: u8) -> &mut Self {
        self.get_mut_buf()[COMMAND_INDEX] = cmd;
        self
    }



    // fn read(&mut self, reader: &mut dyn ReadExt) -> io::Result<&mut Self> {
    //     read_packet(reader, self.get_mut_buf())?;
    //     Ok(self)
    // }

    // fn write(&mut self, writer: &mut dyn WriteExt) -> io::Result<()> {
    //     let len = self.len();
    //     self.get_mut_buf()[len - 1] = write_packet(writer, &self.get_mut_buf()[..len])?;
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct RequestPacket<const N: usize = 256> {
    buffer: [u8; N],
}

impl<const N: usize> Packet for RequestPacket<N> {
    const SIZE_INDEX: usize = 1;
    const DESTINATION_INDEX: usize = 2;
    const DATA_BEGIN_INDEX: usize = 5;

    fn inner(&self) -> &[u8] {
        &self.buffer
    }

    fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl<const N: usize> RequestPacket<N> {
    pub fn new(dest: u8, cmd: u8, data: &[u8]) -> Self {
        let mut packet = RequestPacket::default();

        packet.set_dest(dest).set_cmd(cmd).set_data(data);

        packet
    }

    pub fn new_from_read(reader: &mut dyn ReadExt) -> io::Result<Self> {
        let mut packet = RequestPacket::default();
        packet.read(reader)?;
        Ok(packet)
    }

    pub fn from_raw_packet(raw_packet: &[u8]) -> Self {
        let mut buffer = [0u8; N];

        buffer[..raw_packet.len()].copy_from_slice(raw_packet);

        Self { buffer }
    }
}

impl<const N: usize> Default for RequestPacket<N> {
    fn default() -> Self {
        let mut buffer = [0u8; N];
        buffer[SYNC_INDEX] = SYNC;
        buffer[SIZE_INDEX] = ResponsePacket::<N>::DATA_BEGIN_INDEX as u8;
        Self { buffer }
    }
}

#[derive(Debug)]
pub struct ResponsePacket<const N: usize = 256> {
    buffer: [u8; N],
}

impl<const N: usize> Packet for ResponsePacket<N> {
    const DATA_BEGIN_INDEX: usize = 6;

    fn get_buf(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl<const N: usize> ResponsePacket<N> {
    pub fn new(dest: u8, cmd: u8, data: &[u8]) -> Self {
        let mut packet = ResponsePacket::default();

        packet.set_dest(dest).set_cmd(cmd).set_data(data);

        packet
    }

    pub fn new_from_read(reader: &mut dyn ReadExt) -> io::Result<Self> {
        let mut packet = ResponsePacket::default();
        packet.read(reader)?;
        Ok(packet)
    }

    pub fn from_raw_packet(raw_packet: &[u8]) -> Self {
        let mut buffer = [0u8; N];

        buffer[..raw_packet.len()].copy_from_slice(raw_packet);

        Self { buffer }
    }

    pub fn report(&self) -> u8 {
        self.get_buf()[REPORT_INDEX]
    }

    pub fn set_report(&mut self, report: u8) -> &mut Self {
        self.get_mut_buf()[REPORT_INDEX] = report;
        self
    }
}

impl<const N: usize> Default for ResponsePacket<N> {
    fn default() -> Self {
        let mut buffer = [0u8; N];
        buffer[SYNC_INDEX] = SYNC;
        buffer[SIZE_INDEX] = ResponsePacket::<N>::DATA_BEGIN_INDEX as u8;
        Self { buffer }
    }
}

pub fn read_packet(reader: &mut dyn ReadExt, buf: &mut [u8]) -> io::Result<usize> {
    buf[SYNC_INDEX] = reader.read_u8()?;
    buf[SIZE_INDEX] = reader.read_u8_escaped()?;

    let mut counter: usize = 0;
    while counter < buf[SIZE_INDEX] as usize {
        buf[SIZE_INDEX + 1..][counter] = reader.read_u8_escaped()?;
        counter += 1;
    }

    counter += 2;

    Ok(counter)
}

/// Returns checksum
pub fn write_packet(writer: &mut dyn WriteExt, data: &[u8]) -> io::Result<u8> {
    let mut sum: u8 = 0;

    writer.write_u8(SYNC)?;

    for &b in &data[1..data.len() - 1] {
        writer.write_u8_escaped(b)?;
        sum = sum.wrapping_add(b);
    }

    writer.write_u8_escaped(sum)?;

    writer.flush()?;

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::packets::rs232c::{Packet, RequestPacket, ResponsePacket};
    use std::io::BufReader;

    #[test]
    pub fn req_packet_new() {
        let d: &[u8] = &[0xE0, 0x06, 0xFF, 0x00, 0x02, 0x01, 0x02];
        let data: &[u8] = &[0x01, 0x02];
        let dest = 0xFF;
        let cmd = 0x02;
        let mut packet: RequestPacket = RequestPacket::new(dest, cmd, data);
        assert_eq!(packet.dest(), dest);
        assert_eq!(packet.data(), data);
        assert_eq!(packet.cmd(), cmd);
        assert_eq!(&packet.buffer[..packet.len() - 1], d,);
    }

    #[test]
    pub fn req_packet_new_from_read() {
        let d: &[u8] = &[0xE0, 0x05, 0x00, 0x00, 0x02, 0x00, 0x07];

        let mut buf_reader = BufReader::new(d);

        let mut packet: RequestPacket = RequestPacket::new_from_read(&mut buf_reader).unwrap();
        assert_eq!(packet.dest(), 0x00);
        assert_eq!(packet.cmd(), 0x02);
        assert_eq!(packet.data(), &[0x00]);
        assert_eq!(&packet.buffer[..packet.len()], &d[..d.len()]);
    }

    #[test]
    pub fn req_packet_write() {
        let mut d: Vec<u8> = Vec::new();

        let mut packet: RequestPacket<64> = RequestPacket::new(0xFF, 0x02, &[0x01, 0x02]);
        packet.write(&mut d).unwrap();
        assert_eq!(packet.get_slice(), d.as_slice());
    }

    #[test]
    pub fn res_packet_new() {
        let dest = 0xFF;
        let cmd = 0x02;
        let d: &[u8] = &[0xE0, 0x08, dest, 0x00, cmd, 0x00, 0x01, 0x02, 0x03, 0x16];
        let mut packet: ResponsePacket = ResponsePacket::new(0xFF, cmd, &[0x01, 0x02, 0x03]);

        assert_eq!(packet.dest(), 0xFF);
        assert_eq!(packet.data(), &[0x01, 0x02, 0x03]);
        assert_eq!(&packet.buffer[..packet.len() - 1], &d[..d.len() - 1]);
    }

    #[test]
    pub fn res_packet_new_from_read() {
        let dest = 0xFF;
        let cmd = 0x02;
        let d: &[u8] = &[0xE0, 0x08, dest, 0x00, cmd, 0x01, 0x01, 0x02, 0x03, 0x16];

        let mut buf_reader = BufReader::new(d);

        let mut packet: ResponsePacket = ResponsePacket::new_from_read(&mut buf_reader).unwrap();

        assert_eq!(packet.dest(), 0xFF);
        assert_eq!(packet.data(), &[0x01, 0x02, 0x03]);
        assert_eq!(packet.report(), 0x01);
        assert_eq!(packet.get_slice(), d);
    }

    #[test]
    pub fn res_packet_write() {
        let mut d: Vec<u8> = Vec::new();

        let mut packet: ResponsePacket = ResponsePacket::new(0xFF, 0x02, &[0x01, 0x02]);
        packet.write(&mut d).unwrap();

        assert_eq!(packet.get_slice(), d.as_slice());
    }
}
