use log::{debug, error, info};

use crate::helper_funcs::{bit_read, ReadExt};
use crate::touch::deluxe::Deluxe;
use crate::touch::{HALT, STAT};
use serial2::SerialPort;
use std::io::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

pub const TOUCH_MAX_SIZE: usize = 14;
pub const TOUCH_SETTINGS_MAX_SIZE: usize = 6;

pub struct FrameParser<const MAX_SIZE: usize = TOUCH_MAX_SIZE> {
    inner: [u8; 14],
    count: usize,
}

pub struct Finale {
    pub port: SerialPort,

    byte_count: usize,
    buf: [u8; 14],
    pub dx_p1: Option<Deluxe>,
    pub dx_p2: Option<Deluxe>,
}

impl Finale {
    pub fn new(
        port_name: impl Into<String>,
        dx_p1: Option<Deluxe>,
        dx_p2: Option<Deluxe>,
    ) -> Result<Self> {
        let port_name = port_name.into();
        let mut port = SerialPort::open(&port_name, 9600)?;
        port.set_read_timeout(Duration::from_millis(0))?;

        Ok(Self {
            port,
            byte_count: 0,
            buf: [0u8; 14],
            dx_p1,
            dx_p2,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        match self.receive()? {
            Some(len) if len == TOUCH_MAX_SIZE => {
                    if let Some(p1) = &self.dx_p1 {
                        p1.send(&self.buf[1..5])?;
                    }
                    
                    if let Some(p2) = &self.dx_p2 {
                        p2.send(&self.buf[7..11])?;
                    }
                    
                    Ok(())
            }
            Some(len) if len == TOUCH_SETTINGS_MAX_SIZE => {
                info!(
                    "Reading {:?}",
                    self.buf[0..=TOUCH_SETTINGS_MAX_SIZE]
                        .iter()
                        .map(|&u| u as char)
                        .collect::<Vec<char>>()
                );
                
                Ok(())
            }
            Some(_) => {
                info!(
                    "Reading {:?}",
                    self.buf
                        .iter()
                        .map(|&u| u as char)
                        .collect::<Vec<char>>()
                );
                Ok(())
            }
            None => Ok(()),
        }

        // match self.reader.read_u8() {
        //     Ok(b) => {
        //         self.byte_count += 1;
        //         match b {
        //             b'(' => {
        //                 self.byte_count = 0;
        //                 self.read_buffer[0] = b
        //             }
        //             b')' => {
        //                 self.read_buffer[self.byte_count] = b;
        //                 if self.byte_count > 5 {
        //                     self.dx_p1.as_ref().map_or_else(
        //                         || Ok(()),
        //                         |dx| dx.send(&convert_to_dx_buf(&self.read_buffer[1..5])),
        //                     )?;
        //                     self.dx_p2.as_ref().map_or_else(
        //                         || Ok(()),
        //                         |dx| dx.send(&convert_to_dx_buf(&self.read_buffer[1..5])),
        //                     )?;
        //                 } else {}
        //                 // info!("Reading {:?}", self.read_buffer[..=self.byte_count].iter().map(|&u| u as char).collect::<Vec<char>>());
        //             }
        //             _ => {
        //                 self.read_buffer[self.byte_count] = b;
        //             }
        //         };
        //         Ok(())
        //     }
        //     Err(ref err) if err.kind() == io::ErrorKind::TimedOut => Ok(()),
        //     Err(err) => Err(err.into()),
        // }
        //     match self.port.read_exact(&mut self.buf[0..=self.byte_count]) {
        //         Ok(_) => {
        //             info!(
        //                 "Reading {:?}",
        //                 self.buf[0..=6]
        //                     .iter()
        //                     .map(|&u| u as char)
        //                     .collect::<Vec<char>>()
        //             );
        //             // TODO: Check how well behave relaxed ordering
        //             // Also maybe with serial2 we can read it without any delay? Since it uses different timeout settings.
        //             // if self.deluxe_active[0].load(Ordering::Relaxed) {
        //             //     Self::write_to_deluxe(&mut self.read_buffer[1..5], &mut self.deluxe_ports[0])?;
        //             // }
        //             //
        //             // if self.deluxe_active[1].load(Ordering::Relaxed) {
        //             //     Self::write_to_deluxe(&mut self.read_buffer[7..11], &mut self.deluxe_ports[1])?;
        //             // }

        //             Ok(())
        //         }
        //         Err(ref err) if err.kind() == io::ErrorKind::TimedOut => Ok(()),
        //         Err(err) => Err(err.into()),
        //     }
    }

    pub fn init(&mut self) -> Result<()> {
        const RETRY_COUNT: u8 = 5;

        self.port.set_read_timeout(Duration::from_secs(5))?;
        self.port.set_write_timeout(Duration::from_secs(5))?;

        for c in 0..RETRY_COUNT {
            info!("Trying to initialize Finale Touchscreen. Attempt {}", c + 1);
            match self.send_init() {
                Ok(()) => {
                    self.port.set_read_timeout(Duration::from_millis(0))?;
                    self.port.set_write_timeout(Duration::from_millis(0))?;
                    return Ok(());
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    error!("Init timeout")
                }
                Err(e) => {
                    error!("Init failed. Error: {}", e);
                    return Err(e.into());
                }
            }
        }

        error!("Failed to connect to Finale Touchscreen");
        Err(io::Error::new(io::ErrorKind::TimedOut, "Finale touchscreen timeout").into())
    }

    fn send_init(&mut self) -> io::Result<()> {
        let mut read_buffer: [u8; 6] = [0; 6];

        info!("Sending HALT packet");
        self.port.write_all(HALT)?;

        // Discard input buffer so there is no data if touch was working before
        self.port.discard_input_buffer()?;

        for panel in [b'L', b'R'] {
            info!("Getting threshold from {} panel areas", panel as char);
            for area in 65..82 {
                self.get_threshold(panel, area)?;
            }
        }

        info!("Sending STAT packet");
        self.port.write_all(STAT)?;

        Ok(())
    }

    fn get_threshold(&mut self, panel: u8, area: u8) -> io::Result<()> {
        // info!(
        //     "Writing {:?}",
        //     write_buf.iter().map(|&u| u as char).collect::<Vec<char>>()
        // );
        self.port.write_all(&[b'{', panel, area, b't', b'h', b'}'])
    }

    fn set_threshold(&mut self, panel: u8, area: u8, threshold: u8) -> Result<()> {
        self.port
            .write_all(&[b'{', panel, area, b't', threshold, b'}'])
    }

    pub fn send(&self, buf: &[u8]) -> Result<()> {
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

    pub fn receive(&mut self) -> Result<Option<usize>> {
        match self.port.read(&mut self.buf[self.byte_count..]) {
            Ok(cnt) => {
                self.byte_count += cnt;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => return Ok(None),
            Err(e) => {
                self.byte_count = 0;
                return Err(e);
            }
        }

        let mut buf = &mut self.buf[..self.byte_count];
        info!(
            "Reading {:?}",
            buf
                .iter()
                .map(|&u| u as char)
                .collect::<Vec<char>>()
        );

        match buf.iter().position(|b| b == &b'(') {
            Some(0) => {}
            Some(i) => {
                buf.rotate_left(i - 1);
                self.byte_count -= i + 1;
                buf = &mut self.buf[..self.byte_count];
            }
            None => {
                self.byte_count = 0;
            }
        };

        match buf.iter().rposition(|b| b == &b')') {
            Some(i) => Ok(Some(i)),
            None => Ok(None),
        }
    }

    // Sends HALT packet to touchscreen
    pub fn halt(&mut self) -> Result<()> {
        self.port.write_all(HALT)
    }

    // Sends STAT packet to touchscreen
    pub fn stat(&mut self) -> Result<()> {
        self.port.write_all(STAT)
    }

    pub fn spawn_thread(
        mut finale_touch: Finale,
        exit_sig: Arc<AtomicBool>,
    ) -> Result<JoinHandle<Result<()>>> {
        thread::Builder::new()
            .name("Finale Touch Thread".to_owned())
            .spawn(move || {
                while !exit_sig.load(Ordering::Relaxed) {
                    finale_touch.process()?;
                }
                finale_touch.port.write_all(HALT)?;

                Ok(())
            })
    }
}

impl Drop for Finale {
    fn drop(&mut self) {
        if let Err(e) = self.halt() {
            error!("Failed to send HALT: {e}")
        };
    }
}

fn convert_to_dx_buf(buf: &[u8]) -> [u8; 9] {
    let mut write_buffer = DEFAULT_DELUXE_WRITE_BUFFER;
    for (i, &bit) in buf.iter().enumerate() {
        for pos in 0..5usize {
            if !bit_read(bit, pos) || FINALE_AREAS[i][pos] == STUB_AREAS {
                continue;
            }

            FINALE_AREAS[i][pos]
                .iter()
                .for_each(|a| write_buffer[a.0] |= a.1);
        }
    }

    write_buffer
}

static DEFAULT_DELUXE_WRITE_BUFFER: [u8; 9] = [b'(', 0, 0, 0, 0, 0, 0, 0, b')'];

static FINALE_AREAS: [[[(usize, u8); 3]; 5]; 4] = [
    [
        [A1, D1, D2],
        [B1, E1, E2],
        [A2, D2, D3],
        [B2, E2, E3],
        STUB_AREAS,
    ],
    [
        [A3, D3, D4],
        [B3, E3, E4],
        [A4, D4, D5],
        [B4, E4, E5],
        STUB_AREAS,
    ],
    [
        [A5, D5, D6],
        [B5, E5, E6],
        [A6, D6, D7],
        [B6, E6, E7],
        STUB_AREAS,
    ],
    [
        [A7, D7, D8],
        [B7, E7, E8],
        [A8, D8, D1],
        [B8, E8, E1],
        [C1, C2, STUB_AREA],
    ],
];

/// Mapping for Deluxe touch areas
/// (usize, u8) = (Index of DELUXE_WRITE_BUFFER, Bit Position)
const A1: (usize, u8) = (1, 1);
const A2: (usize, u8) = (1, 2);
const A3: (usize, u8) = (1, 4);
const A4: (usize, u8) = (1, 8);
const A5: (usize, u8) = (1, 16);

const A6: (usize, u8) = (2, 1);
const A7: (usize, u8) = (2, 2);
const A8: (usize, u8) = (2, 4);
const B1: (usize, u8) = (2, 8);
const B2: (usize, u8) = (2, 16);

const B3: (usize, u8) = (3, 1);
const B4: (usize, u8) = (3, 2);
const B5: (usize, u8) = (3, 4);
const B6: (usize, u8) = (3, 8);
const B7: (usize, u8) = (3, 16);

const B8: (usize, u8) = (4, 1);
const C1: (usize, u8) = (4, 2);
const C2: (usize, u8) = (4, 4);
const D1: (usize, u8) = (4, 8);
const D2: (usize, u8) = (4, 16);

const D3: (usize, u8) = (5, 1);
const D4: (usize, u8) = (5, 2);
const D5: (usize, u8) = (5, 4);
const D6: (usize, u8) = (5, 8);
const D7: (usize, u8) = (5, 16);

const D8: (usize, u8) = (6, 1);
const E1: (usize, u8) = (6, 2);
const E2: (usize, u8) = (6, 4);
const E3: (usize, u8) = (6, 8);
const E4: (usize, u8) = (6, 16);

const E5: (usize, u8) = (7, 1);
const E6: (usize, u8) = (7, 2);
const E7: (usize, u8) = (7, 4);
const E8: (usize, u8) = (7, 8);

const STUB_AREA: (usize, u8) = (0, 0);
const STUB_AREAS: [(usize, u8); 3] = [STUB_AREA, STUB_AREA, STUB_AREA];
