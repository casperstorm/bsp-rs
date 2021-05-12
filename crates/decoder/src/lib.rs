use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Bsp {
    pub header: Header,
}

impl Bsp {
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let header = decode_header(&mut reader)?;

        Ok(Bsp { header })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub ident: u32,
    pub version: u32,
    pub lumps: [HeaderLump; 64],
    pub map_revision: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HeaderLump {
    pub file_offset: u32,
    pub len: u32,
    pub version: u32,
    pub four_cc: [u8; 4],
}

fn decode_header<R: Read + Seek>(reader: &mut R) -> Result<Header> {
    let ident = reader.read_u32::<LittleEndian>()?;
    let version = reader.read_u32::<LittleEndian>()?;
    let lumps = [Default::default(); 64];
    let map_revision = reader.read_u32::<LittleEndian>()?;

    Ok(Header {
        ident,
        version,
        lumps,
        map_revision,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_size() {
        let size = std::mem::size_of::<Header>();

        assert_eq!(size, 1036);
    }
}
