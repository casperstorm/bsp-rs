#![allow(dead_code)]

use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::common::{read_array_f32, read_array_i16, read_array_i32, read_uvec2_u16, read_vec3};
use crate::{BspFormat, Error, Result};

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

#[derive(Clone)]
pub struct GoldSrc30Bsp {
    pub models: Vec<Model>,
    pub planes: Vec<Plane>,
    pub edges: Vec<Edge>,
    pub lighting: Vec<Lighting>,
    pub vertices: Vec<Vertex>,
    pub nodes: Vec<Node>,
}

impl fmt::Debug for GoldSrc30Bsp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GoldSrc30Bsp")
            .field("models", &format!("{} models", self.models.len()))
            .field("planes", &format!("{} planes", self.planes.len()))
            .field("edges", &format!("{} edges", self.edges.len()))
            .field("lighting", &format!("{} lighting", self.lighting.len()))
            .field("vertices", &format!("{} verices", self.vertices.len()))
            .field("nodes", &format!("{} nodes", self.nodes.len()))
            .finish()
    }
}

impl BspFormat for GoldSrc30Bsp {}

pub(crate) fn decode<R: Read + Seek>(reader: &mut R, ident: i32) -> Result<GoldSrc30Bsp> {
    let header = decode_header(reader, ident)?;
    let models = decode_lump::<Model, R>(reader, &header, LumpType::Models)?;
    let planes = decode_lump::<Plane, R>(reader, &header, LumpType::Planes)?;
    let edges = decode_lump::<Edge, R>(reader, &header, LumpType::Edges)?;
    let lighting = decode_lump::<Lighting, R>(reader, &header, LumpType::Lighting)?;
    let vertices = decode_lump::<Vertex, R>(reader, &header, LumpType::Vertices)?;
    let nodes = decode_lump::<Node, R>(reader, &header, LumpType::Nodes)?;

    Ok(GoldSrc30Bsp { models, planes, edges,  lighting, vertices, nodes })
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

fn decode_lump<L: Lump, R: Read + Seek>(
    reader: &mut R,
    header: &Header,
    lump_type: LumpType,
) -> Result<Vec<L::Output>> {
    let lump = header.lumps[lump_type as usize];

    reader.seek(SeekFrom::Start(lump.file_offset as u64))?;

    let num_items = lump.len as usize / size_of::<L>();
    let mut items = Vec::with_capacity(num_items);

    for _ in 0..num_items {
        items.push(L::decode(reader)?);
    }

    Ok(items)
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

trait Lump {
    type Output: Copy;

    fn decode<R: Read + Seek>(
        reader: &mut R,
    ) -> Result<Self::Output>;
}

#[derive(Debug, Clone, Copy)]
pub struct Model {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub origin: glam::Vec3,
    pub idx_head_nodes: [i32; MAX_MAP_HULLS],
    pub num_vis_leafs: i32,
    pub idx_first_face: i32,
    pub num_faces: i32,
}

impl Lump for Model {
    type Output = Model;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Model> {
        Ok(
            Model {
                mins: read_array_f32(reader)?,
                maxs: read_array_f32(reader)?,
                origin: read_vec3(reader)?,
                idx_head_nodes: read_array_i32(reader)?,
                num_vis_leafs: reader.read_i32::<LittleEndian>()?,
                idx_first_face: reader.read_i32::<LittleEndian>()?,
                num_faces: reader.read_i32::<LittleEndian>()?,
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub dist: f32,
    pub plane_type: PlaneType,
}

#[derive(Debug, Clone, Copy)]
pub enum PlaneType {
    X,
    Y,
    Z,
    AnyX,
    AnyY,
    AnyZ,
}

impl TryFrom<i32> for PlaneType {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(PlaneType::X),
            1 => Ok(PlaneType::Y),
            2 => Ok(PlaneType::Z),
            3 => Ok(PlaneType::AnyX),
            4 => Ok(PlaneType::AnyY),
            5 => Ok(PlaneType::AnyZ),
            _ => Err(Error::Custom(format!("{} not a valid plane type", value))),
        }
    }
}


impl Lump for Plane {
    type Output = Plane;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Plane> {
        Ok(
            Plane {
                normal: read_vec3(reader)?,
                dist: reader.read_f32::<LittleEndian>()?,
                plane_type: PlaneType::try_from(reader.read_i32::<LittleEndian>()?)?,
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub vertex: glam::UVec2,
}

impl Lump for Edge {
    type Output = Edge;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Edge> {
        Ok(
            Edge {
                vertex: read_uvec2_u16(reader)?,
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SurfEdge(pub i32);

impl Lump for SurfEdge {
    type Output = SurfEdge;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<SurfEdge> {
        Ok(
            SurfEdge(reader.read_i32::<LittleEndian>()?)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Lighting {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Lump for Lighting {
    type Output = Lighting;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Lighting> {
        Ok(
            Lighting{
                r: reader.read_u8()?,
                g: reader.read_u8()?,
                b: reader.read_u8()?,
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex(pub glam::Vec3);

impl Lump for Vertex {
    type Output = Vertex;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Vertex> {
        Ok(
            Vertex(read_vec3(reader)?)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub idx_plane: u32,
    pub idx_children: [i16; 2],
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub first_face: u16,
    pub num_faces: u16,
}


impl Lump for Node {
    type Output = Node;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Node> {
        Ok(
            Node {
                idx_plane: reader.read_u32::<LittleEndian>()?,
                idx_children: read_array_i16(reader)?,
                mins: read_array_i16(reader)?,
                maxs: read_array_i16(reader)?,
                first_face: reader.read_u16::<LittleEndian>()?,
                num_faces: reader.read_u16::<LittleEndian>()?,
            }
        )
    }
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
