#![allow(dead_code)]

use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::common::{read_array_f32, read_array_i32, read_vec3};
use crate::{BspFormat, Result};

const NUM_LUMPS: usize = 16;
const MAX_MAP_HULLS: usize = 4;
const MAX_MAP_MODELS: usize = 400;
const MAX_MAP_BRUSHES: usize = 4096;
const MAX_MAP_ENTITIES: usize = 1024;
const MAX_MAP_ENTSTRING: usize = 128 * 1024;
const MAX_MAP_PLANES: usize = 32767;
const MAX_MAP_NODES: usize = 32767;
const MAX_MAP_CLIPNODES: usize = 32767;
const MAX_MAP_LEAFS: usize = 8192;
const MAX_MAP_VERTS: usize = 65535;
const MAX_MAP_FACES: usize = 65535;
const MAX_MAP_MARKSURFACES: usize = 65535;
const MAX_MAP_TEXINFO: usize = 8192;
const MAX_MAP_EDGES: usize = 256000;
const MAX_MAP_SURFEDGES: usize = 512000;
const MAX_MAP_TEXTURES: usize = 512;
const MAX_MAP_MIPTEX: usize = 0x200000;
const MAX_MAP_LIGHTING: usize = 0x200000;
const MAX_MAP_VISIBILITY: usize = 0x200000;
const MAX_MAP_PORTALS: usize = 65536;

#[derive(Debug, Clone, Copy)]
pub struct GoldSrc30Bsp {
    header: Header,
    models: [Option<Model>; MAX_MAP_MODELS],
}

impl BspFormat for GoldSrc30Bsp {}

pub(crate) fn decode<R: Read + Seek>(reader: &mut R, ident: i32) -> Result<GoldSrc30Bsp> {
    let header = decode_header(reader, ident)?;
    let models = decode_models(reader, &header)?;

    Ok(GoldSrc30Bsp { header, models })
}

fn decode_header<R: Read + Seek>(reader: &mut R, ident: i32) -> Result<Header> {
    let mut lumps = [Default::default(); NUM_LUMPS];

    for lump in lumps.iter_mut() {
        *lump = decode_header_lump(reader)?;
    }

    Ok(Header { ident, lumps })
}

fn decode_header_lump<R: Read + Seek>(reader: &mut R) -> Result<HeaderLump> {
    let file_offset = reader.read_i32::<LittleEndian>()?;
    let len = reader.read_i32::<LittleEndian>()?;

    Ok(HeaderLump { file_offset, len })
}

fn decode_models<R: Read + Seek>(
    reader: &mut R,
    header: &Header,
) -> Result<[Option<Model>; MAX_MAP_MODELS]> {
    let lump = header.lumps[LumpType::Models as usize];

    reader.seek(SeekFrom::Start(lump.file_offset as u64))?;

    let num_models = lump.len as usize / size_of::<Model>();
    let mut models = [Default::default(); MAX_MAP_MODELS];

    for model in models[0..num_models].iter_mut() {
        *model = Some(Model {
            mins: read_array_f32(reader)?,
            maxs: read_array_f32(reader)?,
            origin: read_vec3(reader)?,
            idx_head_nodes: read_array_i32(reader)?,
            num_vis_leafs: reader.read_i32::<LittleEndian>()?,
            idx_first_face: reader.read_i32::<LittleEndian>()?,
            num_faces: reader.read_i32::<LittleEndian>()?,
        });
    }

    Ok(models)
}

#[derive(Debug, Clone, Copy)]
struct Header {
    ident: i32,
    lumps: [HeaderLump; NUM_LUMPS],
}

#[derive(Debug, Clone, Copy, Default)]
struct HeaderLump {
    file_offset: i32,
    len: i32,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum LumpType {
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

#[derive(Debug, Clone, Copy, Default)]
struct Model {
    mins: [f32; 3],
    maxs: [f32; 3],
    origin: glam::Vec3,
    idx_head_nodes: [i32; MAX_MAP_HULLS],
    num_vis_leafs: i32,
    idx_first_face: i32,
    num_faces: i32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_size() {
        let size = std::mem::size_of::<Header>();

        assert_eq!(size, 132);
    }
}
