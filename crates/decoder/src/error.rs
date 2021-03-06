use thiserror::Error;

use crate::BspVersion;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("Invalid or Unimplemented bsp identifier: {ident}")]
    InvalidOrUnimplementedIdent { ident: i32 },
    #[error("Invalid BspFormat used to decode file of version: {version:?}")]
    InvalidBspFormat { version: BspVersion },
    #[error("Invalid Wad format, must be `WAD2` or `WAD3`")]
    InvalidWadFormat,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
