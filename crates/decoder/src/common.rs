use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;

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

pub(crate) fn read_vec3<R: Read + Seek>(reader: &mut R) -> Result<Vec3> {
    let x = reader.read_f32::<LittleEndian>()?;
    let y = reader.read_f32::<LittleEndian>()?;
    let z = reader.read_f32::<LittleEndian>()?;

    Ok(Vec3::new(x, y, z))
}
