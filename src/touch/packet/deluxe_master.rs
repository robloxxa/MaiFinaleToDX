const MAX_PACKET_SIZE: usize = 4;

#[derive(Debug, PartialEq)]
pub enum Packet {
    Reset,
    Halt,
    Stat,
    Rotate([u8; 4]),
    Threshold([u8; 4]),
}

pub struct Parser {
    inner: [u8; MAX_PACKET_SIZE],
    idx: usize,
    in_frame: bool,
}

impl Default for Parser {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: [0u8; MAX_PACKET_SIZE],
            idx: 0,
            in_frame: false,
        }
    }

    #[inline]
    pub fn push(&mut self, b: u8) -> Option<Packet> {
        match b {
            b'{' => {
                self.in_frame = true;
                self.idx = 0;
                None
            }

            b'}' => {
                if !self.in_frame {
                    return None;
                }

                self.in_frame = false;

                if self.idx != MAX_PACKET_SIZE {
                    return None;
                }

                let data =
                    // SAFETY: idx == MAX_PACKET_SIZE == 4, so inner[..4] is the full array
                    unsafe { *(self.inner.as_ptr() as *const [u8; MAX_PACKET_SIZE]) };

                match data[2] {
                    b'E' => Some(Packet::Reset),
                    b'L' => Some(Packet::Halt),
                    b'A' => Some(Packet::Stat),
                    b'r' => Some(Packet::Rotate(data)),
                    b'k' => Some(Packet::Threshold(data)),
                    _ => None,
                }
            }

            _ => {
                if !self.in_frame {
                    return None;
                }

                if self.idx < MAX_PACKET_SIZE {
                    unsafe {
                        // SAFETY: idx < MAX_PACKET_SIZE checked above
                        *self.inner.get_unchecked_mut(self.idx) = b;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn push_frame(parser: &mut Parser, data: &[u8]) -> Option<Packet> {
        parser.push(b'(');
        for &b in data {
            let result = parser.push(b);
            if result.is_some() {
                return result;
            }
        }
        parser.push(b')')
    }

    #[test]
    fn test_reset_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x00, 0x00, b'E', 0x00]);
        assert_eq!(result, Some(Packet::Reset));
    }

    #[test]
    fn test_halt_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x00, 0x00, b'L', 0x00]);
        assert_eq!(result, Some(Packet::Halt));
    }

    #[test]
    fn test_stat_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x00, 0x00, b'A', 0x00]);
        assert_eq!(result, Some(Packet::Stat));
    }

    #[test]
    fn test_rotate_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x01, 0x02, b'r', 0x03]);
        assert_eq!(result, Some(Packet::Rotate([0x01, 0x02, b'r', 0x03])));
    }

    #[test]
    fn test_threshold_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x04, 0x05, b'k', 0x06]);
        assert_eq!(result, Some(Packet::Threshold([0x04, 0x05, b'k', 0x06])));
    }

    #[test]
    fn test_unknown_command() {
        let mut parser = Parser::new();
        let result = push_frame(&mut parser, &[0x00, 0x00, b'Z', 0x00]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_incomplete_packet() {
        let mut parser = Parser::new();
        parser.push(b'(');
        parser.push(0x01);
        parser.push(0x02);
        let result = parser.push(b')');
        assert_eq!(result, None);
    }

    #[test]
    fn test_overflow_resets_frame() {
        let mut parser = Parser::new();
        parser.push(b'(');
        for _ in 0..MAX_PACKET_SIZE {
            parser.push(0xFF);
        }
        parser.push(0xFF);
        assert!(!parser.in_frame);
        assert_eq!(parser.push(b')'), None);
    }

    #[test]
    fn test_nested_open_bracket_resets() {
        let mut parser = Parser::new();
        parser.push(b'(');
        parser.push(0xAA);
        parser.push(0xBB);

        parser.push(b'(');
        assert_eq!(parser.idx, 0);

        let result = push_frame(&mut parser, &[0x00, 0x00, b'A', 0x00]);
        // The first `(` from push_frame starts a new frame again
        assert_eq!(result, Some(Packet::Stat));
    }

    #[test]
    fn test_multiple_sequential_packets() {
        let mut parser = Parser::new();

        let r1 = push_frame(&mut parser, &[0x00, 0x00, b'E', 0x00]);
        assert_eq!(r1, Some(Packet::Reset));

        let r2 = push_frame(&mut parser, &[0x00, 0x00, b'A', 0x00]);
        assert_eq!(r2, Some(Packet::Stat));

        let r3 = push_frame(&mut parser, &[0x01, 0x02, b'r', 0x03]);
        assert_eq!(r3, Some(Packet::Rotate([0x01, 0x02, b'r', 0x03])));
    }

    #[test]
    fn test_close_without_open() {
        let mut parser = Parser::new();
        assert_eq!(parser.push(b')'), None);
    }

    #[test]
    fn test_data_without_frame() {
        let mut parser = Parser::new();
        assert_eq!(parser.push(0x01), None);
        assert_eq!(parser.push(0x02), None);
    }

    #[test]
    fn test_recovery_after_overflow() {
        let mut parser = Parser::new();

        parser.push(b'(');
        for _ in 0..10 {
            parser.push(0xFF);
        }

        let result = push_frame(&mut parser, &[0x00, 0x00, b'L', 0x00]);
        assert_eq!(result, Some(Packet::Halt));
    }

    #[test]
    fn test_parser_default() {
        let p1 = Parser::new();
        let p2 = Parser::default();
        assert_eq!(p1.idx, p2.idx);
        assert_eq!(p1.in_frame, p2.in_frame);
    }
}
