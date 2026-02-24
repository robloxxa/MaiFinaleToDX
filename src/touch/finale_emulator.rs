use crate::error::Result;
use crate::port::MockPort;
use crate::touch::TouchState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::debug;

pub struct FinaleEmulator {
    send_input: bool,
    mock: MockPort,
    touch_state: Arc<TouchState>,
    exit_sig: Arc<AtomicBool>,
}

impl FinaleEmulator {
    pub fn new(mock: MockPort, touch_state: Arc<TouchState>, exit_sig: Arc<AtomicBool>) -> Self {
        Self {
            send_input: false,
            mock,
            touch_state,
            exit_sig,
        }
    }

    pub fn spawn_thread(self) -> Result<JoinHandle<Result<()>>> {
        let thread = thread::Builder::new()
            .name("Finale Emulator Thread".to_owned())
            .spawn(move || self.run())?;
        Ok(thread)
    }

    fn run(mut self) -> Result<()> {
        let mut in_command = false;
        let mut cmd_buf = Vec::with_capacity(6);

        while !self.exit_sig.load(Ordering::Acquire) {
            let data = self.mock.take_write_data();

            if data.is_empty() {
                if self.send_input {
                    self.stream_touch_input();
                }
                thread::sleep(Duration::from_millis(5));
                continue;
            }

            for &b in &data {
                match b {
                    b'{' => {
                        in_command = true;
                        cmd_buf.clear();
                    }
                    b'}' if in_command => {
                        in_command = false;
                        self.handle_command(&cmd_buf);
                    }
                    _ if in_command => {
                        cmd_buf.push(b);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, cmd: &[u8]) {
        debug!("Recieved command: {:?}", cmd);
        if cmd == b"HALT" {
            debug!("Finale emulator: received HALT");
            self.send_input = false;
            return;
        }

        if cmd == b"STAT" {
            debug!("Finale emulator: received STAT, starting touch input stream");
            self.send_input = true;
            return;
        }

        // Threshold commands: [panel, area, 't', 'h'] (get) or [panel, area, 'k', value] (set)
        if cmd.len() == 4 {
            if cmd[2] == b't' && cmd[3] == b'h' {
                debug!(
                    "Finale emulator: get threshold panel={} area={}",
                    cmd[0] as char, cmd[1] as char
                );
                self.mock
                    .push_read_data(&[b'(', cmd[0], cmd[1], b't', 0x60, b')']);
                return;
            }

            if cmd[2] == b'k' {
                debug!(
                    "Finale emulator: set threshold panel={} area={} value={}",
                    cmd[0] as char, cmd[1] as char, cmd[3]
                );
                self.mock
                    .push_read_data(&[b'(', cmd[0], cmd[1], cmd[2], cmd[3], b')']);
                return;
            }
        }

        debug!("Finale emulator: unknown command {:?}", cmd);
    }

    fn stream_touch_input(&self) {
        // Finale input packet: (P1[0] P1[1] P1[2] P1[3] 0x42 0x42 P2[0] P2[1] P2[2] P2[3] 0x42 0x42)
        let packet = [
            b'(', 0, 0, 0, 0, 0x42, 0x42, 0, 0, 0, 0, 0x42, 0x42,
            b')',
        ];

        self.mock.push_read_data(&packet);
        thread::sleep(Duration::from_millis(15));
    }
}
