use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

pub(crate) mod common;
mod error;
pub mod format;
pub mod wad;

pub use error::Error;

use self::format::{gold_src_30, GoldSrc30Bsp};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct BspDecoder<R: Read + Seek> {
    reader: R,
    ident: i32,
    version: BspVersion,
}

impl<R: Read + Seek> BspDecoder<R> {
    pub fn from_reader(mut reader: R) -> Result<Self> {
        let ident = reader.read_i32::<LittleEndian>()?;

        let version = BspVersion::from_ident(ident)?;

        Ok(BspDecoder {
            reader,
            ident,
            version,
        })
    }

    pub fn version(&self) -> BspVersion {
        self.version
    }

    pub fn decode_any(mut self) -> Result<BspFormat> {
        let version = self.version;

        format::decode(&mut self.reader, self.ident, version)
    }

    pub fn decode_gold_src_30(&mut self) -> Result<GoldSrc30Bsp> {
        if self.version != BspVersion::GoldSrc30 {
            Err(Error::InvalidBspFormat {
                version: self.version,
            })
        } else {
            gold_src_30::decode(&mut self.reader, self.ident)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BspVersion {
    GoldSrc30,
}

impl BspVersion {
    fn from_ident(ident: i32) -> Result<BspVersion> {
        match ident {
            30 => Ok(BspVersion::GoldSrc30),
            _ => Err(Error::InvalidOrUnimplementedIdent { ident }),
        }
    }
}

#[derive(Debug)]
pub enum BspFormat {
    GoldSrc30(GoldSrc30Bsp),
}

#[derive(Debug)]
pub struct WadDecoder<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> WadDecoder<R> {
    pub fn from_reader(reader: R) -> Self {
        WadDecoder { reader }
    }

    pub fn decode(mut self) -> Result<wad::Wad> {
        wad::decode(&mut self.reader)
    }
}

trait ByteDecoder {
    type Output: Copy;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self::Output>;
}
