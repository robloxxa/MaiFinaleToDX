use anyhow::{Context, Error, Result};
use log::{debug, error};

use std::io;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serial2::SerialPort;

use crate::helper_funcs::{bit_read};
use crate::touch::HALT;

pub struct Finale {
    pub port: SerialPort,

    read_buffer: [u8; 14],
    pub deluxe_ports: [SerialPort; 2],
    pub deluxe_active: [Arc<AtomicBool>; 2],
}

impl Finale {
    pub fn new(
        port_name: impl Into<String>,
        p1_port: SerialPort,
        p2_port: SerialPort,
        p1_active: &Arc<AtomicBool>,
        p2_active: &Arc<AtomicBool>,
    ) -> Result<Self> {
        let port_name = port_name.into();
        let mut port = SerialPort::open(&port_name, 9600)
        .with_context(|| format!("Failed to open port {}", port_name))?;
        port.set_read_timeout(Duration::from_millis(0))?;

        Ok(Self {
            port,
            read_buffer: [0; 14],
            deluxe_ports: [p1_port, p2_port],
            deluxe_active: [p1_active.clone(), p2_active.clone()],
        })
    }

    pub fn process(&mut self) -> Result<()> {
        match self.port.read_exact(&mut self.read_buffer) {
            Ok(_) => {
                // TODO: Check how well behave relaxed ordering
                // Also maybe with serial2 we can read it without any delay? Since it uses different timeout settings.
                if self.deluxe_active[0].load(Ordering::Relaxed) {
                    Self::write_to_deluxe(&mut self.read_buffer[1..5], &mut self.deluxe_ports[0])?;
                }

                if self.deluxe_active[1].load(Ordering::Relaxed) {
                    Self::write_to_deluxe(&mut self.read_buffer[7..11], &mut self.deluxe_ports[1])?;
                }

                Ok(())
            }
            Err(ref err) if err.kind() == io::ErrorKind::TimedOut => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }

    // pub fn parse_command_from_alls(&mut self, msg: MessageCmd) -> io::Result<()> {
    //     debug!("P{}: {:?}", msg.player_num + 1, msg.cmd);
    //     match msg.cmd {
    //         MasterCommand::Reset => {
    //             self.deluxe_active[msg.player_num] = false;
    //         }
    //         MasterCommand::Halt => {
    //             self.deluxe_active[msg.player_num] = false;
    //         }
    //         MasterCommand::Stat => {
    //             self.deluxe_active[msg.player_num] = true;
    //         }
    //         MasterCommand::Ratio(l_r, area, value) => {
    //             self.deluxe_ports[msg.player_num].write(&[
    //                 b'(',
    //                 l_r,
    //                 area,
    //                 MasterCommand::Ratio as u8,
    //                 value,
    //                 b')',
    //             ])?;
    //         }
    //         MasterCommand::Sens(l_r, area, value) => {
    //             self.deluxe_ports[msg.player_num].write(&[
    //                 b'(',
    //                 l_r,
    //                 area,
    //                 MasterCommand::Sens as u8,
    //                 value,
    //                 b')',
    //             ])?;
    //         }
    //         _ => {}
    //     };
    //     Ok(())
    // }

    fn write_to_deluxe(buf: &mut [u8], port: &mut SerialPort) -> Result<()> {
        let mut write_buffer = DEFAULT_DELUXE_WRITE_BUFFER;
        for (i, bit) in buf.iter().enumerate() {
            for pos in 0..5 as usize {
                if !bit_read(bit, pos) {
                    continue;
                }

                if let Some(areas) = FINALE_AREAS[i][pos] {
                    areas.iter().for_each(|a| write_buffer[a.0] |= a.1);
                }
            }
        }
        // debug!("{:02X?} {:02X?}", &write_buffer, &DEFAULT_ALLS_WRITE_BUFFER);
        // if write_buffer.ne(&DEFAULT_DELUXE_WRITE_BUFFER) {
        //     debug!(
        //         "Touch pressed on {}, {:?}",
        //         port.name(),
        //         &write_buffer
        //     );
        // }
        Ok(port.write_all(&write_buffer)?)
    }
}

impl Drop for Finale {
    fn drop(&mut self) {
        if let Err(e) = self.port.write(HALT) {
            error!("Failed to send HALT: {e}")
        };
    }
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
