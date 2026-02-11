const MAX_PACKET_SIZE: usize = 12;
const MAX_DATA_PACKET_SIZE: usize = 4;

#[derive(Debug, PartialEq)]
pub enum Packet {
    Input { p1: [u8; 4], p2: [u8; 4] },
    Data([u8; 4]),
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
            b'(' => {
                if self.in_frame {
                    self.idx = 0;
                }
                
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
                        // SAFETY: We know idx == 4, so slice is exactly [u8; 4]
                        let data = unsafe {
                            *(self.inner.as_ptr() as *const [u8; MAX_DATA_PACKET_SIZE])
                        };

                        Some(Packet::Data(data))
                    }
                    MAX_PACKET_SIZE => {
                        // SAFETY: Ranges are statically validated at compile time
                        let p1 = unsafe {
                            *(self.inner.as_ptr() as *const [u8; MAX_DATA_PACKET_SIZE])
                        };
                        let p2 = unsafe {
                            *(self.inner.as_ptr().add(6) as *const [u8; MAX_DATA_PACKET_SIZE])
                        };

                        Some(Packet::Input { p1, p2 })
                    }
                    _ => None,
                }
            }

            _ => {
                if !self.in_frame {
                    return None;
                }

                if self.idx < MAX_PACKET_SIZE {
                    unsafe {
                        // SAFETY: We just checked idx < MAX_PACKET_SIZE
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

    #[test]
    fn test_data_packet_valid() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b'C'), None);
        assert_eq!(parser.push(b'D'), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data(_))));

        if let Some(Packet::Data(data)) = result {
            assert_eq!(data, [b'A', b'B', b'C', b'D']);
        }
    }

    #[test]
    fn test_input_packet_valid() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(0x01), None);
        assert_eq!(parser.push(0x02), None);
        assert_eq!(parser.push(0x03), None);
        assert_eq!(parser.push(0x04), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x05), None);
        assert_eq!(parser.push(0x06), None);
        assert_eq!(parser.push(0x07), None);
        assert_eq!(parser.push(0x08), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x00), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Input { .. })));

        if let Some(Packet::Input { p1, p2 }) = result {
            assert_eq!(p1, [0x01, 0x02, 0x03, 0x04]);
            assert_eq!(p2, [0x05, 0x06, 0x07, 0x08]);
        }
    }

    #[test]
    fn test_incomplete_packet() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b')'), None);
    }

    #[test]
    fn test_closing_bracket_without_opening() {
        let mut parser = Parser::new();
        assert_eq!(parser.push(b')'), None);
    }

    #[test]
    fn test_data_without_frame() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b'C'), None);
    }

    #[test]
    fn test_overflow_packet() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);

        for _ in 0..MAX_PACKET_SIZE {
            assert_eq!(parser.push(0xFF), None);
        }

        assert_eq!(parser.push(0xFF), None);
        assert!(!parser.in_frame);
        assert_eq!(parser.push(b')'), None);
    }

    #[test]
    fn test_multiple_packets_sequential() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b'C'), None);
        assert_eq!(parser.push(b'D'), None);

        let result1 = parser.push(b')');
        assert!(matches!(result1, Some(Packet::Data(_))));

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'X'), None);
        assert_eq!(parser.push(b'Y'), None);
        assert_eq!(parser.push(b'Z'), None);
        assert_eq!(parser.push(b'W'), None);

        let result2 = parser.push(b')');
        assert!(matches!(result2, Some(Packet::Data(_))));

        if let Some(Packet::Data(data)) = result2 {
            assert_eq!(data, [b'X', b'Y', b'Z', b'W']);
        }
    }

    #[test]
    fn test_reset_on_new_opening_bracket() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.idx, 0);

        assert_eq!(parser.push(b'C'), None);
        assert_eq!(parser.push(b'D'), None);
        assert_eq!(parser.push(b'E'), None);
        assert_eq!(parser.push(b'F'), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data(_))));

        if let Some(Packet::Data(data)) = result {
            assert_eq!(data, [b'C', b'D', b'E', b'F']);
        }
    }

    #[test]
    fn test_parser_default() {
        let parser1 = Parser::new();
        let parser2 = Parser::default();

        assert_eq!(parser1.idx, parser2.idx);
        assert_eq!(parser1.in_frame, parser2.in_frame);
    }

    #[test]
    fn test_all_zeros_data_packet() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x00), None);
        assert_eq!(parser.push(0x00), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data([0, 0, 0, 0]))));
    }

    #[test]
    fn test_all_ones_input_packet() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        for _ in 0..MAX_PACKET_SIZE {
            assert_eq!(parser.push(0xFF), None);
        }

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Input {
            p1: [0xFF, 0xFF, 0xFF, 0xFF],
            p2: [0xFF, 0xFF, 0xFF, 0xFF]
        })));
    }

    #[test]
    fn test_boundary_exact_max_packet_size() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);

        for i in 0..MAX_PACKET_SIZE {
            assert_eq!(parser.push(i as u8), None);
        }

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Input { .. })));

        if let Some(Packet::Input { p1, p2 }) = result {
            assert_eq!(p1, [0, 1, 2, 3]);
            assert_eq!(p2, [6, 7, 8, 9]);
        }
    }

    #[test]
    fn test_boundary_exact_data_packet_size() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);

        for i in 0..MAX_DATA_PACKET_SIZE {
            assert_eq!(parser.push(i as u8), None);
        }

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data([0, 1, 2, 3]))));
    }

    #[test]
    fn test_repeated_parsing_no_state_leak() {
        let mut parser = Parser::new();

        for round in 0u8..100 {
            if round == b'(' || round == b')'
                || round.wrapping_add(1) == b'(' || round.wrapping_add(1) == b')'
                || round.wrapping_add(2) == b'(' || round.wrapping_add(2) == b')'
                || round.wrapping_add(3) == b'(' || round.wrapping_add(3) == b')' {
                continue;
            }

            assert_eq!(parser.push(b'('), None);
            assert_eq!(parser.push(round), None);
            assert_eq!(parser.push(round.wrapping_add(1)), None);
            assert_eq!(parser.push(round.wrapping_add(2)), None);
            assert_eq!(parser.push(round.wrapping_add(3)), None);

            let result = parser.push(b')');
            assert!(
                matches!(result, Some(Packet::Data(_))),
                "Round {}: Expected Data packet, got {:?}",
                round,
                result
            );

            if let Some(Packet::Data(data)) = result {
                assert_eq!(data[0], round);
                assert_eq!(data[1], round.wrapping_add(1));
                assert_eq!(data[2], round.wrapping_add(2));
                assert_eq!(data[3], round.wrapping_add(3));
            }
        }
    }

    #[test]
    fn test_interleaved_frame_markers() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b'C'), None);
        assert_eq!(parser.push(b'D'), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data(_))));
    }

    #[test]
    fn test_alignment_different_byte_patterns() {
        let mut parser = Parser::new();

        let test_patterns = [
            [0x00, 0xFF, 0x00, 0xFF],
            [0xAA, 0x55, 0xAA, 0x55],
            [0x01, 0x23, 0x45, 0x67],
            [0xFE, 0xDC, 0xBA, 0x98],
        ];

        for pattern in &test_patterns {
            assert_eq!(parser.push(b'('), None);
            for &byte in pattern {
                assert_eq!(parser.push(byte), None);
            }
            let result = parser.push(b')');
            if let Some(Packet::Data(data)) = result {
                assert_eq!(&data, pattern);
            } else {
                panic!("Expected Data packet with pattern {:?}", pattern);
            }
        }
    }

    #[test]
    fn test_parser_state_after_overflow() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        for _ in 0..MAX_PACKET_SIZE + 5 {
            parser.push(0xAB);
        }

        assert!(!parser.in_frame);
        assert_eq!(parser.idx, 0);

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'X'), None);
        assert_eq!(parser.push(b'Y'), None);
        assert_eq!(parser.push(b'Z'), None);
        assert_eq!(parser.push(b'W'), None);

        let result = parser.push(b')');
        assert!(matches!(result, Some(Packet::Data([b'X', b'Y', b'Z', b'W']))));
    }

    #[test]
    fn test_p1_p2_boundary_separation() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);

        for i in 0..MAX_PACKET_SIZE {
            assert_eq!(parser.push(i as u8), None);
        }

        if let Some(Packet::Input { p1, p2 }) = parser.push(b')') {
            assert_eq!(p1, [0, 1, 2, 3]);
            assert_eq!(p2, [6, 7, 8, 9]);
            assert_ne!(p1, p2);
        } else {
            panic!("Expected Input packet");
        }
    }

    #[test]
    fn test_uninitialized_memory_not_leaked() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(0xDE), None);
        assert_eq!(parser.push(0xAD), None);
        assert_eq!(parser.push(0xBE), None);
        assert_eq!(parser.push(0xEF), None);

        if let Some(Packet::Data(data)) = parser.push(b')') {
            assert_eq!(data, [0xDE, 0xAD, 0xBE, 0xEF]);
            for &byte in &data {
                assert!(byte == 0xDE || byte == 0xAD || byte == 0xBE || byte == 0xEF);
            }
        }
    }

    #[test]
    fn test_extreme_rapid_packets() {
        let mut parser = Parser::new();
        let packet_data = b"ABCD";

        for _ in 0..1000 {
            assert_eq!(parser.push(b'('), None);
            for &byte in packet_data {
                assert_eq!(parser.push(byte), None);
            }
            if let Some(Packet::Data(data)) = parser.push(b')') {
                assert_eq!(&data, packet_data);
            } else {
                panic!("Expected Data packet");
            }
        }
    }

    #[test]
    fn test_protocol_validation_opening_bracket_inside_frame() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert!(parser.in_frame);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.idx, 2);

        assert_eq!(parser.push(b'('), None);
        assert!(parser.in_frame);
        assert_eq!(parser.idx, 0);

        assert_eq!(parser.push(b'X'), None);
        assert_eq!(parser.push(b'Y'), None);
        assert_eq!(parser.push(b'Z'), None);
        assert_eq!(parser.push(b'W'), None);

        let result = parser.push(b')');
        assert_eq!(result, Some(Packet::Data([b'X', b'Y', b'Z', b'W'])));
    }

    #[test]
    fn test_protocol_validation_closing_bracket_wrong_size() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'A'), None);
        assert_eq!(parser.push(b'B'), None);
        assert_eq!(parser.push(b'C'), None);

        let result = parser.push(b')');
        assert_eq!(result, None);
        assert!(!parser.in_frame);
    }

    #[test]
    fn test_protocol_validation_multiple_opening_brackets() {
        let mut parser = Parser::new();

        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.push(b'('), None);
        assert_eq!(parser.idx, 0);
        assert!(parser.in_frame);

        assert_eq!(parser.push(1), None);
        assert_eq!(parser.push(2), None);
        assert_eq!(parser.push(3), None);
        assert_eq!(parser.push(4), None);

        let result = parser.push(b')');
        assert_eq!(result, Some(Packet::Data([1, 2, 3, 4])));
    }
}
