use std::io::{Read, Seek};

use crate::{BspHeader, BspVersion, Result};

mod gold_src_30;

pub(crate) fn decode_header<R: Read + Seek>(
    reader: &mut R,
    ident: u32,
    version: BspVersion,
) -> Result<Box<dyn BspHeader>> {
    let header = match version {
        BspVersion::GoldSrc30 => Box::new(gold_src_30::decode_header(reader, ident)?),
    };

    Ok(header)
}
