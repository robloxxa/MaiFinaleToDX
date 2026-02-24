use serde::{Deserialize, Serialize};

pub mod dx;
pub mod finale;

pub use dx::DxTouch;
pub use finale::FinaleTouch;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum TouchMode {
    #[default]
    Hardware,
    Emulated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneId {
    A(u8),
    B(u8),
    C,
    D(u8),
    E(u8),
}

impl std::fmt::Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZoneId::A(n) => write!(f, "A{}", n),
            ZoneId::B(n) => write!(f, "B{}", n),
            ZoneId::C => write!(f, "C"),
            ZoneId::D(n) => write!(f, "D{}", n),
            ZoneId::E(n) => write!(f, "E{}", n),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Touch {
    #[serde(default)]
    pub finale: FinaleTouch,

    #[serde(default)]
    pub dx: DxTouch,
}
