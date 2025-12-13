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
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            enabled: false,
            port: "COM24".to_string(),
            device_file: Some("./device.txt".to_string()),
        }
    }
}
