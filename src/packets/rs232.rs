#![allow(dead_code)]

use crate::helper_funcs::{ReadExt, WriteExt, SYNC};
use std::io;

const SYNC_INDEX: usize = 0;
const DESTINATION_INDEX: usize = 1;
const SIZE_INDEX: usize = 2;
const LEN_OF_HEADER: usize = 3;

pub trait Packet {
    const DATA_BEGIN_INDEX: usize;

    fn get_buf(&self) -> &[u8];
    fn get_mut_buf(&mut self) -> &mut [u8];

    fn get_slice(&self) -> &[u8] {
        &self.get_buf()[..self.len()]
    }

    fn len(&self) -> usize {
        self.get_buf()[SIZE_INDEX] as usize + LEN_OF_HEADER
    }

    fn dest(&self) -> u8 {
        self.get_buf()[DESTINATION_INDEX]
    }

    fn set_dest(&mut self, dest: u8) -> &mut Self {
        self.get_mut_buf()[DESTINATION_INDEX] = dest;
        self
    }

    fn data(&mut self) -> &[u8] {
        let len = self.len();
        &mut self.get_mut_buf()[Self::DATA_BEGIN_INDEX..len - 1]
    }

    // NOTE: This method **DOES NOT COUNT CHECKSUM**
    fn set_data(&mut self, data: &[u8]) -> &mut Self {
        let size = data.len() + Self::DATA_BEGIN_INDEX;
        self.get_mut_buf()[Self::DATA_BEGIN_INDEX..size].copy_from_slice(data);
        self.get_mut_buf()[SIZE_INDEX] = (size - LEN_OF_HEADER + 1) as u8;
        self
    }

    fn checksum(&self) -> u8 {
        self.get_buf()[self.len()]
    }

    fn read(&mut self, reader: &mut dyn ReadExt) -> io::Result<&mut Self> {
        read_packet(reader, &mut self.get_mut_buf())?;
        Ok(self)
    }

    fn write(&mut self, writer: &mut dyn WriteExt) -> io::Result<()> {
        let len = self.len();
        self.get_mut_buf()[len - 1] = write_packet(writer, &self.get_mut_buf()[..len])?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RequestPacket<const N: usize = 256> {
    buffer: [u8; N],
}

impl<const N: usize> Packet for RequestPacket<N> {
    const DATA_BEGIN_INDEX: usize = 3;

    fn get_buf(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl<const N: usize> RequestPacket<N> {
    pub fn new(dest: u8, data: &[u8]) -> Self {
        let mut packet = RequestPacket::default();

        packet.set_dest(dest).set_data(data);

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
        buffer[SIZE_INDEX] = 1;
        Self { buffer }
    }
}

#[derive(Debug)]
pub struct ResponsePacket<const N: usize = 256> {
    buffer: [u8; N],
}

impl<const N: usize> Packet for ResponsePacket<N> {
    const DATA_BEGIN_INDEX: usize = 4;

    fn get_buf(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl<const N: usize> ResponsePacket<N> {
    const STATUS_INDEX: usize = 3;

    pub fn new(dest: u8, data: &[u8]) -> Self {
        let mut packet = ResponsePacket::default();

        packet.set_dest(dest).set_data(data);

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

    pub fn status(&self) -> u8 {
        self.buffer[Self::STATUS_INDEX]
    }

    pub fn set_status(&mut self, status: u8) -> &mut Self {
        self.buffer[Self::STATUS_INDEX] = status;
        self
    }
}

impl<const N: usize> Default for ResponsePacket<N> {
    fn default() -> Self {
        let mut buffer = [0u8; N];
        buffer[SYNC_INDEX] = SYNC;
        buffer[SIZE_INDEX] = 2;
        buffer[Self::STATUS_INDEX] = 1;
        Self { buffer }
    }
}

pub fn read_packet(reader: &mut dyn ReadExt, buf: &mut [u8]) -> io::Result<usize> {
    buf[SYNC_INDEX] = reader.read_u8()?;
    buf[DESTINATION_INDEX] = reader.read_u8_escaped()?;
    buf[SIZE_INDEX] = reader.read_u8_escaped()?;

    let mut counter: usize = 0;
    // TODO: Add a sum check, its not really necessary but would be great if JVS fails (never happened)
    while counter < buf[SIZE_INDEX] as usize {
        buf[SIZE_INDEX + 1..][counter] = reader.read_u8_escaped()?;
        counter += 1;
    }

    // Add DESTINATION and SIZE bytes to buffer size
    counter += 3;

    Ok(counter)
}

/// Returns checksum
pub fn write_packet(writer: &mut dyn WriteExt, buf: &[u8]) -> io::Result<u8> {
    let mut sum: u8 = 0;

    writer.write_u8(SYNC)?;

    for &b in &buf[1..buf.len() - 1] {
        writer.write_u8_escaped(b)?;
        sum = sum.wrapping_add(b);
    }

    writer.write_u8_escaped(sum)?;
    writer.flush()?;
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::packets::rs232::{Packet, RequestPacket, ResponsePacket};
    use std::io::BufReader;

    #[test]
    pub fn req_packet_new() {
        let data: &[u8] = &[0x01, 0x02];
        let dest = 0xFF;
        let mut packet: RequestPacket = RequestPacket::new(dest, data);
        assert_eq!(packet.dest(), dest);
        assert_eq!(packet.data(), data);
        assert_eq!(
            &packet.buffer[..packet.len() - 1],
            &[0xE0, 0xFF, 0x03, 0x01, 0x02]
        );
    }

    #[test]
    pub fn req_packet_new_from_read() {
        let d: &[u8] = &[0xE0, 0x00, 3, 0x01, 0x02, 0x06];

        let mut buf_reader = BufReader::new(d);

        let mut packet: RequestPacket = RequestPacket::new_from_read(&mut buf_reader).unwrap();
        assert_eq!(packet.dest(), 0x00);
        assert_eq!(packet.data(), &[0x01, 0x02]);
        assert_eq!(&packet.buffer[..packet.len()], &d[..d.len()]);
    }

    #[test]
    pub fn req_packet_write() {
        let mut d: Vec<u8> = Vec::new();

        let mut packet: RequestPacket<64> = RequestPacket::new(0xFF, &[0x01, 0x02]);
        packet.write(&mut d).unwrap();

        assert_eq!(&packet.buffer[..packet.len()], d.as_slice());
    }

    #[test]
    pub fn res_packet_new() {
        let mut packet: ResponsePacket = ResponsePacket::new(0xFF, &[0x01, 0x02, 0x03]);

        assert_eq!(packet.dest(), 0xFF);
        assert_eq!(packet.data(), &[0x01, 0x02, 0x03]);
        assert_eq!(
            packet.get_slice(),
            &[0xE0, 0xFF, 0x05, 0x01, 0x01, 0x02, 0x03, 0x00]
        );
    }

    #[test]
    pub fn res_packet_new_from_read() {
        let d: &[u8] = &[0xE0, 0xFF, 4, 0x01, 0x01, 0x02, 0x07];

        let mut buf_reader = BufReader::new(d);

        let mut packet: ResponsePacket = ResponsePacket::new_from_read(&mut buf_reader).unwrap();

        assert_eq!(packet.dest(), 0xFF);
        assert_eq!(packet.data(), &[0x01, 0x02]);
        assert_eq!(packet.status(), 0x01);
        assert_eq!(packet.get_slice(), &[0xE0, 0xFF, 4, 0x01, 0x01, 0x02, 0x07]);
    }

    #[test]
    pub fn res_packet_write() {
        let mut d: Vec<u8> = Vec::new();

        let mut packet: ResponsePacket = ResponsePacket::new(0xFF, &[0x01, 0x02]);
        packet.write(&mut d).unwrap();

        assert_eq!(&packet.buffer[..packet.len()], d.as_slice());
    }
}
