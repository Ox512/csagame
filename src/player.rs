use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::camera::CursorPos;
use crate::terrain::bevy_connect::TilemapLayer;
use crate::terrain::Terrain;
use crate::tile::*;

// Takes all the input from the player and then does
// whatever actions have been desired by the player
pub fn resolve_input(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    mut tm_query: Query<(
        Entity,
        &TilemapSize,
        &TilemapGridSize,
        &mut TileStorage,
        &TilemapLayer,
        &Transform,
    )>,
    mouse_input: Res<Input<MouseButton>>,
    mut terrain: ResMut<Terrain>,
) {
    // NOTE: tm_query is ordered by layer, which is very useful for ensuring that only
    //       the currently visible layer is acted upon, however this isn't guarenteed by
    //       and may break in later versions

    if mouse_input.just_pressed(MouseButton::Left) {
        // Remove a tile
        for (_entity, tm_size, tm_grid_size, mut tm_storage, tm_layer, tm_transform) in
            tm_query.iter_mut()
        {
            let world_pos = (tm_transform.compute_matrix().inverse()
                * Vec4::from((cursor_pos.0, 0.0, 1.0)))
            .xy();

            // If the cusor is at a valid position on the tilemap
            if let Some(tile_pos) =
                TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
            {
                if terrain.remove_tile(
                    &mut commands,
                    &mut tm_storage,
                    tm_layer.0,
                    tile_pos,
                ).is_some() {
                    // Break from the loop if a tile has been removed
                    break;
                }

            }
        }
    } else if mouse_input.just_pressed(MouseButton::Right) {
        // place a tile
        let (tm_entity, tm_size, tm_grid_size, mut tm_storage, tm_layer, tm_transform) =
            tm_query.iter_mut().next().expect("No foreground tilemap");

        let world_pos =
            (tm_transform.compute_matrix().inverse() * Vec4::from((cursor_pos.0, 0.0, 1.0))).xy();

        // If the cusor is at a valid position on the tilemap
        if let Some(tile_pos) =
            TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
        {
            if tm_storage.get(&tile_pos) == None {
                // Create a tile
                let tile = Tile::new(
                    TileId::Ground(Ground::Stone),
                    Some(
                        terrain.layers[tm_layer.0]
                            .get_surrounds(tile_pos.x, tile_pos.y)
                            .get_texture_offset(),
                    ),
                );

                terrain.insert_tile(
                    &mut commands,
                    &mut tm_storage,
                    tm_transform,
                    tm_entity,
                    tm_layer.0,
                    tile_pos,
                    tile,
                );
            }
        }
    }
}
