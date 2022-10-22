#![feature(let_chains)]
#![feature(variant_count)]

pub mod camera;
pub mod surrounds;
pub mod terrain;
pub mod tile;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::TilemapPlugin;

use camera::*;
use terrain::bevy_connect::setup_world;

// Values that will later be changed during world creation
pub const CHUNK_COUNT: u32 = 20;
pub const WORLD_SIZE: (u32, u32) = (128 * CHUNK_COUNT, 128);
pub const WORLD_SEED: &str = "7";

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