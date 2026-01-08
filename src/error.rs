use flexi_logger::FlexiLoggerError;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("logger error: {0}")]
    LoggerError(#[from] FlexiLoggerError),
    
    #[error("toml serialization error: {0}")]
    TomlSerializationError(#[from] toml_edit::ser::Error),
    
    #[error("toml deserialization error: {0}")]
    TomlDeserializationError(#[from] toml_edit::de::Error),

    #[error("finale area error: {0}")]
    FinaleAreaError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
