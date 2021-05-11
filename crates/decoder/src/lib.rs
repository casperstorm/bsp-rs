use std::io::{Read, Seek};

use byteorder::ByteOrder;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Bsp {}

impl Bsp {
    pub fn from_reader<R: Read + Seek>(reader: R) -> Result<Self> {
        Ok(Bsp {})
    }
}
