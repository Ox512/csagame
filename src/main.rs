#![feature(let_chains)]
#![feature(variant_count)]

pub mod camera;
pub mod character;
pub mod player;
pub mod surrounds;
pub mod terrain;
pub mod tile;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

use camera::*;
use character::*;
use player::resolve_input;
use character::collision::*;
use terrain::bevy_connect::setup_world;

// Values that will later be changed during world creation
pub const CHUNK_COUNT: u32 = 1;
pub const WORLD_SIZE: (u32, u32) = (64 * CHUNK_COUNT, 64);
pub const WORLD_SEED: &str = "7";

fn main() {
    let _app = App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.7, 1.0)))
        .insert_resource(CursorPos(Vec2::new(f32::INFINITY, f32::INFINITY)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "csagame".into(),
                        width: 1280.0,
                        height: 720.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TilemapPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(8.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_camera)
        .add_startup_system(setup_world)
        .add_startup_system(setup_entity)
        .add_system(move_camera)
        .add_system(update_cursor_pos)
        .add_system(resolve_input)
        .add_system(update_colliders)
        .run();
}
