
const MAX_PACKET_SIZE: usize = 12;
const MAX_DATA_PACKET_SIZE: usize = 4;

pub enum Packet<'a> {
    Input {
        p1: &'a [u8],
        p2: &'a [u8]
    },
    Data(&'a [u8]),
    Incompleted,
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

    pub fn push(&mut self, b: u8) -> Packet<'_> {
        match b {
            b'(' => {
                self.in_frame = true;
                self.idx = 0;
                Packet::Incompleted
            }

            b')' => {
                if !self.in_frame {
                    return Packet::Incompleted;
                }

                self.in_frame = false;

                match self.idx {
                    MAX_DATA_PACKET_SIZE => Packet::Data(&self.inner[..self.idx]),
                    MAX_PACKET_SIZE => Packet::Input(&self.inner[..self.idx].),
                    _ => ParsedPacket::Incompleted,
                }
            }

            _ => {
                if !self.in_frame {
                    return ParsedPacket::Incompleted; 
                }

                if self.idx < MAX_INPUT_PACKET_SIZE {
                    self.inner[self.idx] = b;
                    self.idx += 1;
                } else {
                    self.in_frame = false;
                    self.idx = 0;
                }

                ParsedPacket::Incompleted
            }
        }
    }
}