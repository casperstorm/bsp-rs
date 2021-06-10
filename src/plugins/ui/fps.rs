use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_fly_camera::FlyCamera;

pub struct FpsPlugin;

struct FpsText;

impl Plugin for FpsPlugin {
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
                            font_size: 60.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "Coords:".to_string(),
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
    camera: Query<(&FlyCamera, &Transform)>,
) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }

        if let Some((_, transform)) = camera.iter().next() {
            let coords = transform.translation;
            text.sections[4].value = format!("({},{},{})", coords.x, coords.y, coords.z);
        }
    }
}
