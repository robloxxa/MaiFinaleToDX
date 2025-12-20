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
    
    #[serde(default)]
    pub p1_threshold: Threshold,
    
    #[serde(default)]
    pub p2_threshold: Threshold,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Threshold {
    // /// Use thresholds values from Maimai DX that are set via Service Mode
    // pub use_from_dx: bool,
    
    pub a1: u8,
    pub a2: u8,
    pub a3: u8,
    pub a4: u8,
    pub a5: u8,
    pub a6: u8,
    pub a7: u8,
    pub a8: u8,
    
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
    pub b4: u8,
    pub b5: u8,
    pub b6: u8,
    pub b7: u8,
    pub b8: u8,
    
    pub c: u8,
}

impl Default for Threshold {
    fn default() -> Self {
        Self {
            // use_from_dx: false,
            a1: 65,
            a2: 130,
            a3: 200,
            a4: 180,
            a5: 180,
            a6: 200,
            a7: 130,
            a8: 65,
            
            b1: 100,
            b2: 100,
            b3: 170,
            b4: 140,
            b5: 140,
            b6: 170,
            b7: 100,
            b8: 100,
            
            c: 110,
        }
    }
}

impl Default for Touch {
    fn default() -> Self {
        Self {
            enabled: true,
            finale_port: "COM23".to_string(),
            dx_p1_port: "COM6".to_string(),
            dx_p2_port: "COM8".to_string(),
            p1_threshold: Threshold::default(),
            p2_threshold: Threshold::default(),
        }
    }
}
