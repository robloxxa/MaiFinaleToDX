use log::{debug, error, info};

use std::io::Result;
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use serial2::SerialPort;

use crate::helper_funcs::{bit_read, ReadExt};
use crate::touch::deluxe::Deluxe;
use crate::touch::{HALT, STAT};

pub struct Finale {
    pub reader: BufReader<SerialPort>,
    pub writer: SerialPort,

    byte_count: usize,
    read_buffer: [u8; 14],
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
            reader: BufReader::new(port.try_clone()?),
            writer: port,
            byte_count: 0,
            read_buffer: [0; 14],
            dx_p1,
            dx_p2,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        match self.reader.read_u8() {
            Ok(b) => {
                self.byte_count += 1;
                match b {
                    b'(' => {
                        self.byte_count = 0;
                        self.read_buffer[0] = b
                    }
                    b')' => {
                        self.read_buffer[self.byte_count] = b;
                        if self.byte_count > 5 {
                            self.dx_p1.as_ref().map_or_else(
                                || Ok(()),
                                |dx| dx.send(&convert_to_dx_buf(&self.read_buffer[1..5])),
                            )?;
                            self.dx_p2.as_ref().map_or_else(
                                || Ok(()),
                                |dx| dx.send(&convert_to_dx_buf(&self.read_buffer[1..5])),
                            )?;
                        } else {
                        }
                        // info!("Reading {:?}", self.read_buffer[..=self.byte_count].iter().map(|&u| u as char).collect::<Vec<char>>());
                    }
                    _ => {
                        self.read_buffer[self.byte_count] = b;
                    }
                };
                Ok(())
            }
            Err(ref err) if err.kind() == io::ErrorKind::TimedOut => Ok(()),
            Err(err) => Err(err.into()),
        }
        // match self.reader.read_exact(&mut self.read_buffer[0..=6]) {
        //     Ok(_) => {
        //         info!("Reading {:?}", self.read_buffer[0..=6].iter().map(|&u| u as char).collect::<Vec<char>>());
        //         // TODO: Check how well behave relaxed ordering
        //         // Also maybe with serial2 we can read it without any delay? Since it uses different timeout settings.
        //         // if self.deluxe_active[0].load(Ordering::Relaxed) {
        //         //     Self::write_to_deluxe(&mut self.read_buffer[1..5], &mut self.deluxe_ports[0])?;
        //         // }
        //         //
        //         // if self.deluxe_active[1].load(Ordering::Relaxed) {
        //         //     Self::write_to_deluxe(&mut self.read_buffer[7..11], &mut self.deluxe_ports[1])?;
        //         // }
        //
        //         Ok(())
        //     }
        //     Err(ref err) if err.kind() == io::ErrorKind::TimedOut => Ok(()),
        //     Err(err) => Err(err.into()),
        // }
    }

    pub fn init(&mut self) -> Result<()> {
        const RETRY_COUNT: u8 = 5;

        self.reader
            .get_mut()
            .set_read_timeout(Duration::from_secs(2))?;
        self.writer.set_write_timeout(Duration::from_secs(2))?;

        for c in 0..RETRY_COUNT {
            info!("Trying to initialize Finale Touchscreen. Attempt {}", c + 1);
            match self.send_init() {
                Ok(()) => {
                    self.reader
                        .get_mut()
                        .set_read_timeout(Duration::from_millis(0))?;
                    self.writer.set_write_timeout(Duration::from_millis(0))?;
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
        self.writer.write_all(HALT)?;

        // Discard input buffer so there is no data if touch was working before
        self.reader.get_ref().discard_input_buffer()?;

        for panel in [b'L', b'R'] {
            info!("Getting threshold from {} panel areas", panel as char);
            for area in 65..82 {
                self.get_threshold(read_buffer.as_mut_slice(), panel, area)?;
            }
        }

        info!("Sending STAT packet");
        self.writer.write_all(STAT)?;

        Ok(())
    }

    fn get_threshold(&mut self, buf: &mut [u8], panel: u8, area: u8) -> io::Result<()> {
        let write_buf = [b'{', panel, area, b't', b'h', b'}'];
        info!(
            "Writing {:?}",
            write_buf.iter().map(|&u| u as char).collect::<Vec<char>>()
        );
        self.writer.write_all(&write_buf)?;
        self.reader.read_exact(buf)
    }

    fn set_threshold(
        &mut self,
        buf: &mut [u8],
        panel: u8,
        area: u8,
        threshold: u8,
    ) -> io::Result<()> {
        self.writer
            .write_all(&[b'{', panel, area, b't', threshold, b'}'])
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
                finale_touch.writer.write_all(HALT)?;

                Ok(())
            })
    }
}

impl Drop for Finale {
    fn drop(&mut self) {
        if let Err(e) = self.writer.write(HALT) {
            error!("Failed to send HALT: {e}")
        };
    }
}

fn convert_to_dx_buf(buf: &[u8]) -> [u8; 9] {
    let mut write_buffer = DEFAULT_DELUXE_WRITE_BUFFER;
    for (i, &bit) in buf.iter().enumerate() {
        for pos in 0..5usize {
            if !bit_read(bit, pos) {
                continue;
            }

            if let Some(areas) = FINALE_AREAS[i][pos] {
                areas.iter().for_each(|a| write_buffer[a.0] |= a.1);
            }
        }
    }

    write_buffer
}

static DEFAULT_DELUXE_WRITE_BUFFER: [u8; 9] = [b'(', 0, 0, 0, 0, 0, 0, 0, b')'];

static FINALE_AREAS: [[Option<[(usize, u8); 3]>; 5]; 4] = [
    [
        Some([A1, D1, D2]),
        Some([B1, E1, E2]),
        Some([A2, D2, D3]),
        Some([B2, E2, E3]),
        None,
    ],
    [
        Some([A3, D3, D4]),
        Some([B3, E3, E4]),
        Some([A4, D4, D5]),
        Some([B4, E4, E5]),
        None,
    ],
    [
        Some([A5, D5, D6]),
        Some([B5, E5, E6]),
        Some([A6, D6, D7]),
        Some([B6, E6, E7]),
        None,
    ],
    [
        Some([A7, D7, D8]),
        Some([B7, E7, E8]),
        Some([A8, D8, D1]),
        Some([B8, E8, E1]),
        Some([C1, C2, (0, 0)]),
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
