use crate::config::{self, touch};
use crate::error::Result;
use crate::touch::packet::finale_slave::*;
use crate::touch::{HALT, STAT};
use crate::{helper_funcs::bit_read, touch::deluxe::Deluxe};
use anyhow::anyhow;
use arrayvec::ArrayVec;
use log::{debug, error, info};
use serial2::SerialPort;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{io, thread};

pub struct Finale {
    parser: Parser,
    buf: [u8; 14],
    p1_threshold: ThresholdInfo,
    p2_threshold: ThresholdInfo,

    p1_mapping: FinaleAreaMapping,
    p2_mapping: FinaleAreaMapping,

    pub port: SerialPort,
    pub dx_p1: Option<Deluxe>,
    pub dx_p2: Option<Deluxe>,
}

impl Finale {
    pub fn new(cfg: config::Touch, dx_p1: Option<Deluxe>, dx_p2: Option<Deluxe>) -> Result<Self> {
        Ok(Self {
            port: SerialPort::open(&cfg.finale_port, 9600)?,
            parser: Parser::new(),
            buf: [0u8; 14],

            p1_threshold: cfg.p1_threshold.into(),
            p2_threshold: cfg.p2_threshold.into(),

            p1_mapping: cfg.p1_dx_touch_mapping.into(),
            p2_mapping: cfg.p2_dx_touch_mapping.into(),

            dx_p1,
            dx_p2,
        })
    }

    pub fn try_init(&mut self, retry_count: i64) -> Result<()> {
        self.port.set_read_timeout(Duration::from_secs(0))?;
        self.port.set_write_timeout(Duration::from_secs(0))?;

        for c in 0..retry_count {
            info!("Trying to initialize Finale Touchscreen. Attempt {}", c + 1);
            match self.init() {
                Ok(()) => {
                    return Ok(());
                }
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

    fn init(&mut self) -> Result<()> {
        info!("Sending HALT packet");
        self.halt()?;

        info!("Initializing Threshold");
        self.init_threshold()?;

        info!("Sending STAT packet");
        self.stat()?;

        Ok(())
    }

    fn init_threshold(&mut self) -> Result<()> {
        let p1 = self.p1_threshold.clone();
        let p2 = self.p2_threshold.clone();

        info!("Initializing L");
        for (area, threshold) in p1.iter() {
            match self.get_threshold(b'L', *area) {
                Ok(_) => (),
                Err(e) => {
                    error!(
                        "Failed to get threshold from panel {:?} area {:?}: {}",
                        b'L', area, e
                    );
                    return Err(e.into());
                }
            }

            if let Packet::Data(_) = self.recieve_once()? {
                self.set_threshold(b'L', *area, *threshold)?;
            }

            // TODO: maybe show in console
            let _ = self.recieve_once()?;
        }

        info!("Initializing R");
        for (area, threshold) in p2.iter() {
            match self.get_threshold(b'R', *area) {
                Ok(_) => (),
                Err(e) => {
                    error!(
                        "Failed to get threshold from panel {:?} area {:?}: {}",
                        b'R', area, e
                    );
                    return Err(e.into());
                }
            }

            if let Packet::Data(_) = self.recieve_once()? {
                self.set_threshold(b'R', *area, *threshold)?;
            }

            // TODO: maybe show in console
            let _ = self.recieve_once()?;
        }

        Ok(())
    }

    fn get_threshold(&mut self, panel: u8, area: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b't', b'h', b'}'])
    }

    fn set_threshold(&self, panel: u8, area: u8, threshold: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b'k', threshold, b'}'])
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<()> {
        debug!(
            "Finale Touch: Sending {}",
            buf.iter().map(|&u| format!("{}", u)).collect::<String>()
        );

        self.port.write_all(buf)
    }

    pub fn receive(&mut self) -> Result<()> {
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
                    if let Some(dx_p1) = &mut self.dx_p1 {
                        if dx_p1.is_active() {
                            dx_p1.send(&self.p1_mapping.convert_to_dx_buf(p1))?;
                        }
                    }

                    if let Some(dx_p2) = &mut self.dx_p2 {
                        if dx_p2.is_active() {
                            dx_p2.send(&self.p2_mapping.convert_to_dx_buf(p2))?;
                        }
                    }
                }
                Some(Packet::Data(d)) => {
                    debug!("Received data packet: {:?}", d);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn recieve_once(&mut self) -> Result<Packet> {
        let mut buf = [0u8; 1];
        let mut attempt = 0;

        while attempt < 10 {
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

    // Sends HALT packet to touchscreen
    pub fn halt(&mut self) -> io::Result<()> {
        self.send(HALT)?;

        // Discard input buffer so there is no data if touch was working before
        self.port.discard_buffers()
    }

    // Sends STAT packet to touchscreen
    pub fn stat(&mut self) -> io::Result<()> {
        self.send(STAT)
    }

    pub fn spawn_thread(
        mut finale_touch: Finale,
        exit_sig: Arc<AtomicBool>,
    ) -> Result<JoinHandle<Result<()>>> {
        let thread = thread::Builder::new()
            .name("Finale Touch Thread".to_owned())
            .spawn(move || {
                while !exit_sig.load(Ordering::Relaxed) {
                    finale_touch.receive()?;
                }
                finale_touch.port.write_all(HALT)?;

                Ok(())
            })?;
        Ok(thread)
    }
}

impl Drop for Finale {
    fn drop(&mut self) {
        if let Err(e) = self.halt() {
            error!("Failed to send HALT: {e}")
        };
    }
}

static DEFAULT_DELUXE_WRITE_BUFFER: [u8; 9] = [b'(', 0, 0, 0, 0, 0, 0, 0, b')'];

#[derive(Debug)]
struct TouchArea {
    index: usize,
    bit_position: u8,

    last_activation: Option<Instant>,

    deactivate_after_ms: Duration,
    reactivate_after_ms: Duration,
}

impl TouchArea {
    fn is_active(&mut self, bit: u8, pos: usize) -> bool {
        if !bit_read(bit, pos) {
            self.last_activation = None;
            return false;
        }

        if !self.deactivate_after_ms.is_zero() {
            let elapsed = match self.last_activation {
                Some(duration) => duration.elapsed(),
                None => {
                    let instant = Instant::now();
                    self.last_activation = Some(instant);
                    instant.elapsed()
                }
            };

            if !self.reactivate_after_ms.is_zero() && elapsed > self.reactivate_after_ms {
                return true;
            }

            if elapsed > self.deactivate_after_ms {
                return false;
            }
        }

        true
    }
}

impl From<config::dx::Area> for TouchArea {
    fn from(area: config::dx::Area) -> Self {
        TouchArea {
            index: area.position,
            bit_position: area.bit as u8,
            last_activation: None,
            deactivate_after_ms: area.deactivate_after_ms,
            reactivate_after_ms: area.reactivate_after_ms,
        }
    }
}

type ThresholdInfo = BTreeMap<u8, u8>;

impl From<touch::Threshold> for ThresholdInfo {
    fn from(t: touch::Threshold) -> Self {
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

#[derive(Debug)]
struct FinaleAreaMapping {
    mapping: [[ArrayVec<TouchArea, 32>; 5]; 4],
}

impl FinaleAreaMapping {
    pub fn new() -> Self {
        FinaleAreaMapping {
            mapping: [
                [
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                ],
                [
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                ],
                [
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                ],
                [
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                    ArrayVec::new(),
                ],
            ],
        }
    }

    fn add_area(&mut self, area: config::dx::Area) {
        let areas = area.activate_on.0.clone();

        for finale_area in areas {
            self.mapping[finale_area.position][finale_area.bit as usize].push(area.clone().into());
        }
    }

    fn convert_to_dx_buf(&mut self, buf: [u8; 4]) -> [u8; 9] {
        let mut write_buffer = DEFAULT_DELUXE_WRITE_BUFFER;

        for (i, &bit) in buf.iter().enumerate() {
            for pos in 0..5usize {
                self.mapping[i][pos].iter_mut().for_each(|a| {
                    if !a.is_active(bit, pos) {
                        return;
                    }

                    write_buffer[a.index] |= a.bit_position;
                });
            }
        }

        write_buffer
    }
}

impl From<config::dx::AreaMapping> for FinaleAreaMapping {
    fn from(mapping: config::dx::AreaMapping) -> Self {
        let mut finale_mapping = FinaleAreaMapping::new();

        for area in mapping.into_values() {
            finale_mapping.add_area(area);
        }

        finale_mapping
    }
}
