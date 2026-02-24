use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::config::touch::finale;
use maifinale_macros::AreaMapping;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DxTouch {
    pub enabled: bool,

    #[serde(default)]
    pub mode: super::TouchMode,

    pub p1_port: String,
    pub p2_port: String,

    #[serde(default)]
    pub p1_mapping: AreaMapping,

    #[serde(default)]
    pub p2_mapping: AreaMapping,
}

impl Default for DxTouch {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: super::TouchMode::default(),
            p1_port: "COM6".to_string(),
            p2_port: "COM8".to_string(),
            p1_mapping: AreaMapping::default(),
            p2_mapping: AreaMapping::default(),
        }
    }
}

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

impl AreaMapping {
    pub fn fields_mut(&mut self, zone: &super::ZoneId) -> Vec<&mut Area> {
        match zone {
            super::ZoneId::A(n) => match n {
                1 => vec![&mut self.a1],
                2 => vec![&mut self.a2],
                3 => vec![&mut self.a3],
                4 => vec![&mut self.a4],
                5 => vec![&mut self.a5],
                6 => vec![&mut self.a6],
                7 => vec![&mut self.a7],
                8 => vec![&mut self.a8],
                _ => vec![],
            },
            super::ZoneId::B(n) => match n {
                1 => vec![&mut self.b1],
                2 => vec![&mut self.b2],
                3 => vec![&mut self.b3],
                4 => vec![&mut self.b4],
                5 => vec![&mut self.b5],
                6 => vec![&mut self.b6],
                7 => vec![&mut self.b7],
                8 => vec![&mut self.b8],
                _ => vec![],
            },
            super::ZoneId::C => vec![&mut self.c1, &mut self.c2],
            super::ZoneId::D(n) => match n {
                1 => vec![&mut self.d1],
                2 => vec![&mut self.d2],
                3 => vec![&mut self.d3],
                4 => vec![&mut self.d4],
                5 => vec![&mut self.d5],
                6 => vec![&mut self.d6],
                7 => vec![&mut self.d7],
                8 => vec![&mut self.d8],
                _ => vec![],
            },
            super::ZoneId::E(n) => match n {
                1 => vec![&mut self.e1],
                2 => vec![&mut self.e2],
                3 => vec![&mut self.e3],
                4 => vec![&mut self.e4],
                5 => vec![&mut self.e5],
                6 => vec![&mut self.e6],
                7 => vec![&mut self.e7],
                8 => vec![&mut self.e8],
                _ => vec![],
            },
        }
    }
}

#[derive(AreaMapping, Deserialize, Serialize, Debug, Clone)]
#[serde(from = "BTreeMap<String, Area>", rename_all = "UPPERCASE")]
pub struct AreaMapping {
    #[area(activate_on = [finale::A1])]
    pub a1: Area,
    #[area(activate_on = [finale::A2])]
    pub a2: Area,
    #[area(activate_on = [finale::A3])]
    pub a3: Area,
    #[area(activate_on = [finale::A4])]
    pub a4: Area,
    #[area(activate_on = [finale::A5])]
    pub a5: Area,
    #[area(activate_on = [finale::A6])]
    pub a6: Area,
    #[area(activate_on = [finale::A7])]
    pub a7: Area,
    #[area(activate_on = [finale::A8])]
    pub a8: Area,

    #[area(activate_on = [finale::B1])]
    pub b1: Area,
    #[area(activate_on = [finale::B2])]
    pub b2: Area,
    #[area(activate_on = [finale::B3])]
    pub b3: Area,
    #[area(activate_on = [finale::B4])]
    pub b4: Area,
    #[area(activate_on = [finale::B5])]
    pub b5: Area,
    #[area(activate_on = [finale::B6])]
    pub b6: Area,
    #[area(activate_on = [finale::B7])]
    pub b7: Area,
    #[area(activate_on = [finale::B8])]
    pub b8: Area,

    #[area(activate_on = [finale::B1, finale::B8])]
    pub e1: Area,
    #[area(activate_on = [finale::B1, finale::B2])]
    pub e2: Area,
    #[area(activate_on = [finale::B2, finale::B3])]
    pub e3: Area,
    #[area(activate_on = [finale::B3, finale::B4])]
    pub e4: Area,
    #[area(activate_on = [finale::B4, finale::B5])]
    pub e5: Area,
    #[area(activate_on = [finale::B5, finale::B6])]
    pub e6: Area,
    #[area(activate_on = [finale::B6, finale::B7])]
    pub e7: Area,
    #[area(activate_on = [finale::B7, finale::B8])]
    pub e8: Area,

    #[area(activate_on = [finale::A1, finale::A8])]
    pub d1: Area,
    #[area(activate_on = [finale::A1, finale::A2])]
    pub d2: Area,
    #[area(activate_on = [finale::A2, finale::A3])]
    pub d3: Area,
    #[area(activate_on = [finale::A3, finale::A4])]
    pub d4: Area,
    #[area(activate_on = [finale::A4, finale::A5])]
    pub d5: Area,
    #[area(activate_on = [finale::A5, finale::A6])]
    pub d6: Area,
    #[area(activate_on = [finale::A6, finale::A7])]
    pub d7: Area,
    #[area(activate_on = [finale::A7, finale::A8])]
    pub d8: Area,

    #[area(activate_on = [finale::C])]
    pub c1: Area,
    #[area(activate_on = [finale::C])]
    pub c2: Area,
}




