use serde::Deserialize;
use winapi::ctypes::c_int;
use winapi::um::winuser::{
    VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub settings: Settings,
    pub input: Input,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub service: c_int,
    pub test: c_int,

    pub p1_btn1: c_int,
    pub p1_btn2: c_int,
    pub p1_btn3: c_int,
    pub p1_btn4: c_int,
    pub p1_btn5: c_int,
    pub p1_btn6: c_int,
    pub p1_btn7: c_int,
    pub p1_btn8: c_int,

    pub p2_btn1: c_int,
    pub p2_btn2: c_int,
    pub p2_btn3: c_int,
    pub p2_btn4: c_int,
    pub p2_btn5: c_int,
    pub p2_btn6: c_int,
    pub p2_btn7: c_int,
    pub p2_btn8: c_int,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            service: 0x32, // 3
            test: 0x54,    // T

            p1_btn1: 0x57, // W
            p1_btn2: 0x45, // E
            p1_btn3: 0x44, // D
            p1_btn4: 0x43, // C
            p1_btn5: 0x58, // X
            p1_btn6: 0x5A, // Z
            p1_btn7: 0x41, // A
            p1_btn8: 0x51, // Q

            p2_btn1: VK_NUMPAD8,
            p2_btn2: VK_NUMPAD9,
            p2_btn3: VK_NUMPAD6,
            p2_btn4: VK_NUMPAD3,
            p2_btn5: VK_NUMPAD2,
            p2_btn6: VK_NUMPAD1,
            p2_btn7: VK_NUMPAD4,
            p2_btn8: VK_NUMPAD7,
        }
    }
}
