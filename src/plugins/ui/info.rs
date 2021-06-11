use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::AppState;

pub struct InfoPlugin;

struct FpsText;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(spawn.system())
            .add_system(text_update_system.system());
    }
}

pub fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position: Rect {
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS:".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-bold.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-regular.ttf"),
                            font_size: 60.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "\n".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-regular.ttf"),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-regular.ttf"),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsText);
}

fn text_update_system(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<FpsText>>,
    state: Res<AppState>,
) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }

        if let Some(idx) = state.current_map {
            if let Some(map) = state.maps.get(idx) {
                // Update current map name
                text.sections[3].value = map.name.clone();
            }
        }
    }
}
