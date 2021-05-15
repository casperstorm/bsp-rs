#![allow(dead_code)]

use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::common::{
    read_array_f32, read_array_i16, read_array_i32, read_array_u8, read_uvec2_u16, read_vec3,
};
use crate::{Error, Result};

const NUM_LUMPS: usize = 16;
const MAX_MAP_HULLS: usize = 4;

#[derive(Clone)]
pub struct GoldSrc30Bsp {
    pub models: Vec<Model>,
    pub planes: Vec<Plane>,
    pub edges: Vec<Edge>,
    pub surf_edges: Vec<SurfEdge>,
    pub lighting: Vec<Lighting>,
    pub vertices: Vec<Vertex>,
    pub nodes: Vec<Node>,
    pub leaves: Vec<Leaf>,
    pub visibility: Vec<Visibility>,
    pub texture_info: Vec<TextureInfo>,
    pub faces: Vec<Face>,
}

impl fmt::Debug for GoldSrc30Bsp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GoldSrc30Bsp")
            .field("models", &format!("{} models", self.models.len()))
            .field("planes", &format!("{} planes", self.planes.len()))
            .field("edges", &format!("{} edges", self.edges.len()))
            .field(
                "surf_edges",
                &format!("{} surf_edges", self.surf_edges.len()),
            )
            .field("lighting", &format!("{} lighting", self.lighting.len()))
            .field("vertices", &format!("{} verices", self.vertices.len()))
            .field("nodes", &format!("{} nodes", self.nodes.len()))
            .field("leaves", &format!("{} leaves", self.leaves.len()))
            .field(
                "visibility",
                &format!("{} visibility", self.visibility.len()),
            )
            .field(
                "texture_info",
                &format!("{} texture_info", self.texture_info.len()),
            )
            .field("faces", &format!("{} faces", self.faces.len()))
            .finish()
    }
}

pub(crate) fn decode<R: Read + Seek>(reader: &mut R, ident: i32) -> Result<GoldSrc30Bsp> {
    let header = decode_header(reader, ident)?;
    let planes = decode_lump::<Plane, R>(reader, &header, LumpType::Planes)?;
    let vertices = decode_lump::<Vertex, R>(reader, &header, LumpType::Vertices)?;
    let visibility = decode_lump::<Visibility, R>(reader, &header, LumpType::Visibility)?;
    let nodes = decode_lump::<Node, R>(reader, &header, LumpType::Nodes)?;
    let texture_info = decode_lump::<TextureInfo, R>(reader, &header, LumpType::Texinfo)?;
    let faces = decode_lump::<Face, R>(reader, &header, LumpType::Faces)?;
    let lighting = decode_lump::<Lighting, R>(reader, &header, LumpType::Lighting)?;
    let leaves = decode_lump::<Leaf, R>(reader, &header, LumpType::Leaves)?;
    let edges = decode_lump::<Edge, R>(reader, &header, LumpType::Edges)?;
    let surf_edges = decode_lump::<SurfEdge, R>(reader, &header, LumpType::Surfedges)?;
    let models = decode_lump::<Model, R>(reader, &header, LumpType::Models)?;

    Ok(GoldSrc30Bsp {
        models,
        planes,
        edges,
        surf_edges,
        lighting,
        vertices,
        nodes,
        leaves,
        visibility,
        texture_info,
        faces,
    })
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

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self::Output>;
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
        Ok(Model {
            mins: read_array_f32(reader)?,
            maxs: read_array_f32(reader)?,
            origin: read_vec3(reader)?,
            idx_head_nodes: read_array_i32(reader)?,
            num_vis_leafs: reader.read_i32::<LittleEndian>()?,
            idx_first_face: reader.read_i32::<LittleEndian>()?,
            num_faces: reader.read_i32::<LittleEndian>()?,
        })
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
        Ok(Plane {
            normal: read_vec3(reader)?,
            dist: reader.read_f32::<LittleEndian>()?,
            plane_type: PlaneType::try_from(reader.read_i32::<LittleEndian>()?)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub vertex: glam::UVec2,
}

impl Lump for Edge {
    type Output = Edge;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Edge> {
        Ok(Edge {
            vertex: read_uvec2_u16(reader)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SurfEdge(pub i32);

impl Lump for SurfEdge {
    type Output = SurfEdge;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<SurfEdge> {
        Ok(SurfEdge(reader.read_i32::<LittleEndian>()?))
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
        Ok(Lighting {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex(pub glam::Vec3);

impl Lump for Vertex {
    type Output = Vertex;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Vertex> {
        Ok(Vertex(read_vec3(reader)?))
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
        Ok(Node {
            idx_plane: reader.read_u32::<LittleEndian>()?,
            idx_children: read_array_i16(reader)?,
            mins: read_array_i16(reader)?,
            maxs: read_array_i16(reader)?,
            first_face: reader.read_u16::<LittleEndian>()?,
            num_faces: reader.read_u16::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Leaf {
    pub contents: Contents,
    pub vis_offset: i32,
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub idx_first_mark_surface: u16,
    pub num_mark_surfaces: u16,
    pub ambient_levels: [u8; 4],
}

#[derive(Debug, Clone, Copy)]
pub enum Contents {
    Empty,
    Solid,
    Water,
    Slime,
    Lava,
    Sky,
    Origin,
    Clip,
    Current0,
    Current90,
    Current180,
    Current270,
    CurrentUp,
    CurrentDown,
    Translucent,
}

impl TryFrom<i32> for Contents {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            -1 => Ok(Contents::Empty),
            -2 => Ok(Contents::Solid),
            -3 => Ok(Contents::Water),
            -4 => Ok(Contents::Slime),
            -5 => Ok(Contents::Lava),
            -6 => Ok(Contents::Sky),
            -7 => Ok(Contents::Origin),
            -8 => Ok(Contents::Clip),
            -9 => Ok(Contents::Current0),
            -10 => Ok(Contents::Current90),
            -11 => Ok(Contents::Current180),
            -12 => Ok(Contents::Current270),
            -13 => Ok(Contents::CurrentUp),
            -14 => Ok(Contents::CurrentDown),
            -15 => Ok(Contents::Translucent),
            _ => Err(Error::Custom(format!("{} not a valid content", value))),
        }
    }
}

impl Lump for Leaf {
    type Output = Leaf;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Leaf> {
        Ok(Leaf {
            contents: Contents::try_from(reader.read_i32::<LittleEndian>()?)?,
            vis_offset: reader.read_i32::<LittleEndian>()?,
            mins: read_array_i16(reader)?,
            maxs: read_array_i16(reader)?,
            idx_first_mark_surface: reader.read_u16::<LittleEndian>()?,
            num_mark_surfaces: reader.read_u16::<LittleEndian>()?,
            ambient_levels: read_array_u8(reader)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Visibility(pub u8);

impl Lump for Visibility {
    type Output = Visibility;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Visibility> {
        Ok(Visibility(reader.read_u8()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureInfo {
    pub s_vector: glam::Vec3,
    pub s_shift: f32,
    pub t_vector: glam::Vec3,
    pub t_shift: f32,
    pub idx_miptex: u32,
    pub flags: u32,
}

impl Lump for TextureInfo {
    type Output = TextureInfo;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<TextureInfo> {
        Ok(TextureInfo {
            s_vector: read_vec3(reader)?,
            s_shift: reader.read_f32::<LittleEndian>()?,
            t_vector: read_vec3(reader)?,
            t_shift: reader.read_f32::<LittleEndian>()?,
            idx_miptex: reader.read_u32::<LittleEndian>()?,
            flags: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub plane: u16,
    pub plane_side: u16,
    pub first_edge: u32,
    pub edges: u16,
    pub texture_info: u16,
    pub styles: [u8; 4],
    pub lightmap_offset: u32,
}

impl Lump for Face {
    type Output = Face;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Face> {
        Ok(Face {
            plane: reader.read_u16::<LittleEndian>()?,
            plane_side: reader.read_u16::<LittleEndian>()?,
            first_edge: reader.read_u32::<LittleEndian>()?,
            edges: reader.read_u16::<LittleEndian>()?,
            texture_info: reader.read_u16::<LittleEndian>()?,
            styles: read_array_u8(reader)?,
            lightmap_offset: reader.read_u32::<LittleEndian>()?,
        })
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
