const MAX_PACKET_SIZE: usize = 12;
const MAX_DATA_PACKET_SIZE: usize = 4;

const P1_INPUT_RANGE: std::ops::Range<usize> = 0..4;
const P2_INPUT_RANGE: std::ops::Range<usize> = 6..10;

#[derive(Debug)]
pub enum Packet {
    Input { p1: [u8; 4], p2: [u8; 4] },
    Data([u8; 4]),
}

pub struct Parser {
    inner: [u8; MAX_PACKET_SIZE],
    idx: usize,
    in_frame: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            idx: 0,
            in_frame: false,
        }
    }

    #[inline]
    pub fn push(&mut self, b: u8) -> Option<Packet> {
        match b {
            b'(' => {
                self.in_frame = true;
                self.idx = 0;
                None
            }

            b')' => {
                if !self.in_frame {
                    return None;
                }

                self.in_frame = false;

                match self.idx {
                    MAX_DATA_PACKET_SIZE => {
                        let mut data = [0u8; MAX_DATA_PACKET_SIZE];

                        data.copy_from_slice(&self.inner[..self.idx]);

                        Some(Packet::Data(data))
                    }
                    MAX_PACKET_SIZE => {
                        let mut p1 = [0u8; MAX_DATA_PACKET_SIZE];
                        let mut p2 = [0u8; MAX_DATA_PACKET_SIZE];

                        p1.copy_from_slice(&self.inner[P1_INPUT_RANGE]);
                        p2.copy_from_slice(&self.inner[P2_INPUT_RANGE]);

                        Some(Packet::Input { p1: p1, p2: p2 })
                    }
                    _ => None,
                }
            }

            _ => {
                if !self.in_frame {
                    return None;
                }

                if self.idx < MAX_PACKET_SIZE {
                    self.inner[self.idx] = b;
                    self.idx += 1;
                } else {
                    self.in_frame = false;
                    self.idx = 0;
                }

                None
            }
        }
    }
}
