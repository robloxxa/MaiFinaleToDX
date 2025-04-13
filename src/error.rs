use flexi_logger::FlexiLoggerError;
use std::io;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    LoggerError(#[from] FlexiLoggerError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// #[derive(Error, Debug)]
// pub enum JvsError {
//
// }
