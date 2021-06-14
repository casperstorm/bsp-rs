use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::common::*;
use crate::format::gold_src_30::{Texture, MAXTEXTURENAME};
use crate::{ByteDecoder, Error, Result};

#[derive(Clone, Debug)]
pub struct Wad {
    pub textures: HashMap<String, Texture>,
}

pub(crate) fn decode<R: Read + Seek>(reader: &mut R) -> Result<Wad> {
    let header = Header::decode(reader)?;

    let version = String::from_utf8_lossy(&header.ident).to_string();
    if !["WAD2", "WAD3"].contains(&version.as_str()) {
        return Err(Error::InvalidWadFormat);
    }

    let mut dirs = Vec::with_capacity(header.num_dirs as usize);
    reader.seek(SeekFrom::Start(header.dir_offset as u64))?;

    for _ in 0..header.num_dirs {
        let entry = DirEntry::decode(reader)?;

        dirs.push(entry);
    }

    let mut textures = HashMap::new();

    for entry in dirs.iter() {
        let offset = entry.offset as usize;

        if entry.compression || entry.size != entry.disk_size || entry.kind != 0x43 {
            continue;
        }

        reader.seek(SeekFrom::Start(offset as u64))?;
        if let Ok(texture) = Texture::decode(reader, offset) {
            let name = String::from_utf8_lossy(&entry.name).to_string();
            let name = name.split('\0').next().unwrap_or_default().to_string();

            textures.insert(name, texture);
        }
    }

    Ok(Wad { textures })
}

#[derive(Debug, Clone, Copy)]
struct Header {
    ident: [u8; 4],
    num_dirs: i32,
    dir_offset: i32,
}

impl ByteDecoder for Header {
    type Output = Header;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self::Output> {
        Ok(Self {
            ident: read_array_u8(reader)?,
            num_dirs: reader.read_i32::<LittleEndian>()?,
            dir_offset: reader.read_i32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct DirEntry {
    offset: i32,
    disk_size: i32,
    size: i32,
    kind: u8,
    compression: bool,
    dummy: i16,
    name: [u8; MAXTEXTURENAME],
}

impl ByteDecoder for DirEntry {
    type Output = DirEntry;

    fn decode<R: Read + Seek>(reader: &mut R) -> Result<Self::Output> {
        Ok(Self {
            offset: reader.read_i32::<LittleEndian>()?,
            disk_size: reader.read_i32::<LittleEndian>()?,
            size: reader.read_i32::<LittleEndian>()?,
            kind: reader.read_u8()?,
            compression: reader.read_u8()? > 0,
            dummy: reader.read_i16::<LittleEndian>()?,
            name: read_array_u8(reader)?,
        })
    }
}
