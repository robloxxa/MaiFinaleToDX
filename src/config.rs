use clap::{ArgAction, Parser};
use clap_serde_derive::ClapSerde;
use serde::{Deserialize, Serialize};
use winapi::ctypes::c_int;
use winapi::um::winuser::{
    VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
};

#[derive(Parser, ClapSerde, Deserialize, Serialize, Debug)]
#[clap(author = "robloxxa", version, about, long_about = None)]
/// Tool that allow playing Maimai DX on original Maimai Finale Cabinet
pub struct Config {
    /// Specify path to config
    #[serde(skip)]
    #[arg(long, default_value = "./config.toml")]
    pub config_path: String,

    #[serde(skip)]
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    /// Creates a new config with default values
    pub create_config: bool,

    /// Log level, options: INFO, WARN, DEBUG, TRACE
    #[serde(skip)]
    #[arg(long, default_value = "info")]
    pub log_level: String,

    #[clap_serde]
    #[command(flatten)]
    pub settings: Settings,

    #[clap_serde]
    #[arg(skip)]
    pub input: Input,
}

#[derive(Parser, ClapSerde, Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    /// Enable TouchScreen feature
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    pub touch: bool,

    /// Enable JVS feature
    /// 
    /// This will try to read from JVS com port (specified by `jvs_port`) and use it as a keyboard.
    /// See [`Input`] to see what keys are emulated.
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    pub jvs: bool,

    /// Disable NFC reader feature
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    pub reader: bool,

    /// Enable Spice API feature
    /// 
    /// WARNING: NOT IMPLEMENTED YET
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    pub spice_api: bool,

    /// COM Port for Finale touch
    #[arg(long, default_value = "COM23")]
    pub touch_finale_port: String,

    /// COM Port for Deluxe Player 1 touch
    #[arg(long, default_value = "DX_TOUCH_1")]
    pub touch_dx_p1_port: String,

    /// COM Port for Deluxe Player 2 touch
    #[arg(long, default_value = "COM8")]
    pub touch_dx_p2_port: String,

    /// COM Port for Finale's JVS
    #[arg(long, default_value = "COM24")]
    pub jvs_port: String,

    /// COM Port for card reader
    #[arg(long, default_value = "COM22")]
    pub reader_port: String,

    #[arg(long)]
    pub reader_device_file: Option<String>,

    /// Spice port for Spice API
    /// 
    /// WARNING: NOT IMPLEMENTED YET
    #[arg(long, default_value = "1337")]
    pub spice_port: String,
}

#[derive(ClapSerde, Deserialize, Serialize, Debug, Clone)]
pub struct Input {
    #[default(SERVICE_DEFAULT)]
    pub service: c_int,
    #[default(TEST_DEFAULT)]
    pub test: c_int,

    #[default(P1_BTN1_DEFAULT)]
	pub p1_btn1: c_int,
    #[default(P1_BTN2_DEFAULT)]
	pub p1_btn2: c_int,
    #[default(P1_BTN3_DEFAULT)]
	pub p1_btn3: c_int,
    #[default(P1_BTN4_DEFAULT)]
	pub p1_btn4: c_int,
    #[default(P1_BTN5_DEFAULT)]
	pub p1_btn5: c_int,
    #[default(P1_BTN6_DEFAULT)]
	pub p1_btn6: c_int,
    #[default(P1_BTN7_DEFAULT)]
	pub p1_btn7: c_int,
    #[default(P1_BTN8_DEFAULT)]
	pub p1_btn8: c_int,

    #[default(P2_BTN1_DEFAULT)]
	pub p2_btn1: c_int,
    #[default(P2_BTN2_DEFAULT)]
	pub p2_btn2: c_int,
    #[default(P2_BTN3_DEFAULT)]
	pub p2_btn3: c_int,
    #[default(P2_BTN4_DEFAULT)]
	pub p2_btn4: c_int,
    #[default(P2_BTN5_DEFAULT)]
	pub p2_btn5: c_int,
    #[default(P2_BTN6_DEFAULT)]
	pub p2_btn6: c_int,
    #[default(P2_BTN7_DEFAULT)]
	pub p2_btn7: c_int,
    #[default(P2_BTN8_DEFAULT)]
	pub p2_btn8: c_int,
}

const TEST_DEFAULT: c_int = 0x54;
const SERVICE_DEFAULT: c_int = 0x33;

// W
const P1_BTN1_DEFAULT: c_int = 0x57;
// E
const P1_BTN2_DEFAULT: c_int = 0x45;
// D
const P1_BTN3_DEFAULT: c_int = 0x44;
// C
const P1_BTN4_DEFAULT: c_int = 0x43;
// X
const P1_BTN5_DEFAULT: c_int = 0x58;
// Z
const P1_BTN6_DEFAULT: c_int = 0x5A;
// A
const P1_BTN7_DEFAULT: c_int = 0x41;
// Q
const P1_BTN8_DEFAULT: c_int = 0x51; 

const P2_BTN1_DEFAULT: c_int = VK_NUMPAD8;
const P2_BTN2_DEFAULT: c_int = VK_NUMPAD9;
const P2_BTN3_DEFAULT: c_int = VK_NUMPAD6;
const P2_BTN4_DEFAULT: c_int = VK_NUMPAD3;
const P2_BTN5_DEFAULT: c_int = VK_NUMPAD2;
const P2_BTN6_DEFAULT: c_int = VK_NUMPAD1;
const P2_BTN7_DEFAULT: c_int = VK_NUMPAD4;
const P2_BTN8_DEFAULT: c_int = VK_NUMPAD7;

// impl Default for Input {
//     fn default() -> Self {
//         Self {
//             service: 0x32, // 3
//             test: 0x54,    // T
//
//             p1_btn1: 0x57, // W
//             p1_btn2: 0x45, // E
//             p1_btn3: 0x44, // D
//             p1_btn4: 0x43, // C
//             p1_btn5: 0x58, // X
//             p1_btn6: 0x5A, // Z
//             p1_btn7: 0x41, // A
//             p1_btn8: 0x51, // Q
//
//             p2_btn1: VK_NUMPAD8,
//             p2_btn2: VK_NUMPAD9,
//             p2_btn3: VK_NUMPAD6,
//             p2_btn4: VK_NUMPAD3,
//             p2_btn5: VK_NUMPAD2,
//             p2_btn6: VK_NUMPAD1,
//             p2_btn7: VK_NUMPAD4,
//             p2_btn8: VK_NUMPAD7,
//         }
//     }
// }
