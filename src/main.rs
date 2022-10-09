#![feature(let_chains)]

pub mod camera;
pub mod surrounds;
pub mod terrain;
pub mod tile;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::TilemapPlugin;

use camera::*;
use terrain::bevy_connect::setup_world;

fn main() {
    let _app = App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.7, 1.0)))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            title: "csagame".into(),
            width: 1280.0,
            height: 720.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_world)
        //.add_startup_system(setup)
        .add_system(camera::move_camera)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Entity
    let entity_sprite = asset_server.load("Entity.png");
    commands.spawn_bundle(SpriteBundle {
        texture: entity_sprite,
        transform: Transform {
            translation: Vec3::default(),
            rotation: Quat::default(),
            scale: Vec3::new(2.0, 2.0, 1.0),
        },
        ..Default::default()
    });
}
