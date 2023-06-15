use log::{debug, error};
use std::io;
use std::io::{Read, Write};
use std::time::Duration;

use serialport::{COMPort, SerialPort};

use crate::helper_funcs::bit_read;
use crate::touch::deluxe::TouchMasterCommand;
use crate::touch::{MessageCmd, HALT};

pub struct RingEdge2 {
    pub port: COMPort,

    read_buffer: [u8; 14],
    pub deluxe_ports: [COMPort; 2],
    pub deluxe_active: [bool; 2],
}

// TODO: Send data over channel to

impl RingEdge2 {
    pub fn new(
        port_name: String,
        deluxe_p1_port: COMPort,
        deluxe_p2_port: COMPort,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, 9600).open_native()?;
        port.set_timeout(Duration::from_millis(0))?;

        Ok(Self {
            port,
            read_buffer: [0; 14],
            deluxe_ports: [deluxe_p1_port, deluxe_p2_port],
            deluxe_active: [false, false],
        })
    }

    pub fn read(&mut self) {
        if let Err(err) = self.port.read_exact(self.read_buffer[0..14].as_mut()) {
            if err.kind() == io::ErrorKind::TimedOut {
                return;
            } else {
                panic!("{}", err);
            }
        }

        // if self.read_buffer[0] != b'(' {
        //     todo!();
        // }
        //
        // if self.read_buffer[5] == b')' {
        //     todo!();
        // }
        //
        // if let Err(err) = self.port.read_exact(self.read_buffer[6..14].as_mut()) {
        //     panic!("{}", err);
        // }

        if self.deluxe_active[0] {
            Self::send_to_deluxe(self.read_buffer[1..5].as_mut(), &mut self.deluxe_ports[0]);
        }

        if self.deluxe_active[1] {
            Self::send_to_deluxe(self.read_buffer[7..11].as_mut(), &mut self.deluxe_ports[1]);
        }
    }

    pub fn parse_command_from_alls(&mut self, msg: MessageCmd) -> io::Result<()> {
        debug!("P{}: {:?}", msg.player_num + 1, msg.cmd);
        match msg.cmd {
            TouchMasterCommand::Reset => {
                self.deluxe_active[msg.player_num] = false;
            }
            TouchMasterCommand::Halt => {
                self.deluxe_active[msg.player_num] = false;
            }
            TouchMasterCommand::Stat => {
                self.deluxe_active[msg.player_num] = true;
            }
            TouchMasterCommand::Ratio(l_r, area, value) => {
                self.deluxe_ports[msg.player_num].write(&[
                    b'(',
                    l_r,
                    area,
                    TouchMasterCommand::Ratio as u8,
                    value,
                    b')',
                ])?;
            }
            TouchMasterCommand::Sens(l_r, area, value) => {
                self.deluxe_ports[msg.player_num].write(&[
                    b'(',
                    l_r,
                    area,
                    TouchMasterCommand::Sens as u8,
                    value,
                    b')',
                ])?;
            }
            _ => {}
        };
        Ok(())
    }

    fn send_to_deluxe(buf: &mut [u8], port: &mut dyn SerialPort) {
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
        if write_buffer.ne(&DEFAULT_DELUXE_WRITE_BUFFER) {
            debug!(
                "Touch pressed on {}, {:?}",
                port.name().unwrap(),
                &write_buffer
            );
        }
        port.write(&write_buffer).unwrap();
    }
}

impl Drop for RingEdge2 {
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
static A1: (usize, u8) = (1, 1);
static A2: (usize, u8) = (1, 2);
static A3: (usize, u8) = (1, 4);
static A4: (usize, u8) = (1, 8);
static A5: (usize, u8) = (1, 16);

static A6: (usize, u8) = (2, 1);
static A7: (usize, u8) = (2, 2);
static A8: (usize, u8) = (2, 4);
static B1: (usize, u8) = (2, 8);
static B2: (usize, u8) = (2, 16);

static B3: (usize, u8) = (3, 1);
static B4: (usize, u8) = (3, 2);
static B5: (usize, u8) = (3, 4);
static B6: (usize, u8) = (3, 8);
static B7: (usize, u8) = (3, 16);

static B8: (usize, u8) = (4, 1);
static C1: (usize, u8) = (4, 2);
static C2: (usize, u8) = (4, 4);
static D1: (usize, u8) = (4, 8);
static D2: (usize, u8) = (4, 16);

static D3: (usize, u8) = (5, 1);
static D4: (usize, u8) = (5, 2);
static D5: (usize, u8) = (5, 4);
static D6: (usize, u8) = (5, 8);
static D7: (usize, u8) = (5, 16);

static D8: (usize, u8) = (6, 1);
static E1: (usize, u8) = (6, 2);
static E2: (usize, u8) = (6, 4);
static E3: (usize, u8) = (6, 8);
static E4: (usize, u8) = (6, 16);

static E5: (usize, u8) = (7, 1);
static E6: (usize, u8) = (7, 2);
static E7: (usize, u8) = (7, 4);
static E8: (usize, u8) = (7, 8);
