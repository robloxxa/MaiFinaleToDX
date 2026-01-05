use crate::config::{self, touch};
use crate::error::Result;
use crate::touch::packet::finale_slave::*;
use crate::touch::{HALT, STAT};
use crate::{helper_funcs::bit_read, touch::deluxe::Deluxe};
use anyhow::anyhow;
use arrayvec::ArrayVec;
use log::{debug, error, info};
use serial2::SerialPort;
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
    pub fn new(
        cfg: config::Touch,
        dx_p1: Option<Deluxe>,
        dx_p2: Option<Deluxe>,
    ) -> Result<Self> {
        let mut port = SerialPort::open(&cfg.finale_port, 9600)?;
        
        port.set_read_timeout(Duration::from_millis(500))?;
        
        port.discard_buffers()?;

        Ok(Self {
            port,
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

    pub fn init(&mut self) -> Result<()> {
        const RETRY_COUNT: u8 = 5;

        self.port.set_read_timeout(Duration::from_secs(2))?;
        self.port.set_write_timeout(Duration::from_secs(2))?;

        for c in 0..RETRY_COUNT {
            info!("Trying to initialize Finale Touchscreen. Attempt {}", c + 1);
            match self.send_init() {
                Ok(()) => {
                    self.port.set_read_timeout(Duration::from_millis(0))?;
                    self.port.set_write_timeout(Duration::from_millis(0))?;
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

    fn send_init(&mut self) -> Result<()> {
        info!("Sending HALT packet");
        self.halt()?;

        self.init_threshold()?;

        info!("Sending STAT packet");
        self.stat()?;

        Ok(())
    }

    fn init_threshold(&mut self) -> Result<()> {
        for panel in [b'L', b'R'] {
            info!("Getting threshold from {} panel area", panel as char);
            for area in 0x41..=0x51 {
                match self.get_threshold(panel, area) {
                    Ok(_) => (),
                    Err(e) => {
                        error!(
                            "Failed to get threshold from panel {:?} area {:?}: {}",
                            panel, area, e
                        );
                        return Err(e.into());
                    }
                }

                if let Packet::Data(_) = self.recieve_once()? {
                    if panel == b'L' {
                        self.set_threshold(panel, area, self.p1_threshold[area as usize - 0x41])?;
                    } else {
                        self.set_threshold(panel, area, self.p2_threshold[area as usize - 0x41])?;
                    }
                }

                match self.get_threshold(panel, area) {
                    Ok(_) => (),
                    Err(e) => {
                        error!(
                            "Failed to get threshold from panel {:?} area {:?}: {}",
                            panel, area, e
                        );
                        return Err(e.into());
                    }
                }

                if let Packet::Data(d) = self.recieve_once()? {
                    if panel == b'L' {
                        self.set_threshold(panel, area, d[3])?;
                    } else {
                        self.set_threshold(panel, area, d[3])?;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_threshold(&mut self, panel: u8, area: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b't', b'h', b'}'])
    }

    fn set_threshold(&self, panel: u8, area: u8, threshold: u8) -> io::Result<()> {
        self.send(&[b'{', panel, area, b't', threshold, b'}'])
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<()> {
        debug!(
            "Finale Touch: Sending {}",
            buf.iter()
                .map(|&u| if u > 89 {
                    String::from(u as char)
                } else {
                    u.to_string()
                })
                .collect::<String>()
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
                Some(Packet::Data(packet)) => {
                    self.set_threshold(packet[0], packet[1], packet[3])?;
                }
                None => {}
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
            let _ = self.last_activation.take();
            return false
        }
        
        if self.deactivate_after_ms.is_zero() {
            let elapsed = match self.last_activation {
                Some(duration) => duration.elapsed(),
                None => {
                    self.last_activation = Some(Instant::now());
                    return true
                },
            };
            
            if !self.reactivate_after_ms.is_zero() && self.reactivate_after_ms > elapsed {
                self.last_activation = Some(Instant::now());
                return true
            }
            
            if self.deactivate_after_ms > elapsed {
                return false
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

type ThresholdInfo = [u8; 17];

impl From<touch::Threshold> for ThresholdInfo {
    fn from(t: touch::Threshold) -> Self {
        [
            t.a1, t.b1, t.a2, t.b2, t.a3, t.b3, t.a4, t.b4, t.a5, t.b5, t.a6, t.b6, t.a7, t.b7,
            t.a8, t.b8, t.c,
        ]
    }
}

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
        self.mapping[area.position][area.bit as usize].push(area.into());
    }

    fn convert_to_dx_buf(&mut self, buf: [u8; 4]) -> [u8; 9] {
        let mut write_buffer = DEFAULT_DELUXE_WRITE_BUFFER;
        
        for (i, &bit) in buf.iter().enumerate() {
            for pos in 0..5usize {
                self.mapping[i][pos]
                    .iter_mut()
                    .for_each(|a| {
                        if !a.is_active(bit, pos) {
                            return
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