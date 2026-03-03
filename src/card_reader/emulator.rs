use crate::card_reader::state::State;
use crate::error::Result;
use crate::exit_signal::ExitSignal;
use crate::port::MockPort;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::debug;

const SYNC: u8 = 0xE0;
const MARK: u8 = 0xD0;

// Request packet field offsets relative to the start of the unescaped
// post-SYNC data: [N, DEST, SEQ, CMD, DATA..., SUM]
const REQ_N: usize = 0;
const REQ_DEST: usize = 1;
const REQ_SEQ: usize = 2;
const REQ_CMD: usize = 3;
const REQ_DATA_BEGIN: usize = 4;

pub struct CardReaderEmulator {
    mock: MockPort,
    state: Arc<Mutex<State>>,
    exit_sig: ExitSignal,
}

impl CardReaderEmulator {
    pub fn new(mock: MockPort, state: Arc<Mutex<State>>, exit_sig: ExitSignal) -> Self {
        Self { mock, state, exit_sig }
    }

    pub fn spawn_thread(self) -> Result<JoinHandle<Result<()>>> {
        Ok(thread::Builder::new()
            .name("Card Reader Emulator Thread".to_owned())
            .spawn(move || self.run())?)
    }

    fn run(self) -> Result<()> {
        while !self.exit_sig.is_set() {
            let data = self.mock.take_write_data();
            if data.is_empty() {
                thread::sleep(Duration::from_millis(5));
                continue;
            }

            for resp in parse_and_respond(&data, &self.state) {
                self.mock.push_read_data(&resp);
            }
        }
        Ok(())
    }
}

/// Parse escaped request packets from `data` and return encoded response bytes.
fn parse_and_respond(data: &[u8], state: &Mutex<State>) -> Vec<Vec<u8>> {
    let mut responses = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] != SYNC {
            i += 1;
            continue;
        }

        // Read and unescape the post-SYNC bytes one-by-one until we have a
        // complete packet: [N, DEST, SEQ, CMD, DATA..., SUM] (N+1 bytes total).
        let mut unescaped: Vec<u8> = Vec::new();
        let mut j = i + 1;

        // Read until we have enough bytes to determine N, then read the rest.
        loop {
            if j >= data.len() {
                break;
            }
            let (b, adv) = read_escaped(data, j);
            j += adv;
            unescaped.push(b);

            // After reading the N byte, we know total post-SYNC count = N + 1.
            if unescaped.len() == 1 {
                // Just read N; continue reading N more bytes.
                let n = unescaped[REQ_N] as usize;
                if n < 4 {
                    // Minimum valid: DEST+SEQ+CMD+DATA(1)+SUM = 5 → N >= 5.
                    break;
                }
                // Read remaining N bytes (DEST, SEQ, CMD, DATA..., SUM).
                for _ in 0..n {
                    if j >= data.len() {
                        break;
                    }
                    let (b2, adv2) = read_escaped(data, j);
                    j += adv2;
                    unescaped.push(b2);
                }
                break;
            }
        }

        i = j;

        let n = match unescaped.first() {
            Some(&v) => v as usize,
            None => continue,
        };

        // unescaped must contain [N, DEST, SEQ, CMD, DATA..., SUM] = n+1 bytes.
        if unescaped.len() < n + 1 || n < 4 {
            continue;
        }

        // Verify checksum: sum of unescaped[0..n] == unescaped[n].
        let ck_calc: u8 = unescaped[..n].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        if ck_calc != unescaped[n] {
            debug!("Card reader emu: checksum mismatch, dropping packet");
            continue;
        }

        let dest = unescaped[REQ_DEST];
        let seq = unescaped[REQ_SEQ];
        let cmd = unescaped[REQ_CMD];
        let req_data = &unescaped[REQ_DATA_BEGIN..n];

        if let Some(resp) = handle_command(dest, seq, cmd, req_data, state) {
            responses.push(resp);
        }
    }

    responses
}

fn handle_command(
    dest: u8,
    seq: u8,
    cmd: u8,
    _req_data: &[u8],
    state: &Mutex<State>,
) -> Option<Vec<u8>> {
    match cmd {
        // RESET — acknowledge with empty payload.
        0x62 => {
            debug!("Card reader emu: RESET (dest={})", dest);
            Some(make_response(dest, seq, cmd, &[0x00]))
        }
        // GET_FIRMWARE_VERSION
        0x30 => {
            debug!("Card reader emu: GET_FIRMWARE (dest={})", dest);
            Some(make_response(dest, seq, cmd, b"837-15396  \xff\x10\x00\x00"))
        }
        // GET_HARDWARE_VERSION
        0x32 => {
            debug!("Card reader emu: GET_HARDWARE (dest={})", dest);
            Some(make_response(dest, seq, cmd, b"837-15396  \xff\x10\x00\x00"))
        }
        // RADIO_ON
        0x40 => {
            debug!("Card reader emu: RADIO_ON (dest={})", dest);
            Some(make_response(dest, seq, cmd, &[0x00]))
        }
        // RADIO_OFF
        0x41 => {
            debug!("Card reader emu: RADIO_OFF (dest={})", dest);
            Some(make_response(dest, seq, cmd, &[0x00]))
        }
        // POLL
        0x42 => {
            let pending = state.lock().unwrap().pending_card.take();
            match pending {
                Some(card_bytes) => {
                    debug!("Card reader emu: POLL — returning card (dest={})", dest);
                    let mut data = [0u8; 19];
                    // data[0] = status, data[1] = card type (Mifare), data[2] = padding
                    data[1] = 0x10;
                    // data[3..=10] = 8-byte card ID (matches handle_card extraction)
                    data[3..=10].copy_from_slice(&card_bytes);
                    Some(make_response(dest, seq, cmd, &data))
                }
                None => {
                    // No card present — short response (len != 19 signals no card).
                    Some(make_response(dest, seq, cmd, &[0x00]))
                }
            }
        }
        _ => {
            debug!("Card reader emu: unknown cmd {:#04x} (dest={})", cmd, dest);
            None
        }
    }
}

/// Build an escaped response packet ready to push into the mock read buffer.
///
/// Response layout (unescaped):
///   [SYNC] [N] [DEST] [SEQ] [STATUS=0x01] [CMD] [REPORT=0x01] [DATA...] [SUM]
/// where N = data.len() + 6.
fn make_response(dest: u8, seq: u8, cmd: u8, data: &[u8]) -> Vec<u8> {
    let n = (data.len() + 6) as u8;

    // Build unescaped payload (bytes after SYNC, up to and including SUM).
    let mut payload: Vec<u8> = Vec::with_capacity(data.len() + 7);
    payload.push(n);
    payload.push(dest);
    payload.push(seq);
    payload.push(0x01); // STATUS = OK
    payload.push(cmd);
    payload.push(0x01); // REPORT = Normal
    payload.extend_from_slice(data);

    // Checksum over all payload bytes except SUM itself.
    let ck: u8 = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    // Write SYNC raw, then escape the rest.
    let mut result = Vec::with_capacity(payload.len() + 4);
    result.push(SYNC);
    for &b in &payload {
        escape_into(&mut result, b);
    }
    escape_into(&mut result, ck);

    result
}

#[inline]
fn escape_into(buf: &mut Vec<u8>, b: u8) {
    if b == SYNC || b == MARK {
        buf.push(MARK);
        buf.push(b.wrapping_sub(1));
    } else {
        buf.push(b);
    }
}

/// Read one possibly-escaped byte from `data` at position `i`.
/// Returns `(value, bytes_consumed)`.
#[inline]
fn read_escaped(data: &[u8], i: usize) -> (u8, usize) {
    if data[i] == MARK {
        if i + 1 < data.len() {
            (data[i + 1].wrapping_add(1), 2)
        } else {
            (data[i], 1)
        }
    } else {
        (data[i], 1)
    }
}
