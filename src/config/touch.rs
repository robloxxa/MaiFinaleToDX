use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Touch {
    /// Enable TouchScreen feature
    pub enabled: bool,

    /// COM Port for Finale touch
    ///
    /// Port of the real device.
    pub finale_port: String,

    /// COM Port for Deluxe Player 1 touch
    ///
    /// Port of the emulated device for Deluxe Player 1.
    pub dx_p1_port: String,

    /// COM Port for Deluxe Player 2 touch
    ///
    /// Port of the emulated device for Deluxe Player 1.
    pub dx_p2_port: String,
}

impl Default for Touch {
    fn default() -> Self {
        Self {
            enabled: true,
            finale_port: "COM23".to_string(),
            dx_p1_port: "COM6".to_string(),
            dx_p2_port: "COM8".to_string(),
        }
    }
}
