// Modified JVS response packet layout (after unescaping):
// [0] SYNC=0xE0  [1] N  [2] DEST  [3] SEQ  [4] STATUS  [5] CMD  [6] REPORT  [7..N+1] DATA  [N+1] SUM
// len_of_packet = N + 2   (SIZE_INDEX=1, DATA_BEGIN_INDEX=7)
// Escaping: MARK_BYTE (0xD0) prefix means next byte + 1 is the real value.

const SYNC_BYTE: u8 = 0xE0;
const MARK_BYTE: u8 = 0xD0;

const DATA_BEGIN: usize = 7;
const MIN_PACKET_LEN: usize = DATA_BEGIN + 1 + 1; // at least 1 data byte + SUM

pub enum ParseResult {
    Valid { data: [u8; 128], len: usize },
    Invalid,
}

pub struct ResponseParser {
    buf: [u8; 256],
    idx: usize,
    mark: bool,
}

impl ResponseParser {
    pub fn new() -> Self {
        Self {
            buf: [0; 256],
            idx: 0,
            mark: false,
        }
    }

    pub fn push(&mut self, raw: u8) -> Option<ParseResult> {
        if self.idx == 0 {
            if raw == SYNC_BYTE {
                self.buf[0] = SYNC_BYTE;
                self.idx = 1;
                self.mark = false;
            }
            return None;
        }

        let b = if self.mark {
            self.mark = false;
            raw.wrapping_add(1)
        } else if raw == MARK_BYTE {
            self.mark = true;
            return None;
        } else {
            raw
        };

        if self.idx >= self.buf.len() {
            self.reset();
            return None;
        }

        self.buf[self.idx] = b;
        self.idx += 1;

        if self.idx < 2 {
            return None;
        }

        let n = self.buf[1] as usize;
        let expected_len = n + 2;

        if self.idx < expected_len || expected_len < MIN_PACKET_LEN {
            return None;
        }

        let calculated: u8 = self.buf[1..expected_len - 1]
            .iter()
            .fold(0u8, |acc, &x| acc.wrapping_add(x));
        let received = self.buf[expected_len - 1];

        // Copy data out before resetting so the result owns it.
        let data_end = expected_len - 1;
        let data_len = data_end.saturating_sub(DATA_BEGIN);
        let mut data = [0u8; 128];
        let copy_len = data_len.min(data.len());
        data[..copy_len].copy_from_slice(&self.buf[DATA_BEGIN..DATA_BEGIN + copy_len]);

        self.reset();

        if calculated == received {
            Some(ParseResult::Valid { data, len: copy_len })
        } else {
            Some(ParseResult::Invalid)
        }
    }

    fn reset(&mut self) {
        self.idx = 0;
        self.mark = false;
    }
}
