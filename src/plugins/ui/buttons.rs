use bevy::prelude::*;
use bevy_bsp::BspConfig;

use crate::{AppState, Event};

pub struct ButtonsPlugin;

pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

impl Plugin for ButtonsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_startup_system(spawn.system())
            .add_system(button_system.system());
    }
}

#[derive(Debug, PartialEq)]
enum ButtonType {
    WireFrame,
    NextMap,
    PreviousMap,
}

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(15.0),
                    right: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Enable".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/iosevka-regular.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                        TextSection {
                            value: "\nWireframe".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/iosevka-regular.ttf"),
                                font_size: 18.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        },
                    ],
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        })
        .insert(ButtonType::WireFrame);

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(15.0),
                    right: Val::Px(15.0 + 150.0 + 15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Next Map".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        })
        .insert(ButtonType::NextMap);

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(15.0),
                    right: Val::Px(15.0 + 150.0 * 2.0 + 15.0 * 2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Prev Map".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/iosevka-regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        })
        .insert(ButtonType::PreviousMap);
}

#[allow(clippy::type_complexity)]
fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &Children,
            &self::ButtonType,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut bsp_config: ResMut<BspConfig>,
    mut events: EventWriter<Event>,
    state: Res<AppState>,
    scene_spawner: Res<SceneSpawner>,
) {
    for (interaction, mut material, children, button_type) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();

        if &ButtonType::WireFrame == button_type {
            let value = if bsp_config.show_wireframe {
                "Disable"
            } else {
                "Enable"
            };

            text.sections[0].value = value.to_string();
        }

        // Prevent buttons from being clicked if map hasn't spawned fully yet (prevent race condition)
        if &ButtonType::NextMap == button_type || &ButtonType::PreviousMap == button_type {
            if let Some(idx) = state.current_map {
                if let Some(map) = state.maps.get(idx) {
                    if let Some(instance_id) = map.instance_id {
                        if !scene_spawner.instance_is_ready(instance_id) {
                            return;
                        }
                    }
                }
            }
        }

        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();

                match button_type {
                    ButtonType::WireFrame => {
                        bsp_config.show_wireframe = !bsp_config.show_wireframe;
                    }
                    ButtonType::NextMap => {
                        let current_map = state.current_map.unwrap_or_default();

                        let new_map = (current_map + 1) % state.maps.len();

                        events.send(Event::LoadMap(new_map));
                    }
                    ButtonType::PreviousMap => {
                        let current_map = state.current_map.unwrap_or_default();

                        let new_map = current_map.wrapping_sub(1).min(state.maps.len() - 1);

                        events.send(Event::LoadMap(new_map));
                    }
                }
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}
