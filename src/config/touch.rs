use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

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
    
    pub p1_dx_touch_mapping: DXTouchAreaMapping,
    pub p2_dx_touch_mapping: DXTouchAreaMapping,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DXTouchAreaMapping {
    pub e1: TouchArea,
    pub e2: TouchArea,
    pub e3: TouchArea,
    pub e4: TouchArea,
    pub e5: TouchArea,
    pub e6: TouchArea,
    pub e7: TouchArea,
    pub e8: TouchArea,
    
    pub d1: TouchArea,
    pub d2: TouchArea,
    pub d3: TouchArea,
    pub d4: TouchArea,
    pub d5: TouchArea,
    pub d6: TouchArea,
    pub d7: TouchArea,
    pub d8: TouchArea,
    
    pub c1: TouchArea,
    pub c2: TouchArea,
}

impl Default for DXTouchAreaMapping {
    fn default() -> Self {
        Self {
            e1: TouchArea::new(vec!["B1".to_string(), "B8".to_string()], None, None),
            e2: TouchArea::new(vec!["B1".to_string(), "B2".to_string()], None, None),
            e3: TouchArea::new(vec!["B2".to_string(), "B3".to_string()], None, None),
            e4: TouchArea::new(vec!["B3".to_string(), "B4".to_string()], None, None),
            e5: TouchArea::new(vec!["B4".to_string(), "B5".to_string()], None, None),
            e6: TouchArea::new(vec!["B5".to_string(), "B6".to_string()], None, None),
            e7: TouchArea::new(vec!["B6".to_string(), "B7".to_string()], None, None),
            e8: TouchArea::new(vec!["B7".to_string(), "B8".to_string()], None, None),
            d1: TouchArea::new(vec!["A1".to_string(), "A8".to_string()], None, None),
            d2: TouchArea::new(vec!["A1".to_string(), "A2".to_string()], None, None),
            d3: TouchArea::new(vec!["A2".to_string(), "A3".to_string()], None, None),
            d4: TouchArea::new(vec!["A3".to_string(), "A4".to_string()], None, None),
            d5: TouchArea::new(vec!["A4".to_string(), "A5".to_string()], None, None),
            d6: TouchArea::new(vec!["A5".to_string(), "A6".to_string()], None, None),
            d7: TouchArea::new(vec!["A6".to_string(), "A7".to_string()], None, None),
            d8: TouchArea::new(vec!["A7".to_string(), "A8".to_string()], None, None),
            c1: TouchArea::new(vec!["C1".to_string()], None, None),
            c2: TouchArea::new(vec!["C1".to_string()], None, None),
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
pub struct DXTouchAreaSettings {
    #[serde_as(as = "Option<DurationMilliSeconds>")]
    pub deactivate_after: Option<Duration>,
    #[serde_as(as = "Option<DurationMilliSeconds>")]
    pub reactivate_after: Option<Duration>,
    
    pub activate_on: Vec<String>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // Важно: пробует десериализовать по очереди
pub enum TouchArea {
    // Если есть только список зон, в TOML это будет: e1 = ["B1", "B8"]
    Simple(Vec<String>),
    // Если есть доп. параметры, в TOML это будет: e1 = { activate_on = [...], ... }
    Full {
        activate_on: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        deactivate_after: Option<Duration>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reactivate_after: Option<Duration>,
    },
}

impl TouchArea {
    pub fn new(activate_on: Vec<String>, deactivate_after: Option<Duration>, reactivate_after: Option<Duration>) -> Self {
        Self::Full {
            activate_on,
            deactivate_after,
            reactivate_after,
        }
    }
}

impl DXTouchAreaSettings {
    pub fn new(activate_on: Vec<String>, deactivate_after: Option<Duration>, reactivate_after: Option<Duration>) -> Self {
        Self {
            deactivate_after,
            reactivate_after,
            activate_on,
        }
    }
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
            p1_dx_touch_mapping: DXTouchAreaMapping::default(),
            p2_dx_touch_mapping: DXTouchAreaMapping::default(),
        }
    }
}
