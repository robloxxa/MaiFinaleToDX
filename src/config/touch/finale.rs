use serde::{Deserialize, Serialize};

use crate::error;

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
