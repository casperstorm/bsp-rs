use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{BspFormat, Result};

const NUM_LUMPS: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct GoldSrc30Bsp {
    pub header: GoldSrc30Header,
}

impl BspFormat for GoldSrc30Bsp {}

pub(crate) fn decode<R: Read + Seek>(reader: &mut R, ident: u32) -> Result<GoldSrc30Bsp> {
    let mut lumps = [Default::default(); NUM_LUMPS];

    for lump in lumps.iter_mut() {
        *lump = decode_header_lump(reader)?;
    }

    let header = GoldSrc30Header { ident, lumps };

    Ok(GoldSrc30Bsp { header })
}

fn decode_header_lump<R: Read + Seek>(reader: &mut R) -> Result<GoldSrc30HeaderLump> {
    let file_offset = reader.read_u32::<LittleEndian>()?;
    let len = reader.read_u32::<LittleEndian>()?;

    Ok(GoldSrc30HeaderLump { file_offset, len })
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GoldSrc30Header {
    pub ident: u32,
    pub lumps: [GoldSrc30HeaderLump; 16],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoldSrc30HeaderLump {
    pub file_offset: u32,
    pub len: u32,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum GoldSrc30LumpType {
    Entities,
    Planes,
    Textures,
    Vertices,
    Visibility,
    Nodes,
    Texinfo,
    Faces,
    Lighting,
    Clipnodes,
    Leaves,
    Marksurfaces,
    Edges,
    Surfedges,
    Models,
    HeaderLumps,
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
