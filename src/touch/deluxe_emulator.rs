use crate::error::Result;
use crate::exit_signal::ExitSignal;
use crate::port::MockPort;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::debug;

pub struct DeluxeEmulator {
    mock: MockPort,
    exit_sig: ExitSignal,
}

impl DeluxeEmulator {
    pub fn new(mock: MockPort, exit_sig: ExitSignal) -> Self {
        Self { mock, exit_sig }
    }

    pub fn spawn_thread(self) -> Result<JoinHandle<Result<()>>> {
        let thread = thread::Builder::new()
            .name("Deluxe Emulator Thread".to_owned())
            .spawn(move || self.run())?;
        Ok(thread)
    }

    fn run(self) -> Result<()> {
        // Send HALT command: (xxLx) where byte[2] == 'L'
        debug!("Deluxe emulator: sending HALT");
        self.mock.push_read_data(&[b'(', 0x00, 0x00, b'L', 0x00, b')']);

        thread::sleep(Duration::from_millis(50));

        // Send STAT command: (xxAx) where byte[2] == 'A'
        debug!("Deluxe emulator: sending STAT");
        self.mock.push_read_data(&[b'(', 0x00, 0x00, b'A', 0x00, b')']);

        // Drain write buffer periodically to prevent unbounded growth
        while !self.exit_sig.is_set() {
            let _ = self.mock.take_write_data();
            thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }
}
