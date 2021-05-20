use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

use crate::assets::{BspFile, BspFileLoader};
use crate::plugins::ui::UiPlugin;

mod assets;
mod plugins;

fn main() {
    App::build()
        .init_resource::<State>()
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_asset::<BspFile>()
        .init_asset_loader::<BspFileLoader>()
        .add_startup_system(setup.system())
        .add_system(render_map.system())
        .run();
}

#[derive(Default)]
struct State {
    bsp_handle: Handle<BspFile>,
    printed: bool,
}

fn setup(
    mut state: ResMut<State>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(0.0, 8.0, 0.0),
        ..Default::default()
    });

    state.bsp_handle = asset_server.load("maps/de_dust.bsp");

    // https://github.com/mcpar-land/bevy_fly_camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 4.0, 10.0),
            ..Default::default()
        })
        .insert(FlyCamera {
            pitch: 15.0,
            yaw: -0.0,
            ..Default::default()
        });

    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
}

fn render_map(mut state: ResMut<State>, bsp_files: ResMut<Assets<BspFile>>) {
    let bsp_file = bsp_files.get(&state.bsp_handle);

    if bsp_file.is_some() && !state.printed {
        info!("Map loaded: {:?}", bsp_file.unwrap());

        state.printed = true;
    }
}
