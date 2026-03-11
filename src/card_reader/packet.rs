// Modified JVS response packet layout (after unescaping):
// [0] SYNC=0xE0  [1] N  [2] DEST  [3] SEQ  [4] STATUS  [5] CMD  [6] REPORT  [7..N] DATA  [N+1] SUM
// Escaping: MARK_BYTE (0xD0) prefix means next byte + 1 is the real value.

const SYNC_BYTE: u8 = 0xE0;
const MARK_BYTE: u8 = 0xD0;

const SYNC_IDX: u8 = 0;
const N_IDX: u8 = 1;
const DEST_IDX: u8 = 2;
const SEQ_IDX: u8 = 3;
const STATUS_IDX: u8 = 4;
const CMD_IDX: u8 = 5;
const REPORT_IDX: u8 = 6;
const DATA_START_IDX: u8 = 7;
const DATA_MAX_IDX: u8 = 255;

pub enum Packet<'a> {
    Valid { data: &'a [u8] },
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
    pub fn push<'a>(&'a mut self, raw: u8) -> Option<Packet<'a>> {
        let b = if self.mark {
            self.mark = false;
            raw.wrapping_add(1)
        } else if raw == MARK_BYTE {
            self.mark = true;
            return None;
        } else {
            raw
        };

        match self.idx {
            SYNC_IDX => {
                if b == SYNC_BYTE {
                    self.buf[0] = b;
                    self.idx = 1;
                    self.sum = 0;
                }
                None
            }
            N_IDX => {
                if b < 6 {
                    self.reset();
                    return None;
                }
                self.buf[1] = b;
                self.sum = self.sum.wrapping_add(b);
                self.idx = 2;
                None
            }
            DEST_IDX | SEQ_IDX | STATUS_IDX | CMD_IDX | REPORT_IDX => {
                self.buf[self.idx as usize] = b;
                self.sum = self.sum.wrapping_add(b);
                self.idx += 1;
                None
            }
            DATA_START_IDX..=DATA_MAX_IDX => {
                let checksum_idx = self.buf[N_IDX as usize] + 1;

                self.buf[self.idx as usize] = b;

                if self.idx == checksum_idx {
                    let calculated = self.sum;
                    self.reset();

                    if calculated == b {
                        Some(Packet::Valid {
                            data: &self.buf[DATA_START_IDX as usize..checksum_idx as usize],
                        })
                    } else {
                        Some(Packet::Invalid)
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
        self.idx = 0;
        self.mark = false;
        self.sum = 0;
    }
}

pub struct Builder {
    buf: [u8; 512],
    len: usize,
    seq: u8,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            buf: [0; 512],
            len: 0,
            seq: 0,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Build a request packet: [SYNC] [N] [DEST] [SEQ] [CMD] [DATA...] [SUM]
    /// N = data.len() + 4 (DEST + SEQ + CMD + SUM)
    /// SUM = wrapping sum of N through last DATA byte
    /// SEQ auto-increments mod 32 on each call.
    pub fn build(&mut self, dest: u8, cmd: u8, data: &[u8]) -> &[u8] {
        self.len = 0;

        self.buf[self.len] = SYNC_BYTE;
        self.len += 1;

        let n = (data.len() + 4) as u8;
        let seq = self.seq;
        self.seq = (self.seq + 1) % 32;

        let mut sum: u8 = 0;

        self.push_byte(n);
        sum = sum.wrapping_add(n);

        self.push_byte(dest);
        sum = sum.wrapping_add(dest);

        self.push_byte(seq);
        sum = sum.wrapping_add(seq);

        self.push_byte(cmd);
        sum = sum.wrapping_add(cmd);

        for &b in data {
            self.push_byte(b);
            sum = sum.wrapping_add(b);
        }

        self.push_byte(sum);

        &self.buf[..self.len]
    }

    fn push_byte(&mut self, b: u8) {
        if b == SYNC_BYTE || b == MARK_BYTE {
            self.buf[self.len] = MARK_BYTE;
            self.len += 1;
            self.buf[self.len] = b.wrapping_sub(1);
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

    /// Helper: build an unescaped response frame and return escaped wire bytes.
    fn make_response(dest: u8, seq: u8, cmd: u8, data: &[u8]) -> Vec<u8> {
        let n = (data.len() + 6) as u8;
        let mut payload = Vec::new();
        payload.push(n);
        payload.push(dest);
        payload.push(seq);
        payload.push(0x01); // STATUS
        payload.push(cmd);
        payload.push(0x01); // REPORT
        payload.extend_from_slice(data);

        let ck: u8 = payload.iter().fold(0u8, |a, &b| a.wrapping_add(b));

        let mut wire = vec![SYNC_BYTE];
        for &b in &payload {
            escape(&mut wire, b);
        }
        escape(&mut wire, ck);
        wire
    }

    fn escape(buf: &mut Vec<u8>, b: u8) {
        if b == SYNC_BYTE || b == MARK_BYTE {
            buf.push(MARK_BYTE);
            buf.push(b.wrapping_sub(1));
        } else {
            buf.push(b);
        }
    }

    #[test]
    fn valid_response_parse() {
        let wire = make_response(0x01, 0x00, 0x30, &[0xAA, 0xBB]);
        let mut parser = Parser::new();
        for &b in &wire {
            if let Some(result) = parser.push(b) {
                match result {
                    Packet::Valid { data } => {
                        assert_eq!(data, &[0xAA, 0xBB]);
                        return;
                    }
                    _ => panic!("Expected Valid packet"),
                }
            }
        }
        panic!("No packet parsed");
    }

    #[test]
    fn invalid_checksum() {
        let mut wire = make_response(0x01, 0x00, 0x30, &[0xAA]);
        // Corrupt last byte (checksum)
        let last = wire.len() - 1;
        wire[last] ^= 0xFF;

        let mut parser = Parser::new();
        for &b in &wire {
            if let Some(result) = parser.push(b) {
                assert!(matches!(result, Packet::Invalid));
                return;
            }
        }
        panic!("No packet parsed");
    }

    #[test]
    fn escaped_bytes_in_response() {
        // Use data that contains values requiring escaping (0xE0, 0xD0).
        let wire = make_response(0x01, 0x00, 0x42, &[0xE0, 0xD0, 0x55]);
        let mut parser = Parser::new();
        for &b in &wire {
            if let Some(result) = parser.push(b) {
                match result {
                    Packet::Valid { data } => {
                        assert_eq!(data, &[0xE0, 0xD0, 0x55]);
                        return;
                    }
                    _ => panic!("Expected Valid packet with escaped data"),
                }
            }
        }
        panic!("No packet parsed");
    }

    #[test]
    fn builder_output() {
        let mut builder = Builder::new();
        let out = builder.build(0x01, 0x42, &[0x00]);

        // Unescape and verify structure.
        let mut unescaped = Vec::new();
        let mut i = 1; // skip SYNC
        let raw = out;
        while i < raw.len() {
            if raw[i] == MARK_BYTE && i + 1 < raw.len() {
                unescaped.push(raw[i + 1].wrapping_add(1));
                i += 2;
            } else {
                unescaped.push(raw[i]);
                i += 1;
            }
        }

        // [N, DEST, SEQ, CMD, DATA..., SUM]
        assert_eq!(raw[0], SYNC_BYTE);
        let n = unescaped[0] as usize;
        assert_eq!(n, 5); // data.len()==1 + 4
        assert_eq!(unescaped[1], 0x01); // DEST
        assert_eq!(unescaped[2], 0x00); // SEQ (first call)
        assert_eq!(unescaped[3], 0x42); // CMD
        assert_eq!(unescaped[4], 0x00); // DATA

        let ck: u8 = unescaped[..n].iter().fold(0u8, |a, &b| a.wrapping_add(b));
        assert_eq!(unescaped[n], ck);
    }

    #[test]
    fn seq_increment() {
        let mut builder = Builder::new();
        let unescape_seq = |builder: &mut Builder| -> u8 {
            let out = builder.build(0x01, 0x30, &[0x00]);
            // SEQ is the 3rd unescaped byte after SYNC (index 2 in unescaped).
            let mut unescaped = Vec::new();
            let mut i = 1;
            while i < out.len() {
                if out[i] == MARK_BYTE && i + 1 < out.len() {
                    unescaped.push(out[i + 1].wrapping_add(1));
                    i += 2;
                } else {
                    unescaped.push(out[i]);
                    i += 1;
                }
            }
            unescaped[2]
        };

        assert_eq!(unescape_seq(&mut builder), 0);
        assert_eq!(unescape_seq(&mut builder), 1);
        assert_eq!(unescape_seq(&mut builder), 2);
    }

    #[test]
    fn seq_wraps_at_32() {
        let mut builder = Builder::new();
        for _ in 0..32 {
            builder.build(0x01, 0x30, &[0x00]);
        }
        // Next call should wrap to 0.
        let out = builder.build(0x01, 0x30, &[0x00]);
        let mut unescaped = Vec::new();
        let mut i = 1;
        while i < out.len() {
            if out[i] == MARK_BYTE && i + 1 < out.len() {
                unescaped.push(out[i + 1].wrapping_add(1));
                i += 2;
            } else {
                unescaped.push(out[i]);
                i += 1;
            }
        }
        assert_eq!(unescaped[2], 0);
    }

    #[test]
    fn builder_escaping() {
        let mut builder = Builder::new();
        // Use dest=0xE0 which requires escaping.
        let out = builder.build(0xE0, 0x30, &[0xD0]);

        // Verify SYNC is raw, then escaped bytes follow.
        assert_eq!(out[0], SYNC_BYTE);

        // The dest byte 0xE0 must appear as [0xD0, 0xDF].
        // The data byte 0xD0 must appear as [0xD0, 0xCF].
        assert!(out.windows(2).any(|w| w == [MARK_BYTE, 0xDF]));
        assert!(out.windows(2).any(|w| w == [MARK_BYTE, 0xCF]));
    }
}
