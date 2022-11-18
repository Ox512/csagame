use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::camera::CursorPos;
use crate::terrain::bevy_connect::TilemapLayer;
use crate::terrain::layer::Layer;
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
    terrain: ResMut<Terrain>,
) {
    // NOTE: tm_query is ordered by layer, which is very useful for ensuring that only
    //       the currently visible layer is acted upon, however this isn't guarenteed by
    //       and may break in later versions

    if mouse_input.just_pressed(MouseButton::Left) {
        // Remove a tile
        for (_entity, tm_size, tm_grid_size, tm_storage, tm_layer, tm_transform) in
            tm_query.iter_mut()
        {
            let world_pos = (tm_transform.compute_matrix().inverse()
                * Vec4::from((cursor_pos.0, 0.0, 1.0)))
            .xy();

            // If the cusor is at a valid position on the tilemap
            if let Some(tile_pos) =
                TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
            {
                if let Some(tile_entity) = tm_storage.get(&tile_pos) {
                    remove_tile(
                        &mut commands,
                        terrain,
                        tm_layer,
                        tm_storage,
                        tile_pos,
                        tile_entity,
                    );

                    break;
                }
            }
        }
    } else if mouse_input.just_pressed(MouseButton::Right) {
        // place a tile
        let (entity, tm_size, tm_grid_size, tm_storage, tm_layer, tm_transform) =
            tm_query.iter_mut().next().expect("No foreground tilemap");

        let world_pos =
            (tm_transform.compute_matrix().inverse() * Vec4::from((cursor_pos.0, 0.0, 1.0))).xy();

        // If the cusor is at a valid position on the tilemap
        if let Some(tile_pos) =
            TilePos::from_world_pos(&world_pos, tm_size, tm_grid_size, &TilemapType::Square)
        {
            if tm_storage.get(&tile_pos) == None {
                place_tile(
                    &mut commands,
                    terrain,
                    tm_layer,
                    tm_storage,
                    tile_pos,
                    entity,
                );
            }
        }
    }
}

fn remove_tile(
    commands: &mut Commands,
    mut terrain: ResMut<Terrain>,
    layer: &TilemapLayer,
    mut storage: Mut<TileStorage>,
    pos: TilePos,
    entity: Entity,
) {
    // Remove tile from tilemap, bevy (entity) and world data
    storage.remove(&pos);
    commands.entity(entity).despawn_recursive();
    terrain.layers[layer.0][(pos.x, pos.y)] = Tile::EMPTY;

    // Update surrounding tiles - only on fore and background
    if layer.0 == Layer::MIDDLE {
        return;
    }

    for x in (pos.x as isize - 1)..=(pos.x as isize + 1) {
        for y in (pos.y as isize - 1)..=(pos.y as isize + 1) {
            // This has to be done out of the if-let - E0502
            let new_offset = terrain.layers[layer.0]
                .get_surrounds(x as u32, y as u32)
                .get_texture_offset();

            if let Some(tile) = terrain.layers[layer.0].get_tile_mut_checked(x, y) {
                if let Some(entity) = storage.get(&TilePos::new(x as u32, y as u32)) {
                    tile.texture_offset = Some(new_offset);
                    commands
                        .entity(entity)
                        .insert(TileTextureIndex(tile.get_texture_index()));
                }
            }
        }
    }
}

fn place_tile(
    commands: &mut Commands,
    mut terrain: ResMut<Terrain>,
    layer: &TilemapLayer,
    mut storage: Mut<TileStorage>,
    pos: TilePos,
    tm_entity: Entity,
) {
    // Create a tile
    let tile = Tile::new(
        TileId::Ground(Ground::Stone),
        Some(
            terrain.layers[layer.0]
                .get_surrounds(pos.x, pos.y)
                .get_texture_offset(),
        ),
    );

    // Add tile to tilemap, bevy (entity) and world data
    let tile_entity = commands
        .spawn_empty()
        .insert(TileBundle {
            position: pos,
            tilemap_id: TilemapId(tm_entity),
            texture_index: TileTextureIndex(tile.get_texture_index()),
            ..Default::default()
        })
        .id();

    storage.set(&pos, tile_entity);

    terrain.layers[layer.0][(pos.x, pos.y)] = tile;

    // Update surrounding tiles - only on fore and background
    if layer.0 == Layer::MIDDLE {
        return;
    }

    for x in (pos.x as isize - 1)..=(pos.x as isize + 1) {
        for y in (pos.y as isize - 1)..=(pos.y as isize + 1) {
            // This has to be done out of the if-let - E0502
            let new_offset = terrain.layers[layer.0]
                .get_surrounds(x as u32, y as u32)
                .get_texture_offset();

            if let Some(tile) = terrain.layers[layer.0].get_tile_mut_checked(x, y) {
                if let Some(entity) = storage.get(&TilePos::new(x as u32, y as u32)) {
                    tile.texture_offset = Some(new_offset);
                    commands
                        .entity(entity)
                        .insert(TileTextureIndex(tile.get_texture_index()));
                }
            }
        }
    }
}
