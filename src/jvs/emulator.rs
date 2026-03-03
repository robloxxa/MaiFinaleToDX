use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::debug;

use crate::error::Result;
use crate::exit_signal::ExitSignal;
use crate::jvs::State;
use crate::port::MockPort;

pub struct JvsEmulator {
    mock: MockPort,
    jvs_state: Arc<State>,
    exit_sig: ExitSignal,
    board: u8,
}

impl JvsEmulator {
    pub fn new(mock: MockPort, jvs_state: Arc<State>, exit_sig: ExitSignal, board: u8) -> Self {
        Self { mock, jvs_state, exit_sig, board }
    }

    pub fn spawn_thread(self) -> Result<JoinHandle<Result<()>>> {
        let thread = thread::Builder::new()
            .name("JVS Emulator Thread".to_owned())
            .spawn(move || self.run())?;
        Ok(thread)
    }

    fn run(self) -> Result<()> {
        while !self.exit_sig.is_set() {
            let data = self.mock.take_write_data();
            if data.is_empty() {
                thread::sleep(Duration::from_millis(5));
                continue;
            }

            let responses = parse_and_respond(&data, self.board, &self.jvs_state);
            for resp in responses {
                self.mock.push_read_data(&resp);
            }
        }
        Ok(())
    }
}

fn parse_and_respond(data: &[u8], board: u8, jvs_state: &State) -> Vec<Vec<u8>> {
    let mut responses = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Find sync byte 0xE0
        if data[i] != 0xE0 {
            i += 1;
            continue;
        }
        // Need at least: sync(1) + src(1) + len(1) = 3 bytes minimum
        if i + 2 >= data.len() {
            break;
        }
        let _src = data[i + 1];
        let length = data[i + 2] as usize;
        // payload is length-1 bytes (length includes checksum byte)
        if length == 0 || i + 2 + length > data.len() {
            i += 1;
            continue;
        }
        let payload = &data[i + 3..i + 2 + length];

        if let Some(resp) = handle_command(payload, board, jvs_state) {
            responses.push(resp);
        }

        i += 3 + length;
    }

    responses
}

fn handle_command(payload: &[u8], board: u8, jvs_state: &State) -> Option<Vec<u8>> {
    if payload.is_empty() {
        return None;
    }
    let cmd = payload[0];

    match cmd {
        // RESET is broadcast, no response
        0xF0 => {
            debug!("JVS emu: RESET");
            None
        }
        0xF1 => {
            debug!("JVS emu: ASSIGN_ADDRESS");
            Some(make_response(board, &[]))
        }
        0x10 => {
            debug!("JVS emu: IDENTIFY");
            Some(make_response(board, b"MFIIDX,EMULATED;837-13551 ;Ver1.00;2000/01/01"))
        }
        0x11 => {
            debug!("JVS emu: COMMAND_REVISION");
            Some(make_response(board, &[0x13]))
        }
        0x12 => {
            debug!("JVS emu: JVS_VERSION");
            Some(make_response(board, &[0x30]))
        }
        0x13 => {
            debug!("JVS emu: COMMS_VERSION");
            Some(make_response(board, &[0x10]))
        }
        0x14 => {
            debug!("JVS emu: CAPABILITIES");
            // 2 players, 16 buttons each
            Some(make_response(board, &[0x01, 0x02, 0x10, 0x00, 0x00]))
        }
        0x20 => {
            let btns = jvs_state.load_buttons();
            let switch_data = build_switch_data(&btns);
            Some(make_response(board, &switch_data))
        }
        _ => None,
    }
}

fn make_response(board: u8, data: &[u8]) -> Vec<u8> {
    let length = (data.len() + 2) as u8;
    let mut pkt = vec![0xE0u8, board, length, 0x01u8];
    pkt.extend_from_slice(data);
    let ck: u8 = pkt[1..].iter().copied().fold(0u8, u8::wrapping_add);
    pkt.push(ck);
    pkt
}

fn build_switch_data(btns: &crate::jvs::state::JvsButtons) -> [u8; 5] {
    let mut data = [0u8; 5];

    // data[0]: system switches — service bit 7 (active-high)
    if btns.service {
        data[0] = 0x80;
    }

    // data[1]: P1 high byte — test bit 6 (active-high); btn1/2/3 at bits 2/3/0 (active-low)
    data[1] = 0x0D; // bits 0,2,3 high = not pressed
    if btns.test {
        data[1] |= 1 << 6;
    }
    if btns.p1[0] { data[1] &= !(1 << 2); }
    if btns.p1[1] { data[1] &= !(1 << 3); }
    if btns.p1[2] { data[1] &= !(1 << 0); }

    // data[2]: P1 low byte — btn4-8 at bits 7-3 (active-low)
    data[2] = 0xF8;
    if btns.p1[3] { data[2] &= !(1 << 7); }
    if btns.p1[4] { data[2] &= !(1 << 6); }
    if btns.p1[5] { data[2] &= !(1 << 5); }
    if btns.p1[6] { data[2] &= !(1 << 4); }
    if btns.p1[7] { data[2] &= !(1 << 3); }

    // data[3]: P2 high byte — same layout as data[1]
    data[3] = 0x0D;
    if btns.p2[0] { data[3] &= !(1 << 2); }
    if btns.p2[1] { data[3] &= !(1 << 3); }
    if btns.p2[2] { data[3] &= !(1 << 0); }

    // data[4]: P2 low byte — same layout as data[2]
    data[4] = 0xF8;
    if btns.p2[3] { data[4] &= !(1 << 7); }
    if btns.p2[4] { data[4] &= !(1 << 6); }
    if btns.p2[5] { data[4] &= !(1 << 5); }
    if btns.p2[6] { data[4] &= !(1 << 4); }
    if btns.p2[7] { data[4] &= !(1 << 3); }

    data
}
