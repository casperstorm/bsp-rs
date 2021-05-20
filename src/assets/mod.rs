use std::io::Cursor;

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;

#[derive(Debug, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct BspFile(pub decoder::BspFormat);

#[derive(Default)]
pub struct BspFileLoader;

impl AssetLoader for BspFileLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let cursor = Cursor::new(bytes);
            let decoder = decoder::BspDecoder::from_reader(cursor)?;
            let format = decoder.decode_any()?;

            load_context.set_default_asset(LoadedAsset::new(BspFile(format)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bsp"]
    }
}
