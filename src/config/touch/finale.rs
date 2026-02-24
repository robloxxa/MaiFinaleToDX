use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FinaleTouch {
    pub enabled: bool,

    #[serde(default)]
    pub mode: super::TouchMode,

    pub port: String,

    pub init_retry_count: Option<i64>,

    #[serde(default)]
    pub p1_threshold: Threshold,

    #[serde(default)]
    pub p2_threshold: Threshold,
}

impl Default for FinaleTouch {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: super::TouchMode::default(),
            port: "COM23".to_string(),
            init_retry_count: None,
            p1_threshold: Threshold::default(),
            p2_threshold: Threshold::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
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

impl Threshold {
    pub fn field_mut(&mut self, zone: &super::ZoneId) -> Option<&mut u8> {
        match zone {
            super::ZoneId::A(n) => match n {
                1 => Some(&mut self.a1),
                2 => Some(&mut self.a2),
                3 => Some(&mut self.a3),
                4 => Some(&mut self.a4),
                5 => Some(&mut self.a5),
                6 => Some(&mut self.a6),
                7 => Some(&mut self.a7),
                8 => Some(&mut self.a8),
                _ => None,
            },
            super::ZoneId::B(n) => match n {
                1 => Some(&mut self.b1),
                2 => Some(&mut self.b2),
                3 => Some(&mut self.b3),
                4 => Some(&mut self.b4),
                5 => Some(&mut self.b5),
                6 => Some(&mut self.b6),
                7 => Some(&mut self.b7),
                8 => Some(&mut self.b8),
                _ => None,
            },
            super::ZoneId::C => Some(&mut self.c),
            _ => None,
        }
    }
}

impl Default for Threshold {
    fn default() -> Self {
        Self {
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

macro_rules! define_finale_areas {
    ($( $name:ident, ($pos:expr, $bit:expr) );* $(;)?) => {
        $(
            pub const $name: Area = Area::new(stringify!($name), ($pos, $bit));
        )*

        impl TryFrom<&str> for Area {
            type Error = error::Error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $( stringify!($name) => Ok($name), )*
                    v => Err(error::Error::FinaleArea(format!(
                        "Invalid Finale Area Name: {}", v
                    ))),
                }
            }
        }
    };
}

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
            name,
            position: pos.0,
            bit: pos.1,
        }
    }
}

define_finale_areas! {
    A1, (0, 0);
    A2, (0, 2);
    A3, (1, 0);
    A4, (1, 2);
    A5, (2, 0);
    A6, (2, 2);
    A7, (3, 0);
    A8, (3, 2);

    B1, (0, 1);
    B2, (0, 3);
    B3, (1, 1);
    B4, (1, 3);
    B5, (2, 1);
    B6, (2, 3);
    B7, (3, 1);
    B8, (3, 3);

    C,  (3, 4);
}
