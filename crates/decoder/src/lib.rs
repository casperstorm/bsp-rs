use std::fmt;
use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

mod error;
mod format;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Bsp {
    pub version: BspVersion,
    pub header: Box<dyn BspHeader>,
}

impl Bsp {
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let ident = reader.read_u32::<LittleEndian>()?;

        let version = BspVersion::from_ident(ident)?;

        let header = format::decode_header(&mut reader, ident, version)?;

        Ok(Bsp { version, header })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BspVersion {
    GoldSrc30,
}

impl BspVersion {
    fn from_ident(ident: u32) -> Result<BspVersion> {
        match ident {
            30 => Ok(BspVersion::GoldSrc30),
            _ => Err(Error::InvalidOrUnimplementedIdent { ident }),
        }
    }
}

pub trait BspHeader: fmt::Debug {}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum LumpType {
    Entities,
    Planes,
    Texdata,
    Vertexes,
    Visibility,
    Nodes,
    Texinfo,
    Faces,
    Lighting,
    Occlusion,
    Leafs,
    Faceids,
    Edges,
    Surfedges,
    Models,
    Worldlights,
    Leaffaces,
    Leafbrushes,
    Brushes,
    Brushsides,
    Areas,
    Areaportals,
    Portals,
    Unused0,
    Propcollision,
    Clusters,
    Unused1,
    Prophulls,
    Portalverts,
    Unused2,
    Prophullverts,
    Clusterportals,
    Unused3,
    Proptris,
    Dispinfo,
    Originalfaces,
    Physdisp,
    Physcollide,
    Vertnormals,
    Vertnormalindices,
    DispLightmapAlphas,
    DispVerts,
    DispLightmapSamplePositions,
    GameLump,
    Leafwaterdata,
    Primitives,
    Primverts,
    Primindices,
    Pakfile,
    Clipportalverts,
    Cubemaps,
    TexdataStringData,
    TexdataStringTable,
    Overlays,
    Leafmindisttowater,
    FaceMacroTextureInfo,
    DispTris,
    Physcollidesurface,
    PropBlob,
    Wateroverlays,
    Lightmappages,
    LeafAmbientIndexHdr,
    Lightmappageinfos,
    LeafAmbientIndex,
    LightingHdr,
    WorldlightsHdr,
    LeafAmbientLightingHdr,
    LeafAmbientLighting,
    Xzippakfile,
    FacesHdr,
    MapFlags,
    OverlayFades,
    OverlaySystemLevels,
    Physlevel,
    DispMultiblend,
}
