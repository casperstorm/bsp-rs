use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};
use glam::{UVec2, Vec3};

use crate::Result;

pub(crate) fn read_array_f32<R: Read + Seek, const N: usize>(reader: &mut R) -> Result<[f32; N]> {
    let mut array = [0.0; N];

    for i in array.iter_mut() {
        *i = reader.read_f32::<LittleEndian>()?;
    }

    Ok(array)
}

pub(crate) fn read_array_i32<R: Read + Seek, const N: usize>(reader: &mut R) -> Result<[i32; N]> {
    let mut array = [0; N];

    for i in array.iter_mut() {
        *i = reader.read_i32::<LittleEndian>()?;
    }

    Ok(array)
}

pub(crate) fn read_array_i16<R: Read + Seek, const N: usize>(reader: &mut R) -> Result<[i16; N]> {
    let mut array = [0; N];

    for i in array.iter_mut() {
        *i = reader.read_i16::<LittleEndian>()?;
    }

    Ok(array)
}

pub(crate) fn read_array_u8<R: Read + Seek, const N: usize>(reader: &mut R) -> Result<[u8; N]> {
    let mut array = [0; N];

    for i in array.iter_mut() {
        *i = reader.read_u8()?;
    }

    Ok(array)
}
pub(crate) fn read_vec3<R: Read + Seek>(reader: &mut R) -> Result<Vec3> {
    let x = reader.read_f32::<LittleEndian>()?;
    let y = reader.read_f32::<LittleEndian>()?;
    let z = reader.read_f32::<LittleEndian>()?;

    Ok(Vec3::new(x, y, z))
}

pub(crate) fn read_uvec2_u16<R: Read + Seek>(reader: &mut R) -> Result<UVec2> {
    let x = reader.read_u16::<LittleEndian>()? as u32;
    let y = reader.read_u16::<LittleEndian>()? as u32;

    Ok(UVec2::new(x, y))
}
