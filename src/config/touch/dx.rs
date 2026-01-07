use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::config::touch::finale;

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Area {
    #[serde(skip)]
    pub position: usize,

    #[serde(skip)]
    pub bit: u8,
    
    pub activate_on: finale::AreaVec,
    
    #[serde(default)]
    #[serde_as(as = "DurationMilliSeconds")]
    pub deactivate_after_ms: Duration,
    
    #[serde(default)]
    #[serde_as(as = "DurationMilliSeconds")]
    pub reactivate_after_ms: Duration,

}

impl Area {
    pub const fn new(
        mapping: (usize, u8),
        activate_on: Vec<finale::Area>,
        deactivate_after: Duration,
        reactivate_after: Duration,
    ) -> Self {
        Self {
            position: mapping.0,
            bit: mapping.1,
            activate_on: finale::AreaVec(activate_on),
            deactivate_after_ms: deactivate_after,
            reactivate_after_ms: reactivate_after,
        }
    }
    
    pub const fn set_pos(mut self, pos: (usize, u8)) -> Self {
        self.position = pos.0;
        self.bit = pos.1;
        
        self
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(from = "BTreeMap<String, Area>", rename_all = "UPPERCASE")]
pub struct AreaMapping {
    pub a1: Area,
    pub a2: Area,
    pub a3: Area,
    pub a4: Area,
    pub a5: Area,
    pub a6: Area,
    pub a7: Area,
    pub a8: Area,

    pub b1: Area,
    pub b2: Area,
    pub b3: Area,
    pub b4: Area,
    pub b5: Area,
    pub b6: Area,
    pub b7: Area,
    pub b8: Area,

    pub e1: Area,
    pub e2: Area,
    pub e3: Area,
    pub e4: Area,
    pub e5: Area,
    pub e6: Area,
    pub e7: Area,
    pub e8: Area,

    pub d1: Area,
    pub d2: Area,
    pub d3: Area,
    pub d4: Area,
    pub d5: Area,
    pub d6: Area,
    pub d7: Area,
    pub d8: Area,

    pub c1: Area,
    pub c2: Area,
}

impl AreaMapping {
    pub fn into_values(self) -> [Area; 34] {
        [
            self.a1, self.a2, self.a3, self.a4, self.a5, self.a6, self.a7, self.a8,
            self.b1, self.b2, self.b3, self.b4, self.b5, self.b6, self.b7, self.b8,
            self.e1, self.e2, self.e3, self.e4, self.e5, self.e6, self.e7, self.e8,
            self.d1, self.d2, self.d3, self.d4, self.d5, self.d6, self.d7, self.d8,
            self.c1, self.c2,
        ]
    }
}

impl From<BTreeMap<String, Area>> for AreaMapping {
    fn from(map: BTreeMap<String, Area>) -> Self {
        let mut mapping = Self::default();
        
        map.into_iter().for_each(|(key, area)| match key.as_str() {
            A1_NAME => mapping.a1 = area.set_pos(A1_POS),
            A2_NAME => mapping.a2 = area.set_pos(A2_POS),
            A3_NAME => mapping.a3 = area.set_pos(A3_POS),
            A4_NAME => mapping.a4 = area.set_pos(A4_POS),
            A5_NAME => mapping.a5 = area.set_pos(A5_POS),
            A6_NAME => mapping.a6 = area.set_pos(A6_POS),
            A7_NAME => mapping.a7 = area.set_pos(A7_POS),
            A8_NAME => mapping.a8 = area.set_pos(A8_POS),
            
            B1_NAME => mapping.b1 = area.set_pos(B1_POS),
            B2_NAME => mapping.b2 = area.set_pos(B2_POS),
            B3_NAME => mapping.b3 = area.set_pos(B3_POS),
            B4_NAME => mapping.b4 = area.set_pos(B4_POS),
            B5_NAME => mapping.b5 = area.set_pos(B5_POS),
            B6_NAME => mapping.b6 = area.set_pos(B6_POS),
            B7_NAME => mapping.b7 = area.set_pos(B7_POS),
            B8_NAME => mapping.b8 = area.set_pos(B8_POS),
            
            E1_NAME => mapping.e1 = area.set_pos(E1_POS),
            E2_NAME => mapping.e2 = area.set_pos(E2_POS),
            E3_NAME => mapping.e3 = area.set_pos(E3_POS),
            E4_NAME => mapping.e4 = area.set_pos(E4_POS),
            E5_NAME => mapping.e5 = area.set_pos(E5_POS),
            E6_NAME => mapping.e6 = area.set_pos(E6_POS),
            E7_NAME => mapping.e7 = area.set_pos(E7_POS),
            E8_NAME => mapping.e8 = area.set_pos(E8_POS),

            D1_NAME => mapping.d1 = area.set_pos(D1_POS),
            D2_NAME => mapping.d2 = area.set_pos(D2_POS),
            D3_NAME => mapping.d3 = area.set_pos(D3_POS),
            D4_NAME => mapping.d4 = area.set_pos(D4_POS),
            D5_NAME => mapping.d5 = area.set_pos(D5_POS),
            D6_NAME => mapping.d6 = area.set_pos(D6_POS),
            D7_NAME => mapping.d7 = area.set_pos(D7_POS),
            D8_NAME => mapping.d8 = area.set_pos(D8_POS),
            
            C1_NAME => mapping.c1 = area.set_pos(C1_POS),
            C2_NAME => mapping.c2 = area.set_pos(C2_POS),
            
            _ => {}
        });

        mapping
    }
}

impl Default for AreaMapping {
    fn default() -> Self {
        Self {
            a1: Area::new(A1_POS, vec![finale::A1], Duration::default(), Duration::default()),
            a2: Area::new(A2_POS, vec![finale::A2], Duration::default(), Duration::default()),
            a3: Area::new(A3_POS, vec![finale::A3], Duration::default(), Duration::default()),
            a4: Area::new(A4_POS, vec![finale::A4], Duration::default(), Duration::default()),
            a5: Area::new(A5_POS, vec![finale::A5], Duration::default(), Duration::default()),
            a6: Area::new(A6_POS, vec![finale::A6], Duration::default(), Duration::default()),
            a7: Area::new(A7_POS, vec![finale::A7], Duration::default(), Duration::default()),
            a8: Area::new(A8_POS, vec![finale::A8], Duration::default(), Duration::default()),

            b1: Area::new(B1_POS, vec![finale::B1], Duration::default(), Duration::default()),
            b2: Area::new(B2_POS, vec![finale::B2], Duration::default(), Duration::default()),
            b3: Area::new(B3_POS, vec![finale::B3], Duration::default(), Duration::default()),
            b4: Area::new(B4_POS, vec![finale::B4], Duration::default(), Duration::default()),
            b5: Area::new(B5_POS, vec![finale::B5], Duration::default(), Duration::default()),
            b6: Area::new(B6_POS, vec![finale::B6], Duration::default(), Duration::default()),
            b7: Area::new(B7_POS, vec![finale::B7], Duration::default(), Duration::default()),
            b8: Area::new(B8_POS, vec![finale::B8], Duration::default(), Duration::default()),

            e1: Area::new(E1_POS, vec![finale::B1, finale::B8], Duration::default(), Duration::default()),
            e2: Area::new(E2_POS, vec![finale::B1, finale::B2], Duration::default(), Duration::default()),
            e3: Area::new(E3_POS, vec![finale::B2, finale::B3], Duration::default(), Duration::default()),
            e4: Area::new(E4_POS, vec![finale::B3, finale::B4], Duration::default(), Duration::default()),
            e5: Area::new(E5_POS, vec![finale::B4, finale::B5], Duration::default(), Duration::default()),
            e6: Area::new(E6_POS, vec![finale::B5, finale::B6], Duration::default(), Duration::default()),
            e7: Area::new(E7_POS, vec![finale::B6, finale::B7], Duration::default(), Duration::default()),
            e8: Area::new(E8_POS, vec![finale::B7, finale::B8], Duration::default(), Duration::default()),

            d1: Area::new(D1_POS, vec![finale::A1, finale::A8], Duration::default(), Duration::default()),
            d2: Area::new(D2_POS, vec![finale::A1, finale::A2], Duration::default(), Duration::default()),
            d3: Area::new(D3_POS, vec![finale::A2, finale::A3], Duration::default(), Duration::default()),
            d4: Area::new(D4_POS, vec![finale::A3, finale::A4], Duration::default(), Duration::default()),
            d5: Area::new(D5_POS, vec![finale::A4, finale::A5], Duration::default(), Duration::default()),
            d6: Area::new(D6_POS, vec![finale::A5, finale::A6], Duration::default(), Duration::default()),
            d7: Area::new(D7_POS, vec![finale::A6, finale::A7], Duration::default(), Duration::default()),
            d8: Area::new(D8_POS, vec![finale::A7, finale::A8], Duration::default(), Duration::default()),

            c1: Area::new(C1_POS, vec![finale::C1], Duration::default(), Duration::default()),
            c2: Area::new(C2_POS, vec![finale::C1], Duration::default(), Duration::default()),
        }
    }
}

pub const A1_POS: (usize, u8) = (1, 1);
pub const A2_POS: (usize, u8) = (1, 2);
pub const A3_POS: (usize, u8) = (1, 4);
pub const A4_POS: (usize, u8) = (1, 8);
pub const A5_POS: (usize, u8) = (1, 16);
pub const A6_POS: (usize, u8) = (2, 1);
pub const A7_POS: (usize, u8) = (2, 2);
pub const A8_POS: (usize, u8) = (2, 4);

pub const B1_POS: (usize, u8) = (2, 8);
pub const B2_POS: (usize, u8) = (2, 16);
pub const B3_POS: (usize, u8) = (3, 1);
pub const B4_POS: (usize, u8) = (3, 2);
pub const B5_POS: (usize, u8) = (3, 4);
pub const B6_POS: (usize, u8) = (3, 8);
pub const B7_POS: (usize, u8) = (3, 16);
pub const B8_POS: (usize, u8) = (4, 1);

pub const D1_POS: (usize, u8) = (4, 8);
pub const D2_POS: (usize, u8) = (4, 16);
pub const D3_POS: (usize, u8) = (5, 1);
pub const D4_POS: (usize, u8) = (5, 2);
pub const D5_POS: (usize, u8) = (5, 4);
pub const D6_POS: (usize, u8) = (5, 8);
pub const D7_POS: (usize, u8) = (5, 16);
pub const D8_POS: (usize, u8) = (6, 1);

pub const E1_POS: (usize, u8) = (6, 2);
pub const E2_POS: (usize, u8) = (6, 4);
pub const E3_POS: (usize, u8) = (6, 8);
pub const E4_POS: (usize, u8) = (6, 16);
pub const E5_POS: (usize, u8) = (7, 1);
pub const E6_POS: (usize, u8) = (7, 2);
pub const E7_POS: (usize, u8) = (7, 4);
pub const E8_POS: (usize, u8) = (7, 8);

pub const C1_POS: (usize, u8) = (4, 2);
pub const C2_POS: (usize, u8) = (4, 4);

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

pub const D1_NAME: &str = "D1";
pub const D2_NAME: &str = "D2";
pub const D3_NAME: &str = "D3";
pub const D4_NAME: &str = "D4";
pub const D5_NAME: &str = "D5";
pub const D6_NAME: &str = "D6";
pub const D7_NAME: &str = "D7";
pub const D8_NAME: &str = "D8";

pub const E1_NAME: &str = "E1";
pub const E2_NAME: &str = "E2";
pub const E3_NAME: &str = "E3";
pub const E4_NAME: &str = "E4";
pub const E5_NAME: &str = "E5";
pub const E6_NAME: &str = "E6";
pub const E7_NAME: &str = "E7";
pub const E8_NAME: &str = "E8";

pub const C1_NAME: &str = "C1";
pub const C2_NAME: &str = "C2";
