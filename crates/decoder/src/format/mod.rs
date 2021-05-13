use std::any::Any;
use std::io::{Read, Seek};

use crate::{BspVersion, Result};

pub(crate) mod gold_src_30;
pub use gold_src_30::GoldSrc30Bsp;

pub(crate) fn decode<R: Read + Seek>(
    reader: &mut R,
    ident: i32,
    version: BspVersion,
) -> Result<Box<dyn Any>> {
    let format = match version {
        BspVersion::GoldSrc30 => Box::new(gold_src_30::decode(reader, ident)?),
    };

    Ok(format)
}
