use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use decoder::BspFormat;

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
    loaded: bool,
}

fn setup(mut state: ResMut<State>, mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the inital map
    state.bsp_handle = asset_server.load("maps/de_dust.bsp");

    // https://github.com/mcpar-land/bevy_fly_camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(1792.0, -104.0, -900.0),
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

#[allow(clippy::unnecessary_unwrap)]
fn render_map(
    mut state: ResMut<State>,
    bsp_files: ResMut<Assets<BspFile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bsp_file = bsp_files.get(&state.bsp_handle);

    if bsp_file.is_some() && !state.loaded {
        let bsp_file = bsp_file.unwrap();
        info!("Map loaded: {:?}", bsp_file);

        state.loaded = true;

        match &bsp_file.0 {
            BspFormat::GoldSrc30(format) => {
                // Debug volumes
                for model in format.models[1..].iter() {
                    let mesh = Mesh::from(shape::Box {
                        max_z: model.maxs[0],
                        max_x: model.maxs[1],
                        max_y: model.maxs[2],
                        min_z: model.mins[0],
                        min_x: model.mins[1],
                        min_y: model.mins[2],
                    });

                    let x = (model.maxs[1] + model.mins[1]) / 2.0;
                    let y = (model.maxs[2] + model.mins[2]) / 2.0;
                    let z = (model.maxs[0] + model.mins[0]) / 2.0;

                    commands.spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(Color::rgb(0.0, 0.66, 1.0).into()),
                        transform: Transform::from_xyz(x, y, z),
                        ..Default::default()
                    });
                }
            }
        }
    }
}
