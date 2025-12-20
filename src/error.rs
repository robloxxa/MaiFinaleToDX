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

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
