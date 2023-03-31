use std::ptr::write;
use std::time::{Duration, Instant};

use serialport::SerialPort;

use crate::helper_funcs::bit_read;
use crate::touch::alls::AllsTouchMasterCommand;
use crate::touch::AllsMessageCmd;

pub struct RingEdge2 {
    pub port: Box<dyn SerialPort>,

    read_buffer: [u8; 14],
    pub alls_ports: [Box<dyn SerialPort>; 2],
    pub alls_active: [bool; 2],
}

impl RingEdge2 {
    pub fn new(
        port_name: String,
        baud_rate: u32,
        alls_p1_port: Box<dyn SerialPort>,
        alls_p2_port: Box<dyn SerialPort>,
    ) -> Result<Self, serialport::Error> {
        let mut port = serialport::new(port_name, baud_rate).open()?;
        port.set_timeout(Duration::from_millis(1))?;

        Ok(Self {
            port,
            read_buffer: [0; 14],
            alls_ports: [alls_p1_port, alls_p2_port],
            alls_active: [true, true],
        })
    }

    pub fn read(&mut self) {
        let read_timer = Instant::now();
        if let Err(err) = self.port.read_exact(self.read_buffer[0..6].as_mut()) {
            if err.kind() == std::io::ErrorKind::TimedOut {
                return;
            } else {
                panic!("{}", err);
            }
        }
        if self.read_buffer[5] == b')' {
            todo!();
        }

        if let Err(err) = self.port.read_exact(self.read_buffer[6..14].as_mut()) {
            panic!("{}", err);
        }
        let read_timer_elapsed = read_timer.elapsed();
        let write_timer = Instant::now();
        if self.alls_active[0] {
            Self::send_to_alls(self.read_buffer[1..5].as_mut(), self.alls_ports[0].as_mut());
        }

        if self.alls_active[1] {
            Self::send_to_alls(
                self.read_buffer[7..11].as_mut(),
                self.alls_ports[1].as_mut(),
            );
        }
        let write_timer_elapsed = write_timer.elapsed();
        println!("{:?}, {:?}", read_timer_elapsed, write_timer_elapsed);
    }

    pub fn parse_command_from_alls(&mut self, msg: AllsMessageCmd) {
        match msg.cmd {
            AllsTouchMasterCommand::Halt => {
                self.alls_active[msg.player_num] = false;
            }
            AllsTouchMasterCommand::Stat => {
                self.alls_active[msg.player_num] = true;
            }
            AllsTouchMasterCommand::Ratio(l_r, area, value) => {
                self.alls_ports[msg.player_num]
                    .write(&[
                        b'(',
                        l_r,
                        area,
                        AllsTouchMasterCommand::Ratio as u8,
                        value,
                        b')',
                    ])
                    .unwrap();
                self.alls_ports[msg.player_num].flush().unwrap();
            }
            AllsTouchMasterCommand::Sens(l_r, area, value) => {
                self.alls_ports[msg.player_num]
                    .write(&[
                        b'(',
                        l_r,
                        area,
                        AllsTouchMasterCommand::Sens as u8,
                        value,
                        b')',
                    ])
                    .unwrap();
                self.alls_ports[msg.player_num].flush().unwrap();
            }
            _ => {}
        };
    }

    fn send_to_alls(buf: &mut [u8], port: &mut dyn SerialPort) {
        let mut write_buffer = DEFAULT_ALLS_WRITE_BUFFER;
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
        port.write(write_buffer.as_mut()).unwrap();
        port.flush().unwrap();
    }
}

static DEFAULT_ALLS_WRITE_BUFFER: [u8; 9] = [b'(', 0, 0, 0, 0, 0, 0, 0, b')'];

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

/// Mapping for Maimai ALLs touch areas
/// (usize, u8) = (Index of ALLS_WRITE_BUFFER, Bit Position)
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
