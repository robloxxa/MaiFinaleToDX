use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Reader {
    /// Enable NFC reader feature
    pub enabled: bool,

    /// COM Port for NFC reader
    ///
    /// Port of the real device.
    pub port: String,

    /// Device file for NFC reader
    pub device_file: Option<String>,

    /// List of Reader destionations.
    /// By default Finale Cabinet has two card readers which is 00 and 01.
    /// Do not change if you don't know what are you doing
    #[serde(default = "default_destinations")]
    pub destinations: Vec<u8>,

    pub init_retry_count: Option<i64>,
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            enabled: false,
            port: "COM24".to_string(),
            device_file: Some("./device.txt".to_string()),
            init_retry_count: None,
            destinations: vec![00, 01],
        }
    }
}

fn default_destinations() -> Vec<u8> {
    vec![00, 01]
}
