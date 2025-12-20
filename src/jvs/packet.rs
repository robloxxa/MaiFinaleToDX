const SYNC_BYTE: u8 = 0xE0;
const MARK_BYTE: u8 = 0xD0;

const SYNC_IDX: u8 = 0;
const DEST_IDX: u8 = 1;
const LEN_IDX: u8 = 2;
const DATA_START_IDX: u8 = 3;
const DATA_MAX_IDX: u8 = 255;

#[derive(Debug)]
pub enum JvsPacket<'a> {
    Valid { dest: u8, data: &'a [u8] },
    Invalid,
}

pub struct Parser {
    buf: [u8; 256],
    idx: u8,
    mark: bool,
    sum: u8,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            buf: [0; 256],
            idx: 0,
            mark: false,
            sum: 0,
        }
    }

    #[inline]
    pub fn push<'a>(&'a mut self, mut b: u8) -> Option<JvsPacket<'a>> {
        if self.mark {
            b = b.wrapping_sub(1);
            self.mark = false;
        }

        if b == MARK_BYTE {
            self.mark = true;
            return None;
        }

        match self.idx {
            SYNC_IDX => {
                if b == SYNC_BYTE {
                    self.buf[self.idx as usize] = b;
                    self.idx = 1;
                }

                None
            }
            DEST_IDX => {
                self.buf[self.idx as usize] = b;
                self.idx += 1;

                self.sum = self.sum.wrapping_add(b);

                None
            }
            LEN_IDX => {
                self.buf[self.idx as usize] = b;
                self.idx += 1;

                self.sum = self.sum.wrapping_add(b);

                None
            }
            DATA_START_IDX..=DATA_MAX_IDX => {
                let len_of_data = self.buf[LEN_IDX as usize] + 2;

                self.buf[self.idx as usize] = b;

                if self.idx == len_of_data {
                    let calculated_checksum = self.sum;
                    let received_checksum = b;

                    dbg!(calculated_checksum, received_checksum);

                    self.reset();

                    if calculated_checksum == received_checksum {
                        Some(JvsPacket::Valid {
                            dest: self.buf[DEST_IDX as usize],
                            data: &self.buf[DATA_START_IDX as usize..len_of_data as usize],
                        })
                    } else {
                        Some(JvsPacket::Invalid)
                    }
                } else {
                    self.sum = self.sum.wrapping_add(b);

                    self.idx += 1;

                    None
                }
            }
        }
    }

    fn reset(&mut self) {
        self.mark = false;
        self.idx = 0;
        self.sum = 0;
    }
}

pub struct Builder {
    buf: [u8; 512], // Double size to account for worst-case escaping (every byte escaped)
    len: usize,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            buf: [0; 512],
            len: 0,
        }
    }

    pub fn build(&mut self, dest: u8, data: &[u8]) -> &[u8] {
        self.len = 0;
        self.buf[self.len] = SYNC_BYTE;
        self.len += 1;

        let n = (data.len() + 2) as u8;
        let mut sum: u8 = 0;

        sum = sum.wrapping_add(dest);
        sum = sum.wrapping_add(n);
        for &b in data {
            sum = sum.wrapping_add(b);
        }

        // Write escaped bytes
        self.push_byte(dest);
        self.push_byte(n);

        for &b in data {
            self.push_byte(b);
        }

        self.push_byte(sum);

        &self.buf[..self.len]
    }

    fn push_byte(&mut self, b: u8) {
        if b == SYNC_BYTE || b == MARK_BYTE {
            self.buf[self.len] = MARK_BYTE;
            self.len += 1;

            self.buf[self.len] = b.wrapping_add(1);
            self.len += 1;
        } else {
            self.buf[self.len] = b;
            self.len += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_valid_packet() {
        let mut parser = Parser::new();
        let mut packet = vec![0xE0, 0xFF, 0x04, 0x01, 0x02, 0x03];

        
        let sum: u8 = 0xFFu8.wrapping_add(0x04).wrapping_add(0x01).wrapping_add(0x02).wrapping_add(0x03);
        
        packet.push(sum);

        let mut result = None;
        for &byte in &packet {
            result = parser.push(byte);
        }

        match result {
            Some(JvsPacket::Valid { dest, data }) => {
                assert_eq!(dest, 0xFF, "Destination should be 0x01");
                assert_eq!(data, &[0x01, 0x02, 0x03], "Data should be [0x01, 0x02]");
            }
            _ => panic!("Expected valid packet, got {:?}", result),
        }
    }
}
