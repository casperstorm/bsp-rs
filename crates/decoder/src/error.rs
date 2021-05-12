use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid or Unimplemented bsp identifier: {ident}")]
    InvalidOrUnimplementedIdent { ident: u32 },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
