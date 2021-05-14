use bevy::prelude::*;

pub mod fps;
use fps::FpsPlugin;
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FpsPlugin);
    }
}
