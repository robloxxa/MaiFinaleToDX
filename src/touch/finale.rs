use crate::config::touch::finale::{FinaleTouch, Threshold};
use crate::config::touch::TouchMode;
use crate::error::{Error, Result};
use crate::port::{MockPort, Port, RealPort};
use crate::runtime::Module;
use crate::state::SharedState;
use crate::touch::packet::finale_slave::*;
use crate::touch::{AtomicTouchInput, TouchInput, HALT, STAT};
use anyhow::anyhow;
use tracing::{debug, error, info};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Finale {
    parser: Parser,
    buf: [u8; 14],
    p1_threshold: ThresholdInfo,
    p2_threshold: ThresholdInfo,
    retry_count: i64,
    exit_sig: Arc<AtomicBool>,

    input: Box<dyn TouchInput>,

    pub port: Box<dyn Port>,
    pub emu_handle: Option<JoinHandle<Result<()>>>,
}

impl Finale {
    pub fn new(exit_sig: Arc<AtomicBool>, cfg: FinaleTouch, input: Box<dyn TouchInput>, port: Box<dyn Port>) -> Result<Self> {
        Ok(Self {
            port,
            parser: Parser::new(),
            buf: [0u8; 14],
            retry_count: cfg.init_retry_count.unwrap_or(i64::MAX),
            exit_sig,

            p1_threshold: cfg.p1_threshold.into(),
            p2_threshold: cfg.p2_threshold.into(),

            input,
            emu_handle: None,
        })
    }

    fn init_threshold(&mut self) -> Result<()> {
        self.init_threshold_for_panel(b'L', &self.p1_threshold.clone())?;
        self.init_threshold_for_panel(b'R', &self.p2_threshold.clone())?;
        Ok(())
    }

    fn init_threshold_for_panel(&mut self, panel: u8, thresholds: &ThresholdInfo) -> Result<()> {
        info!("Initializing panel {}", panel as char);
        for (area, threshold) in thresholds.iter() {
            self.get_threshold(panel, *area).map_err(|e| {
                error!(
                    "Failed to get threshold from panel {} area {:?}: {}",
                    panel as char, area, e
                );
                e
            })?;

            if let Packet::Data(_) = self.receive_once()? {
                self.set_threshold(panel, *area, *threshold)?;
            }

            let _ = self.receive_once()?;
        }
        Ok(())
    }

    fn get_threshold(&mut self, panel: u8, area: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b't', b'h', b'}'])
    }

    fn set_threshold(&mut self, panel: u8, area: u8, threshold: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b'k', threshold, b'}'])
    }

    pub fn send(&mut self, buf: &[u8]) -> io::Result<()> {
        debug!(
            "Finale Touch: Sending {}",
            buf.iter().map(|&u| format!("{}", u)).collect::<String>()
        );

        self.port.write_all(buf)
    }

    pub fn receive_once(&mut self) -> Result<Packet> {
        let mut buf = [0u8; 1];
        let mut attempt = 0;

        const MAX_RECEIVE_ATTEMPTS: usize = 10;
        while attempt < MAX_RECEIVE_ATTEMPTS {
            match self.port.read_exact(&mut buf) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            let packet = self.parser.push(buf[0]);

            match packet {
                None => {
                    attempt += 1;
                    continue;
                }
                Some(p) => {
                    debug!("Received packet: {:?}", p);
                    return Ok(p);
                }
            }
        }

        Err(anyhow!("Could not read packet after 10 attempts").into())
    }

    pub fn halt(&mut self) -> io::Result<()> {
        self.send(HALT)?;
        self.port.discard_buffers()
    }

    pub fn stat(&mut self) -> io::Result<()> {
        self.send(STAT)
    }

}

impl Module for Finale {
    fn init(&mut self) -> Result<()> {
        self.port.set_read_timeout(Duration::from_millis(500))?;
        self.port.set_write_timeout(Duration::from_secs(0))?;

        for c in 0..self.retry_count {
            if self.exit_sig.load(Ordering::Acquire) {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled").into());
            }
            info!("Trying to initialize Finale Touchscreen. Attempt {}", c + 1);
            match (|| -> Result<()> {
                self.halt()?;
                self.init_threshold()?;
                self.stat()?;
                Ok(())
            })() {
                Ok(()) => {
                    info!("Finale Touchscreen is ready");
                    return Ok(())
                },
                Err(crate::error::Error::Io(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
                    error!("Init timeout")
                }
                Err(e) => {
                    error!("Init failed. Error: {}", e);
                }
            }
        }

        error!("Failed to connect to Finale Touchscreen");
        Err(io::Error::new(io::ErrorKind::TimedOut, "Finale touchscreen timeout").into())
    }

    fn poll(&mut self) -> Result<()> {
        let n = match self.port.read(&mut self.buf) {
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => return Ok(()),
            Err(e) => {
                error!("Failed to read from port: {}", e);
                return Ok(());
            }
        };

        let buf = &self.buf[..n];

        for &b in buf {
            match self.parser.push(b) {
                Some(Packet::Input { p1, p2 }) => {
                    self.input.touch_input(p1, p2)?;
                }
                Some(Packet::Data(d)) => {
                    debug!("Received data packet: {:?}", d);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

pub fn setup(
    config: FinaleTouch,
    exit_sig: Arc<AtomicBool>,
    shared_state: SharedState,
) -> Result<Box<dyn Module>> {
    if !config.enabled {
        return Err(Error::ModuleDisabled(crate::runtime::ModuleName::TouchFinale));
    }
    
    let input = AtomicTouchInput::new(shared_state.touch.clone());

    let (port, emu_handle): (Box<dyn Port>, Option<JoinHandle<Result<()>>>) = match config.mode {
        TouchMode::Emulated => {
            let mock = MockPort::new();
            let mock_clone = mock.try_clone()?;
            let emulator = super::finale_emulator::FinaleEmulator::new(
                mock,
                shared_state.touch.clone(),
                exit_sig.clone(),
            );
            let handle = emulator.spawn_thread()?;
            (mock_clone, Some(handle))
        }
        TouchMode::Hardware => {
            let port = Box::new(RealPort::open(&config.port, 9600)?);
            (port, None)
        }
    };

    let mut finale = Finale::new(exit_sig, config, Box::new(input), port)?;
    finale.emu_handle = emu_handle;
    Ok(Box::new(finale))
}

impl Drop for Finale {
    fn drop(&mut self) {
        if let Err(e) = self.halt() {
            error!("Failed to send HALT: {e}")
        };
        self.input.reset();
        if let Some(handle) = self.emu_handle.take() {
            let _ = handle.join();
        }
    }
}

type ThresholdInfo = BTreeMap<u8, u8>;

impl From<Threshold> for ThresholdInfo {
    fn from(t: Threshold) -> Self {
        let mut map = BTreeMap::new();

        map.insert(b'A', t.a1);
        map.insert(b'C', t.a2);
        map.insert(b'E', t.a3);
        map.insert(b'G', t.a4);
        map.insert(b'I', t.a5);
        map.insert(b'K', t.a6);
        map.insert(b'M', t.a7);
        map.insert(b'O', t.a8);

        map.insert(b'B', t.b1);
        map.insert(b'D', t.b2);
        map.insert(b'F', t.b3);
        map.insert(b'H', t.b4);
        map.insert(b'J', t.b5);
        map.insert(b'L', t.b6);
        map.insert(b'N', t.b7);
        map.insert(b'P', t.b8);

        map.insert(b'Q', t.c);

        map
    }
}
