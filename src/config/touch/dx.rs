use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};
use std::collections::BTreeMap;
use std::time::Duration;

use crate::config::touch::finale;
use maifinale_macros::AreaMapping;

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




