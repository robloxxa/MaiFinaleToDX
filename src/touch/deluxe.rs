use arrayvec::ArrayVec;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::{error, warn};

use crate::config;
use crate::config::touch::dx::DxTouch;
use crate::config::touch::TouchMode;
use crate::error::{Error, Result};
use crate::exit_signal::ExitSignal;
use crate::helper_funcs::bit_read;
use crate::port::{MockPort, Port, RealPort};
use crate::runtime::Module;
use crate::state::{SharedState};
use crate::touch::packet::deluxe_master::{Packet, Parser};

const MAX_TIMEOUT_COUNT: u8 = 5;

static DEFAULT_DELUXE_WRITE_BUFFER: [u8; 9] = [b'(', 0, 0, 0, 0, 0, 0, 0, b')'];

pub struct Deluxe {
    num: u8,
    pub port: Box<dyn Port>,
    active: bool,
    touch_state: Arc<crate::touch::TouchState>,
    dx_raw: Arc<AtomicU64>,
    mapping: FinaleAreaMapping,
    timeout_count: u8,
    parser: Parser,
    buf: [u8; 12],
    emu_handle: Option<JoinHandle<Result<()>>>,
}

impl Deluxe {
    pub fn new(
        num: u8,
        mut port: Box<dyn Port>,
        touch_state: Arc<crate::touch::TouchState>,
        dx_raw: Arc<AtomicU64>,
        mapping: FinaleAreaMapping,
    ) -> Result<Self> {
        port.set_read_timeout(Duration::from_millis(1))?;

        port.discard_input_buffer()?;
        port.discard_output_buffer()?;

        Ok(Self {
            num,
            port,
            active: false,
            touch_state,
            dx_raw,
            mapping,
            timeout_count: 0,
            parser: Parser::new(),
            buf: [0u8; 12],
            emu_handle: None,
        })
    }

    fn poll_impl(&mut self) -> Result<()> {
        match self.port.read(&mut self.buf) {
            Ok(n) => {
                for i in 0..n {
                    if let Some(packet) = self.parser.push(self.buf[i]) {
                        self.handle_packet(packet)?;
                    }
                }
                Ok(())
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if self.active {
                    let raw = match self.num {
                        1 => self.touch_state.load_p1_finale(),
                        _ => self.touch_state.load_p2_finale(),
                    };
                    
                    let dx_buf = self.mapping.convert_to_dx_buf(raw);
                    let mut packed = [0u8; 8];
                    packed[..7].copy_from_slice(&dx_buf[1..8]);
                    
                    self.dx_raw.store(u64::from_le_bytes(packed), Ordering::Relaxed);
                    self.send(&dx_buf)?;
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to read from port: {}", e);
                Err(e.into())
            }
        }
    }
    
    fn handle_packet(&mut self, packet: Packet) -> Result<()> {
        match packet {
            Packet::Reset => {
                self.port.discard_input_buffer()?;
                self.port.discard_output_buffer()?;
                self.active = false;
            }
            Packet::Halt => {
                self.port.discard_input_buffer()?;
                self.port.discard_output_buffer()?;
                self.active = true;
            }
            Packet::Stat => {
                self.active = true;
            }
            Packet::Rotate(data) | Packet::Threshold(data) => {
                self.send(&[b'(', data[0], data[1], data[2], data[3], b')'])?;
            }
        }
        Ok(())
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
                    self.active = false;
                }

                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

}

impl Module for Deluxe {
    fn poll(&mut self) -> Result<()> {
        self.poll_impl()
    }
}

impl Drop for Deluxe {
    fn drop(&mut self) {
        if let Some(handle) = self.emu_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn setup(
    num: u8,
    config: DxTouch,
    exit_sig: ExitSignal,
    shared_state: SharedState,
) -> Result<Box<dyn Module>> {
    let (enabled, port_name, dx_raw, mapping) = match num {
        1 => (config.enabled, config.p1_port.clone(), shared_state.touch.p1_dx_raw.clone(), config.p1_mapping.clone()),
        2 => (config.enabled, config.p2_port.clone(), shared_state.touch.p2_dx_raw.clone(), config.p2_mapping.clone()),
        _ => return Err(anyhow::anyhow!("Invalid player number: {}", num).into()),
    };
    
    if !enabled {
        return Err(Error::ModuleDisabled(crate::runtime::ModuleName::TouchDeluxe(num)));
    }

    let (port, emu_handle): (Box<dyn Port>, Option<JoinHandle<Result<()>>>) = match config.mode {
        TouchMode::Emulated => {
            let mock = MockPort::new();
            let mock_clone = mock.try_clone()?;
            let emulator = super::deluxe_emulator::DeluxeEmulator::new(mock, exit_sig.clone());
            let handle = emulator.spawn_thread()?;
            (mock_clone, Some(handle))
        }
        TouchMode::Hardware => {
            let name = port_name.ok_or_else(|| {
                anyhow::anyhow!("No port configured for Deluxe P{} Touchscreen", num)
            })?;
            let port = RealPort::open(&name, 115_200).map_err(|e| {
                error!("Cannot open serial port for Deluxe P{} Touchscreen: {}", num, e);
                e
            })?;
            (Box::new(port), None)
        }
    };

    let mut deluxe = Deluxe::new(num, port, shared_state.touch.clone(), dx_raw, mapping.into())?;
    deluxe.emu_handle = emu_handle;
    Ok(Box::new(deluxe))
}

// DX touch conversion types

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
            bit_position: area.bit,
            last_activation: None,
            deactivate_after_ms: area.deactivate_after_ms,
            reactivate_after_ms: area.reactivate_after_ms,
        }
    }
}

#[derive(Debug)]
pub struct FinaleAreaMapping {
    mapping: [[ArrayVec<TouchArea, 32>; 5]; 4],
}

impl FinaleAreaMapping {
    pub fn new() -> Self {
        FinaleAreaMapping {
            mapping: Default::default(),
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
                for area in &mut self.mapping[i][pos] {
                    if area.is_active(bit, pos) {
                        write_buffer[area.index] |= area.bit_position;
                    }
                }
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
