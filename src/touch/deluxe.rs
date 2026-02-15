use anyhow::Context;
use std::io::Read;

use crate::error::Result;
use crate::port::{Port, RealPort};
use log::{error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
const MAX_TIMEOUT_COUNT: u8 = 5;

pub struct Deluxe {
    num: u8,
    pub port: Box<dyn Port>,
    pub active: Arc<AtomicBool>,

    timeout_count: u8,
}

impl Deluxe {
    pub fn new(port_name: impl Into<String>, num: u8) -> Result<Self> {
        let port_name = port_name.into();
        let mut port = RealPort::open(&port_name, 115_200).map_err(|e| {
            error!(
                "Cannot open serial port for Deluxe P{} Touchscreen: {}",
                num, e
            );
            e
        })?;

        port.set_read_timeout(Duration::from_millis(500))?;

        port.discard_input_buffer()?;
        port.discard_output_buffer()?;

        Ok(Self {
            num,
            port: Box::new(port),
            active: Arc::new(AtomicBool::new(false)),
            timeout_count: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut read_buffer: [u8; 6] = [0; 6];
        match self.port.read_exact(&mut read_buffer) {
            Ok(_) => {
                match read_buffer[3] {
                    b'E' => {
                        self.port.discard_input_buffer()?;
                        self.port.discard_output_buffer()?;

                        self.active.store(false, Ordering::Release)
                    }
                    b'L' => {
                        self.port.discard_input_buffer()?;
                        self.port.discard_output_buffer()?;

                        self.active.store(false, Ordering::Release);
                    }
                    b'A' => {
                        self.active.store(true, Ordering::Release);
                    }
                    b'k' | b'r' => {
                        read_buffer[0] = b'(';
                        read_buffer[5] = b')';
                        self.send(&read_buffer)?;
                    }
                    _ => {
                        warn!("Unknown command: {:?}", &read_buffer);
                    }
                }
                Ok(())
            }
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub(crate) fn send(&mut self, buf: &[u8]) -> Result<()> {
        match self.port.write_all(buf) {
            Ok(()) => Ok(()),
            Err(ref err) if err.kind() == std::io::ErrorKind::TimedOut => {
                warn!(
                    "Write to Deluxe P{} timed out. This is probably due to MaiMai being closed",
                    self.num
                );

                self.timeout_count += 1;

                if self.timeout_count > MAX_TIMEOUT_COUNT {
                    warn!(
                        "Too much timeouts for Deluxe P{}, please restart your game",
                        self.num
                    );

                    self.timeout_count = 0;
                    self.active.store(false, Ordering::Release);
                }

                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            num: self.num,
            port: self.port.try_clone()?,
            active: self.active.clone(),
            timeout_count: 0,
        })
    }

    pub fn spawn_thread(
        mut deluxe_touch: Deluxe,
        exit_sig: Arc<AtomicBool>,
    ) -> Result<JoinHandle<Result<()>>> {
        let num = deluxe_touch.num;

        let thread = thread::Builder::new()
            .name(format!("Deluxe P{} Touch Thread", num))
            .spawn(move || {
                while !exit_sig.load(Ordering::Acquire) {
                    deluxe_touch.process()?;
                }

                Ok(())
            })
            .with_context(|| format!("Spawning Deluxe P{} Touch Thread failed", num))?;

        Ok(thread)
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
}
