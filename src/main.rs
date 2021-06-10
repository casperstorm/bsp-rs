use bevy::pbr::AmbientLight;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy::render::wireframe::WireframePlugin;
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
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.5 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(UiPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(BspPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bsp = asset_server.load("maps/de_dust2.bsp#Map");
    commands.spawn_scene(bsp);

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
            ..Default::default()
        });

    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
}
