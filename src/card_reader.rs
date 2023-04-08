use std::time::Duration;

use serialport::SerialPort;

#[derive(Debug)]
#[repr(u8)]
enum Command {
    LEDReset = 0x10,
    GetFirmware = 0x30,
    GetHardware = 0x32,
    RadioOn = 0x40,
    RadioOff = 0x41,
    Poll = 0x42,
    Reset = 0x62,
}

pub struct CardReader {
    re2_port: Box<dyn SerialPort>,
    alls_port: Box<dyn SerialPort>,
}

impl CardReader {
    pub fn new(re2_port_name: String, alls_port_name: String) -> Result<Self, serialport::Error> {
        let mut re2_port = serialport::new(re2_port_name, 38_400).open()?;
        let mut alls_port = serialport::new(alls_port_name, 115_200).open()?;
        re2_port.set_timeout(Duration::from_millis(500))?;
        alls_port.set_timeout(Duration::from_millis(500))?;

        Ok(Self {
            re2_port,
            alls_port,
        })
    }

    pub fn re2_port(&mut self) {}

    pub fn read_alls(&mut self) {}
}
