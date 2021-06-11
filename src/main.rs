use std::fs;

use bevy::pbr::AmbientLight;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy::render::wireframe::WireframePlugin;
use bevy::scene::InstanceId;
use bevy::wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions};
use bevy_bsp::BspPlugin;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

//use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use crate::plugins::ui::UiPlugin;

mod plugins;

fn main() {
    App::build()
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                // The Wireframe requires NonFillPolygonMode feature
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Bsp-rs".to_string(),
            vsync: false,
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.5 / 5.0f32,
        })
        .insert_resource(AppState::default())
        .add_event::<Event>()
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(UiPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(BspPlugin)
        .add_system(cursor_grab_system.system())
        .add_system(change_map_system.system())
        .add_startup_system(setup.system())
        .run();
}

#[derive(Debug, Clone, Default)]
struct AppState {
    scene_entity: Option<Entity>,
    maps: Vec<Map>,
    current_map: Option<usize>,
}

#[derive(Debug, Clone)]
struct Map {
    name: String,
    scene_handle: Option<Handle<Scene>>,
    instance_id: Option<InstanceId>,
}

fn setup(mut commands: Commands, mut state: ResMut<AppState>, mut events: EventWriter<Event>) {
    let mut maps = vec![];

    if let Ok(dir) = fs::read_dir("assets/maps") {
        for entry in dir.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bsp") {
                    //let scene_handle = asset_server.load(format!("maps/{}#Map", &name).as_str());

                    maps.push(Map {
                        name,
                        scene_handle: None,
                        instance_id: None,
                    });
                }
            }
        }
    }
    maps.sort_by_key(|m| m.name.clone());

    state.maps = maps;

    // Load first map
    if !state.maps.is_empty() {
        events.send(Event::LoadMap(0));
    }

    let perspective_projection = PerspectiveProjection {
        fov: 90.0,
        near: 0.1,
        far: 6000.0,
        ..Default::default()
    };

    //https://github.com/mcpar-land/bevy_fly_camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            perspective_projection,
            ..Default::default()
        })
        .insert(FlyCamera {
            max_speed: 500.0,
            accel: 1000.0,
            friction: 975.0,
            sensitivity: 10.0,
            enabled: false,
            ..Default::default()
        });

    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut query: Query<&mut FlyCamera>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Right) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);

        query.iter_mut().for_each(|mut fly| {
            fly.enabled = true;
        });
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);

        query.iter_mut().for_each(|mut fly| {
            fly.enabled = false;
        });
    }
}

enum Event {
    LoadMap(usize),
}

fn change_map_system(
    mut events: EventReader<Event>,
    mut state: ResMut<AppState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    for event in events.iter() {
        match event {
            Event::LoadMap(idx) => {
                // Despawn current scene
                if let Some(entity) = state.scene_entity.take() {
                    commands.entity(entity).despawn_recursive();
                }

                let scene_entity = if let Some(map) = state.maps.get_mut(*idx) {
                    // TODO: Load new scene. Check if map scene has already been loaded, otherwise load
                    let scene_handle = map.scene_handle.clone().unwrap_or_else(|| {
                        asset_server.load(format!("maps/{}#Map", &map.name).as_str())
                    });

                    map.scene_handle = Some(scene_handle.clone());

                    let scene_entity = commands.spawn().id();
                    let instance_id = scene_spawner.spawn_as_child(scene_handle, scene_entity);

                    map.instance_id = Some(instance_id);

                    scene_entity
                } else {
                    continue;
                };

                state.scene_entity = Some(scene_entity);
                state.current_map = Some(*idx);
            }
        }
    }
}
