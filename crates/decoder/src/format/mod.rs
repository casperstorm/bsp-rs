use std::io::{Read, Seek};

use crate::{BspFormat, BspVersion, Result};

pub mod gold_src_30;
pub use gold_src_30::GoldSrc30Bsp;

pub(crate) fn decode<R: Read + Seek>(
    reader: &mut R,
    ident: i32,
    version: BspVersion,
) -> Result<BspFormat> {
    let format = match version {
        BspVersion::GoldSrc30 => BspFormat::GoldSrc30(gold_src_30::decode(reader, ident)?),
    };

    Ok(format)
}
