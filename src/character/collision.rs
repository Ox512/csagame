use bevy::{math::Vec4Swizzles, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::terrain::bevy_connect::TilemapLayer;

// World Collision Detection System:
// This system handles collisions between the world and
// characters. This works by only enabling colliders around
// certain entities.

// radius around WorldCollider objects in which colliders
// should be enabled
pub const ACTIVE_RADIUS: isize = 2;

// Any object that is expected to be involed in collisions
// and physics interactions with the world
// Eg: Enemies, NPCs
#[derive(Component, Default)]
pub struct WorldCollider {
    // A list of all colliders enabled by this entity
    pub enabled_colliders: Vec<Entity>,
}

// Update the current active colliders
pub fn update_colliders(
    mut commands: Commands,
    mut col_query: Query<(&mut WorldCollider, &Transform)>,
    mut tm_query: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TileStorage,
        &TilemapLayer,
        &Transform,
    )>,
    tile_query: Query<&Transform, With<TilePos>>,
    col_tile_query: Query<&Collider, With<TilePos>>,
) {
    // We only work with the Foreground layer
    let (tm_size, tm_grid_size, tm_storage, tm_layer, tm_transform) =
        tm_query.iter_mut().next().expect("No foreground tilemap");

    assert!(tm_layer.0 == 0);

    for (mut col, transform) in col_query.iter_mut() {
        let pos = {
            let world_pos = (tm_transform.compute_matrix().inverse()
                * Vec4::from((transform.translation, 1.0)))
            .xy();

            if let Some(pos) =
                TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
            {
                pos
            } else {
                // Entity out of range of the world tilemap
                // Skip to next
                continue;
            }
        };

        // Store all the entities that now have a collider
        let mut colliders = Vec::new();

        for x in (pos.x as isize - ACTIVE_RADIUS)..=(pos.x as isize + ACTIVE_RADIUS) {
            for y in (pos.y as isize - ACTIVE_RADIUS)..=(pos.y as isize + ACTIVE_RADIUS) {
                // Ensure the position contains only +ve values
                let pos = {
                    let mut pos = TilePos::new(0, 0);
                    if x > 0 {
                        pos.x = x as u32;
                    }

                    if y > 0 {
                        pos.y = y as u32;
                    }

                    pos
                };

                if let Some(tile_entity) = tm_storage.checked_get(&pos) {
                    // Add a collider if there isn't already one
                    if col_tile_query.get_component::<Collider>(tile_entity).is_err() {
                        let tile_transform = tile_query
                            .get_component::<Transform>(tile_entity)
                            .expect("Tile without transform!");

                        let tile = commands.get_entity(tile_entity);

                        if let Some(mut tile) = tile {
                            tile
                                .insert(Collider::cuboid(4.0, 4.0))
                                .insert(TransformBundle::from(*tile_transform));
                        }
                    }

                    // Entity now either has or already had a collider
                    // This can now be added to this WorldCollider's list
                    colliders.push(tile_entity);
                }
            }
        }

        // Remove all the colliders that are
        // no longer in range of this Character
        for c in &col.enabled_colliders {
            if !colliders.contains(c) {
                let tile = commands.get_entity(*c);

                if let Some(mut tile) = tile {
                    tile.remove::<Collider>();
                }
            }
        }

        col.enabled_colliders = colliders;
    }
}
