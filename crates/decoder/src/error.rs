use thiserror::Error;

use crate::BspVersion;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid or Unimplemented bsp identifier: {ident}")]
    InvalidOrUnimplementedIdent { ident: u32 },
    #[error("Invalid BspFormat used to decode file of version: {version:?}")]
    InvalidBspFormat { version: BspVersion },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
