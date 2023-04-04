use clap::Parser;
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
    /// Specify path to config, default is
    #[serde(skip)]
    #[arg(long, default_value = "./config.toml")]
    pub config_path: String,

    /// Log level, options: INFO, WARN,
    #[serde(default)]
    #[arg(long, default_value = "info")]
    pub log_level: String,

    #[serde(default)]
    #[clap_serde]
    #[command(flatten)]
    pub settings: Settings,

    #[serde(default)]
    #[clap_serde]
    #[arg(skip)]
    pub input: Input,
}

#[derive(Parser, ClapSerde, Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    /// When set to True (or presented) will disable touch features
    #[arg(long, default_value = "false")]
    pub disable_touch: bool,

    /// When set to True (or presented) will disable JVS features
    #[arg(long, default_value = "false")]
    pub disable_jvs: bool,

    /// COM Port for Finale touch
    #[arg(long, default_value = "COM9")]
    pub touch_re2_com: String,

    /// COM Port for Deluxe Player 1 touch
    #[arg(long, default_value = "COM6")]
    pub touch_alls_p1_com: String,

    /// COM Port for Deluxe Player 2 touch
    #[arg(long, default_value = "COM7")]
    pub touch_alls_p2_com: String,

    /// COM Port for Finale's JVS
    #[arg(long, default_value = "COM24")]
    pub jvs_re2_com: String,
}

#[derive(ClapSerde, Deserialize, Serialize, Debug, Clone)]
pub struct Input {
    #[default(0x32)] // 3
    pub service: c_int,
    #[default(0x54)] // T
    pub test: c_int,

    #[default(0x57)] // W
    pub p1_btn1: c_int,
    #[default(0x45)] // E
    pub p1_btn2: c_int,
    #[default(0x44)] // D
    pub p1_btn3: c_int,
    #[default(0x43)] // C
    pub p1_btn4: c_int,
    #[default(0x58)] // X
    pub p1_btn5: c_int,
    #[default(0x5A)] // Z
    pub p1_btn6: c_int,
    #[default(0x41)] // A
    pub p1_btn7: c_int,
    #[default(0x51)] // Q
    pub p1_btn8: c_int,

    #[default(VK_NUMPAD8)]
    pub p2_btn1: c_int,
    #[default(VK_NUMPAD9)]
    pub p2_btn2: c_int,
    #[default(VK_NUMPAD6)]
    pub p2_btn3: c_int,
    #[default(VK_NUMPAD3)]
    pub p2_btn4: c_int,
    #[default(VK_NUMPAD2)]
    pub p2_btn5: c_int,
    #[default(VK_NUMPAD1)]
    pub p2_btn6: c_int,
    #[default(VK_NUMPAD4)]
    pub p2_btn7: c_int,
    #[default(VK_NUMPAD7)]
    pub p2_btn8: c_int,
}

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
