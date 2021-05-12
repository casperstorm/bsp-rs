use std::fmt;
use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{BspHeader, Result};

const NUM_LUMPS: usize = 16;

pub(crate) fn decode_header<R: Read + Seek>(reader: &mut R, ident: u32) -> Result<GoldSrc30Header> {
    let mut lumps = [Default::default(); NUM_LUMPS];

    for lump in lumps.iter_mut() {
        *lump = decode_header_lump(reader)?;
    }

    Ok(GoldSrc30Header { ident, lumps })
}

fn decode_header_lump<R: Read + Seek>(reader: &mut R) -> Result<GoldSrc30HeaderLump> {
    let file_offset = reader.read_u32::<LittleEndian>()?;
    let len = reader.read_u32::<LittleEndian>()?;

    Ok(GoldSrc30HeaderLump { file_offset, len })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GoldSrc30Header {
    pub ident: u32,
    pub lumps: [GoldSrc30HeaderLump; 16],
}

impl BspHeader for GoldSrc30Header {}

impl fmt::Debug for GoldSrc30Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("ident", &self.ident)
            .field("lumps", &self.lumps)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoldSrc30HeaderLump {
    pub file_offset: u32,
    pub len: u32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_size() {
        let size = std::mem::size_of::<GoldSrc30Header>();

        assert_eq!(size, 132);
    }
}
