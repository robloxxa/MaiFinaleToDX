use std::io;
use thiserror::Error;

use crate::runtime::ModuleName;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("toml serialization error: {0}")]
    TomlSerialization(#[from] toml_edit::ser::Error),

    #[error("toml deserialization error: {0}")]
    TomlDeserialization(#[from] toml_edit::de::Error),

    #[error("finale area error: {0}")]
    FinaleArea(String),
    
    #[error("module is disabled: {0}")]
    ModuleDisabled(ModuleName),
    
    #[error("module is stopped")]
    ModuleStopped,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
