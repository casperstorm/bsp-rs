use bevy::prelude::*;

pub mod fps;
pub mod load_map;

use fps::FpsPlugin;
use load_map::LoadMapPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FpsPlugin).add_plugin(LoadMapPlugin);
    }
}
