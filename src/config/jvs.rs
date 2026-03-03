use serde::{Deserialize, Serialize};
use winapi::ctypes::c_int;
use winapi::um::winuser::{
    VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum JvsMode {
    #[default]
    Hardware,
    Emulated,
}

const TEST_DEFAULT: c_int = 0x54;
const SERVICE_DEFAULT: c_int = 0x33;

const P1_BTN1_DEFAULT: c_int = 0x57;
const P1_BTN2_DEFAULT: c_int = 0x45;
const P1_BTN3_DEFAULT: c_int = 0x44;
const P1_BTN4_DEFAULT: c_int = 0x43;
const P1_BTN5_DEFAULT: c_int = 0x58;
const P1_BTN6_DEFAULT: c_int = 0x5A;
const P1_BTN7_DEFAULT: c_int = 0x41;
const P1_BTN8_DEFAULT: c_int = 0x51;

const P2_BTN1_DEFAULT: c_int = VK_NUMPAD8;
const P2_BTN2_DEFAULT: c_int = VK_NUMPAD9;
const P2_BTN3_DEFAULT: c_int = VK_NUMPAD6;
const P2_BTN4_DEFAULT: c_int = VK_NUMPAD3;
const P2_BTN5_DEFAULT: c_int = VK_NUMPAD2;
const P2_BTN6_DEFAULT: c_int = VK_NUMPAD1;
const P2_BTN7_DEFAULT: c_int = VK_NUMPAD4;
const P2_BTN8_DEFAULT: c_int = VK_NUMPAD7;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Jvs {
    /// Enable Jvs feature
    ///
    /// This will try to read from Jvs com port (specified by `jvs_port`) and use it as a keyboard.
    /// See [`Input`] to see what keys are emulated.
    pub enabled: bool,

    #[serde(default)]
    pub mode: JvsMode,

    /// COM Port for Finale's Jvs
    pub port: String,

    pub init_retry_count: Option<i64>,

    /// Jvs input bindings to keyboard
    #[serde(default)]
    pub input: Input,
}

impl Default for Jvs {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: JvsMode::default(),
            port: "COM23".to_string(),
            init_retry_count: None,
            input: Input::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
            service: SERVICE_DEFAULT,
            test: TEST_DEFAULT,

            p1_btn1: P1_BTN1_DEFAULT,
            p1_btn2: P1_BTN2_DEFAULT,
            p1_btn3: P1_BTN3_DEFAULT,
            p1_btn4: P1_BTN4_DEFAULT,
            p1_btn5: P1_BTN5_DEFAULT,
            p1_btn6: P1_BTN6_DEFAULT,
            p1_btn7: P1_BTN7_DEFAULT,
            p1_btn8: P1_BTN8_DEFAULT,

            p2_btn1: P2_BTN1_DEFAULT,
            p2_btn2: P2_BTN2_DEFAULT,
            p2_btn3: P2_BTN3_DEFAULT,
            p2_btn4: P2_BTN4_DEFAULT,
            p2_btn5: P2_BTN5_DEFAULT,
            p2_btn6: P2_BTN6_DEFAULT,
            p2_btn7: P2_BTN7_DEFAULT,
            p2_btn8: P2_BTN8_DEFAULT,
        }
    }
}
