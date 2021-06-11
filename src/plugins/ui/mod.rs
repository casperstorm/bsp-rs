use bevy::prelude::*;

pub mod buttons;
pub mod info;

use buttons::ButtonsPlugin;
use info::InfoPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(InfoPlugin).add_plugin(ButtonsPlugin);
    }
}
