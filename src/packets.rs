//! Contains structures and functions to parse and manipulate packets related to RingEdge 2 Maimai Cabinet.
//!
//! Note that all Packet structures contains data arrays **WITHOUT SYNC and SUM byte**,
//! because they are only need when writing/reading from devices.


use std::io::{Read, Write};

use crate::helper_funcs::{ReadExt, WriteExt};

pub mod rs232;
pub mod rs232c;

pub static SYNC_INDEX: u8 = 0;
pub trait Packet: Read + Write {
    const DATA_BEGIN_INDEX: usize;
    const SIZE_INDEX: usize;
    const DESTINATION_INDEX: usize;
    const LEN_OF_HEADER: usize;

    fn inner(&self) -> &[u8];
    fn inner_mut(&mut self) -> &mut [u8];

    /// Length of the Packet, including HEADER(SYNC, LEN, DESTINATION, etc.).
    /// 
    /// Check out [Packet::data_size] if you only need size of the data.
    fn len(&self) -> usize {
        self.inner()[Self::SIZE_INDEX] as usize + Self::LEN_OF_HEADER
    }
    
    /// Slice of the packet.
    /// 
    /// Returns the slice of the whole packet.
    fn as_slice(&self) -> &[u8] {
        &self.inner()[..self.len()]
    }

    /// Get destination byte from packet.
    fn get_dest(&self) -> u8 {
        self.inner()[Self::DESTINATION_INDEX]
    }

    /// Set destionation byte in packet.
    fn dest(&mut self, dest: u8) -> &mut Self {
        self.inner_mut()[Self::DESTINATION_INDEX] = dest;
        self
    }

    /// Returns slice of the data in packet. It doesn't include checksum.
    fn get_data(&self) -> &[u8] {
        &self.inner()[Self::DATA_BEGIN_INDEX..self.len() - 1]
    }

    /// Set data in packet.
    fn data(&mut self, data: &[u8]) -> &mut Self {
        let size = data.len() + Self::DATA_BEGIN_INDEX;
        let inner = self.inner_mut();
        inner[Self::DATA_BEGIN_INDEX..size].copy_from_slice(data);
        inner[Self::SIZE_INDEX] = (size - Self::DATA_BEGIN_INDEX) as u8;
        self
    }

    fn checksum(&mut self, checksum: u8) -> &mut Self {
        let len = self.len();
        self.inner_mut()[len] = checksum;
        self
    }

    /// Returns checksum
    fn get_checksum(&self) -> u8 {
        self.inner()[self.len()]
    }
}

