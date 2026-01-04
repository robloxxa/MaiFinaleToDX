use std::{collections::BTreeMap, ops::{Deref, DerefMut}, time::Duration };

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

use crate::error;

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
    pub a1: TouchArea,
    pub a2: TouchArea,
    pub a3: TouchArea,
    pub a4: TouchArea,
    pub a5: TouchArea,
    pub a6: TouchArea,
    pub a7: TouchArea,
    pub a8: TouchArea,

    pub b1: TouchArea,
    pub b2: TouchArea,
    pub b3: TouchArea,
    pub b4: TouchArea,
    pub b5: TouchArea,
    pub b6: TouchArea,
    pub b7: TouchArea,
    pub b8: TouchArea,

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
            a1: TouchArea::new(DXAreaPositions::A1, vec!["A1".to_string()], None, None),
            a2: TouchArea::new(DXAreaPositions::A2, vec!["A2".to_string()], None, None),
            a3: TouchArea::new(DXAreaPositions::A3, vec!["A3".to_string()], None, None),
            a4: TouchArea::new(DXAreaPositions::A4, vec!["A4".to_string()], None, None),
            a5: TouchArea::new(DXAreaPositions::A5, vec!["A5".to_string()], None, None),
            a6: TouchArea::new(DXAreaPositions::A6, vec!["A6".to_string()], None, None),
            a7: TouchArea::new(DXAreaPositions::A7, vec!["A7".to_string()], None, None),
            a8: TouchArea::new(DXAreaPositions::A8, vec!["A8".to_string()], None, None),

            b1: TouchArea::new(DXAreaPositions::B1, vec!["B1".to_string()], None, None),
            b2: TouchArea::new(DXAreaPositions::B2, vec!["B2".to_string()], None, None),
            b3: TouchArea::new(DXAreaPositions::B3, vec!["B3".to_string()], None, None),
            b4: TouchArea::new(DXAreaPositions::B4, vec!["B4".to_string()], None, None),
            b5: TouchArea::new(DXAreaPositions::B5, vec!["B5".to_string()], None, None),
            b6: TouchArea::new(DXAreaPositions::B6, vec!["B6".to_string()], None, None),
            b7: TouchArea::new(DXAreaPositions::B7, vec!["B7".to_string()], None, None),
            b8: TouchArea::new(DXAreaPositions::B8, vec!["B8".to_string()], None, None),

            e1: TouchArea::new(
                DXAreaPositions::E1,
                vec!["B1".to_string(), "B8".to_string()],
                None,
                None,
            ),
            e2: TouchArea::new(
                DXAreaPositions::E2,
                vec!["B1".to_string(), "B2".to_string()],
                None,
                None,
            ),
            e3: TouchArea::new(
                DXAreaPositions::E3,
                vec!["B2".to_string(), "B3".to_string()],
                None,
                None,
            ),
            e4: TouchArea::new(
                DXAreaPositions::E4,
                vec!["B3".to_string(), "B4".to_string()],
                None,
                None,
            ),
            e5: TouchArea::new(
                DXAreaPositions::E5,
                vec!["B4".to_string(), "B5".to_string()],
                None,
                None,
            ),
            e6: TouchArea::new(
                DXAreaPositions::E6,
                vec!["B5".to_string(), "B6".to_string()],
                None,
                None,
            ),
            e7: TouchArea::new(
                DXAreaPositions::E7,
                vec!["B6".to_string(), "B7".to_string()],
                None,
                None,
            ),
            e8: TouchArea::new(
                DXAreaPositions::E8,
                vec!["B7".to_string(), "B8".to_string()],
                None,
                None,
            ),

            d1: TouchArea::new(
                DXAreaPositions::D1,
                vec!["A1".to_string(), "A8".to_string()],
                None,
                None,
            ),
            d2: TouchArea::new(
                DXAreaPositions::D2,
                vec!["A1".to_string(), "A2".to_string()],
                None,
                None,
            ),
            d3: TouchArea::new(
                DXAreaPositions::D3,
                vec!["A2".to_string(), "A3".to_string()],
                None,
                None,
            ),
            d4: TouchArea::new(
                DXAreaPositions::D4,
                vec!["A3".to_string(), "A4".to_string()],
                None,
                None,
            ),
            d5: TouchArea::new(
                DXAreaPositions::D5,
                vec!["A4".to_string(), "A5".to_string()],
                None,
                None,
            ),
            d6: TouchArea::new(
                DXAreaPositions::D6,
                vec!["A5".to_string(), "A6".to_string()],
                None,
                None,
            ),
            d7: TouchArea::new(
                DXAreaPositions::D7,
                vec!["A6".to_string(), "A7".to_string()],
                None,
                None,
            ),
            d8: TouchArea::new(
                DXAreaPositions::D8,
                vec!["A7".to_string(), "A8".to_string()],
                None,
                None,
            ),

            c1: TouchArea::new(DXAreaPositions::C1, vec!["C1".to_string()], None, None),
            c2: TouchArea::new(DXAreaPositions::C2, vec!["C1".to_string()], None, None),
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
pub struct TouchArea {
    pub position: usize,
    pub bit: u8,

    #[serde_as(as = "Option<DurationMilliSeconds>")]
    pub deactivate_after: Option<Duration>,
    #[serde_as(as = "Option<DurationMilliSeconds>")]
    pub reactivate_after: Option<Duration>,

    
    pub activate_on: FinaleAreaVec,
}

impl TouchArea {
    pub fn new(
        mapping: (usize, u8),
        activate_on: Vec<String>,
        deactivate_after: Option<Duration>,
        reactivate_after: Option<Duration>,
    ) -> Self {
        Self {
            position: mapping.0,
            bit: mapping.1,
            activate_on,
            deactivate_after,
            reactivate_after,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Threshold {
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

#[non_exhaustive]
struct DXAreaPositions;

impl DXAreaPositions {
    const A1: (usize, u8) = (1, 1);
    const A2: (usize, u8) = (1, 2);
    const A3: (usize, u8) = (1, 4);
    const A4: (usize, u8) = (1, 8);
    const A5: (usize, u8) = (1, 16);

    const A6: (usize, u8) = (2, 1);
    const A7: (usize, u8) = (2, 2);
    const A8: (usize, u8) = (2, 4);
    const B1: (usize, u8) = (2, 8);
    const B2: (usize, u8) = (2, 16);

    const B3: (usize, u8) = (3, 1);
    const B4: (usize, u8) = (3, 2);
    const B5: (usize, u8) = (3, 4);
    const B6: (usize, u8) = (3, 8);
    const B7: (usize, u8) = (3, 16);

    const B8: (usize, u8) = (4, 1);
    const C1: (usize, u8) = (4, 2);
    const C2: (usize, u8) = (4, 4);
    const D1: (usize, u8) = (4, 8);
    const D2: (usize, u8) = (4, 16);

    const D3: (usize, u8) = (5, 1);
    const D4: (usize, u8) = (5, 2);
    const D5: (usize, u8) = (5, 4);
    const D6: (usize, u8) = (5, 8);
    const D7: (usize, u8) = (5, 16);

    const D8: (usize, u8) = (6, 1);
    const E1: (usize, u8) = (6, 2);
    const E2: (usize, u8) = (6, 4);
    const E3: (usize, u8) = (6, 8);
    const E4: (usize, u8) = (6, 16);

    const E5: (usize, u8) = (7, 1);
    const E6: (usize, u8) = (7, 2);
    const E7: (usize, u8) = (7, 4);
    const E8: (usize, u8) = (7, 8);
}

#[derive(Serialize, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub struct FinaleAreaVec(pub Vec<FinaleArea>);

impl DerefMut for FinaleAreaVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for FinaleAreaVec {
    type Target = Vec<FinaleArea>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FinaleArea {
    pub position: usize,
    pub bit: u8,
}

impl FinaleArea {
    const A1: FinaleArea = FinaleArea::new(0, 0);
    const A2: FinaleArea = FinaleArea::new(0, 2);
    const A3: FinaleArea = FinaleArea::new(1, 0);
    const A4: FinaleArea = FinaleArea::new(1, 2);
    const A5: FinaleArea = FinaleArea::new(2, 0);
    const A6: FinaleArea = FinaleArea::new(2, 2);
    const A7: FinaleArea = FinaleArea::new(3, 0);
    const A8: FinaleArea = FinaleArea::new(3, 2);

    const B1: FinaleArea = FinaleArea::new(0, 1);
    const B2: FinaleArea = FinaleArea::new(0, 3);
    const B3: FinaleArea = FinaleArea::new(1, 1);
    const B4: FinaleArea = FinaleArea::new(1, 3);
    const B5: FinaleArea = FinaleArea::new(2, 1);
    const B6: FinaleArea = FinaleArea::new(2, 3);
    const B7: FinaleArea = FinaleArea::new(3, 1);
    const B8: FinaleArea = FinaleArea::new(3, 3);
    
    const C1: FinaleArea = FinaleArea::new(4, 4);
    
    const A1_NAME: &str = "A1";
    const A2_NAME: &str = "A2";
    const A3_NAME: &str = "A3";
    const A4_NAME: &str = "A4";
    const A5_NAME: &str = "A5";
    const A6_NAME: &str = "A6";
    const A7_NAME: &str = "A7";
    const A8_NAME: &str = "A8";

    const B1_NAME: &str = "B1";
    const B2_NAME: &str = "B2";
    const B3_NAME: &str = "B3";
    const B4_NAME: &str = "B4";
    const B5_NAME: &str = "B5";
    const B6_NAME: &str = "B6";
    const B7_NAME: &str = "B7";
    const B8_NAME: &str = "B8";

    const C1_NAME: &str = "C1";
    
    pub const fn new(position: usize, bit: u8) -> Self {
        FinaleArea { position, bit }
    }
}

impl TryFrom<&str> for FinaleArea {
    type Error = error::Error;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.as_ref() {
            Self::A1_NAME => Ok(Self::A1),
            Self::A2_NAME => Ok(Self::A2),
            Self::A3_NAME => Ok(Self::A3),
            Self::A4_NAME => Ok(Self::A4),
            Self::A5_NAME => Ok(Self::A5),
            Self::A6_NAME => Ok(Self::A6),
            Self::A7_NAME => Ok(Self::A7),
            Self::A8_NAME => Ok(Self::A8),
            Self::B1_NAME => Ok(Self::B1),
            Self::B2_NAME => Ok(Self::B2),
            Self::B3_NAME => Ok(Self::B3),
            Self::B4_NAME => Ok(Self::B4),
            Self::B5_NAME => Ok(Self::B5),
            Self::B6_NAME => Ok(Self::B6),
            Self::B7_NAME => Ok(Self::B7),
            Self::B8_NAME => Ok(Self::B8),
            Self::C1_NAME => Ok(Self::C1),
            v => Err(error::Error::FinaleAreaError(format!("Invalid Finale Area Name: {}", v))),
        }
    }
}

impl TryFrom<Vec<String>> for FinaleAreaVec {
    type Error = error::Error;
    
    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let vec = value.iter().map(|s| {
            FinaleArea::try_from(s.as_str())
        }).collect::<Result<Vec<_>, _>>()?;
        
        Ok(FinaleAreaVec(vec))
    }
}