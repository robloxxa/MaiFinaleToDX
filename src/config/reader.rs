use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum ReaderMode {
    #[default]
    Hardware,
    Emulated,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Reader {
    /// Enable NFC reader feature
    pub enabled: bool,

    #[serde(default)]
    pub mode: ReaderMode,

    /// COM Port for NFC reader
    ///
    /// Port of the real device.
    pub port: String,

    /// Device file for NFC reader
    pub device_file: Option<String>,

    /// List of Reader destinations.
    /// By default Finale Cabinet has two card readers which is 00 and 01.
    /// Do not change if you don't know what you are doing.
    /// Maximum 4 destinations supported.
    #[serde(default = "default_destinations")]
    pub destinations: ArrayVec<u8, 4>,

    pub init_retry_count: Option<i64>,
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: ReaderMode::default(),
            port: "COM24".to_string(),
            device_file: Some("./device.txt".to_string()),
            init_retry_count: None,
            destinations: default_destinations(),
        }
    }
}

fn default_destinations() -> ArrayVec<u8, 4> {
    let mut destinations = ArrayVec::new();
    destinations.push(0);
    destinations.push(1);
    destinations
}
