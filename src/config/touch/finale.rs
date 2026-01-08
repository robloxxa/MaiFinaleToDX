use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct AreaVec(pub Vec<Area>);

impl TryFrom<Vec<String>> for AreaVec {
    type Error = error::Error;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let vec = value
            .iter()
            .map(|s| Area::try_from(s.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AreaVec(vec))
    }
}

impl From<AreaVec> for Vec<String> {
    fn from(value: AreaVec) -> Self {
        value.0.iter().map(|area| String::from(area.name)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Area {
    pub name: &'static str,
    pub position: usize,
    pub bit: u8,
}

impl Area {
    pub const fn new(name: &'static str, pos: (usize, u8)) -> Self {
        Area {
            name: name,
            position: pos.0,
            bit: pos.1,
        }
    }
}

impl TryFrom<&str> for Area {
    type Error = error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.as_ref() {
            A1_NAME => Ok(A1),
            A2_NAME => Ok(A2),
            A3_NAME => Ok(A3),
            A4_NAME => Ok(A4),
            A5_NAME => Ok(A5),
            A6_NAME => Ok(A6),
            A7_NAME => Ok(A7),
            A8_NAME => Ok(A8),
            B1_NAME => Ok(B1),
            B2_NAME => Ok(B2),
            B3_NAME => Ok(B3),
            B4_NAME => Ok(B4),
            B5_NAME => Ok(B5),
            B6_NAME => Ok(B6),
            B7_NAME => Ok(B7),
            B8_NAME => Ok(B8),
            C1_NAME => Ok(C1),
            v => Err(error::Error::FinaleAreaError(format!(
                "Invalid Finale Area Name: {}",
                v
            ))),
        }
    }
}

pub const A1: Area = Area::new(A1_NAME, A1_POS);
pub const A2: Area = Area::new(A2_NAME, A2_POS);
pub const A3: Area = Area::new(A3_NAME, A3_POS);
pub const A4: Area = Area::new(A4_NAME, A4_POS);
pub const A5: Area = Area::new(A5_NAME, A5_POS);
pub const A6: Area = Area::new(A6_NAME, A6_POS);
pub const A7: Area = Area::new(A7_NAME, A7_POS);
pub const A8: Area = Area::new(A8_NAME, A8_POS);

pub const B1: Area = Area::new(B1_NAME, B1_POS);
pub const B2: Area = Area::new(B2_NAME, B2_POS);
pub const B3: Area = Area::new(B3_NAME, B3_POS);
pub const B4: Area = Area::new(B4_NAME, B4_POS);
pub const B5: Area = Area::new(B5_NAME, B5_POS);
pub const B6: Area = Area::new(B6_NAME, B6_POS);
pub const B7: Area = Area::new(B7_NAME, B7_POS);
pub const B8: Area = Area::new(B8_NAME, B8_POS);

pub const C1: Area = Area::new(C1_NAME, C1_POS);

pub const A1_NAME: &str = "A1";
pub const A2_NAME: &str = "A2";
pub const A3_NAME: &str = "A3";
pub const A4_NAME: &str = "A4";
pub const A5_NAME: &str = "A5";
pub const A6_NAME: &str = "A6";
pub const A7_NAME: &str = "A7";
pub const A8_NAME: &str = "A8";

pub const B1_NAME: &str = "B1";
pub const B2_NAME: &str = "B2";
pub const B3_NAME: &str = "B3";
pub const B4_NAME: &str = "B4";
pub const B5_NAME: &str = "B5";
pub const B6_NAME: &str = "B6";
pub const B7_NAME: &str = "B7";
pub const B8_NAME: &str = "B8";

pub const C1_NAME: &str = "C1";

pub const A1_POS: (usize, u8) = (0, 0);
pub const A2_POS: (usize, u8) = (0, 2);
pub const A3_POS: (usize, u8) = (1, 0);
pub const A4_POS: (usize, u8) = (1, 2);
pub const A5_POS: (usize, u8) = (2, 0);
pub const A6_POS: (usize, u8) = (2, 2);
pub const A7_POS: (usize, u8) = (3, 0);
pub const A8_POS: (usize, u8) = (3, 2);

pub const B1_POS: (usize, u8) = (0, 1);
pub const B2_POS: (usize, u8) = (0, 3);
pub const B3_POS: (usize, u8) = (1, 1);
pub const B4_POS: (usize, u8) = (1, 3);
pub const B5_POS: (usize, u8) = (2, 1);
pub const B6_POS: (usize, u8) = (2, 3);
pub const B7_POS: (usize, u8) = (3, 1);
pub const B8_POS: (usize, u8) = (3, 3);

pub const C1_POS: (usize, u8) = (3, 4);
