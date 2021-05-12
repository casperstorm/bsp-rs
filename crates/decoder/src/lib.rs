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
#[derive(Debug)]
pub struct Header {
    pub ident: u32,
    pub version: u32,
    pub lumps: Vec<HeaderLump>,
    pub map_revision: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct HeaderLump {
    pub file_offset: u32,
    pub len: u32,
    pub version: u32,
    pub four_cc: [u8; 4],
}

fn decode_header<R: Read + Seek>(reader: &mut R) -> Result<Header> {
    let ident = reader.read_u32::<LittleEndian>()?;

    let version = reader.read_u32::<LittleEndian>()?;

    let lumps = vec![];

    let map_revision = reader.read_u32::<LittleEndian>()?;

    Ok(Header {
        ident,
        version,
        lumps,
        map_revision,
    })
}
