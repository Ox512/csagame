// Module containing functions that tie worldgen into
// bevy. These are seperated to keep the code modular

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::terrain::settings::*;
use crate::terrain::*;
use crate::tile::TILESET_SIZE;
use crate::*;

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 8.0, y: 8.0 };
const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 8.0, y: 8.0 };

#[derive(Component)]
pub struct TilemapLayer(pub usize);

impl Terrain {
    pub fn spawn_layer_tilemap(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        layer: usize,
    ) {
        // Create atlas out of desired tileset
        let atlas = {
            let handle;
            if layer == FRONT {
                handle = asset_server.load("Tiles.png");
            } else if layer == MIDDLE {
                handle = asset_server.load("MiddlegroundTiles.png");
            } else {
                handle = asset_server.load("BackgroundTiles.png");
            }

            TextureAtlas::from_grid(
                handle,
                Vec2::new(TILE_SIZE.x, TILE_SIZE.y),
                TILESET_SIZE.0 as usize,
                TILESET_SIZE.1 as usize,
                None,
                None,
            )
        };

        // Tiles in the MIDDLE layer are shifted down by
        // 2 pixels so that they connect nicely to FRONT tiles
        let offset = if layer == MIDDLE { -3.0 } else { 0.0 };

        let tm_size = TilemapSize {
            x: self.width,
            y: self.height,
        };

        let mut storage = TileStorage::empty(tm_size);

        // Entity corresponding to the whole tilemap
        let tm_entity = commands.spawn_empty().id();

        let mut tm_transform = get_tilemap_center_transform(
            &tm_size,
            &GRID_SIZE,
            &TilemapType::Square,
            -1.0 - (layer as usize as f32),
        );

        // Place tiles
        for x in 0..tm_size.x {
            for y in 0..tm_size.y {
                // Skip empty tiles
                if self.layers[layer][(x, y)] == Tile::EMPTY {
                    continue;
                }

                let pos = TilePos { x, y };

                let entity = commands
                    .spawn_empty()
                    .insert(TileBundle {
                        position: pos,
                        tilemap_id: TilemapId(tm_entity),
                        texture_index: TileTextureIndex(
                            self.layers[layer][(x, y)].get_texture_index(),
                        ),
                        ..Default::default()
                    })
                    .insert(TransformBundle::from(Transform::from_xyz(
                        x as f32 * 8.0 + tm_transform.translation.x,
                        y as f32 * 8.0 + tm_transform.translation.y,
                        0.0,
                    )))
                    .id();

                storage.set(&pos, entity);
            }
        }

        tm_transform.translation.y += offset;

        // Add the tilemap to bevy
        commands
            .entity(tm_entity)
            .insert(TilemapBundle {
                tile_size: TILE_SIZE,
                grid_size: GRID_SIZE,
                size: tm_size,
                texture: TilemapTexture::Single(atlas.texture),
                transform: tm_transform,
                storage,
                ..Default::default()
            })
            .insert(TilemapLayer(layer));
    }

    // Update the textures of tiles surround a tile
    // Used when adding or removing tiles
    pub fn update_surrounds(
        &mut self,
        commands: &mut Commands,
        storage: &mut TileStorage,
        pos: TilePos,
        layer: usize,
    ) {
        for x in (pos.x as isize - 1)..=(pos.x as isize + 1) {
            for y in (pos.y as isize - 1)..=(pos.y as isize + 1) {
                // This has to be done out of the if-let - E0502
                let new_offset = self
                    .get_surrounds(layer, x as u32, y as u32)
                    .get_texture_offset();

                if let Some(tile) = self.layers[layer].get_mut(x, y) {
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

    // Insert a tile into the tilemap
    pub fn insert_tile(
        &mut self,
        commands: &mut Commands,
        tm_storage: &mut TileStorage,
        tm_transform: &Transform,
        tm_entity: Entity,
        layer: usize,
        pos: TilePos,
        tile: Tile,
    ) {
        let entity = commands
            .spawn_empty()
            .insert(TileBundle {
                position: pos,
                tilemap_id: TilemapId(tm_entity),
                texture_index: TileTextureIndex(tile.get_texture_index()),
                ..Default::default()
            })
            .insert(TransformBundle::from(Transform::from_xyz(
                pos.x as f32 * 8.0 + tm_transform.translation.x,
                pos.y as f32 * 8.0 + tm_transform.translation.y,
                0.0,
            )))
            .id();

        tm_storage.set(&pos, entity);

        self.layers[layer][(pos.x, pos.y)] = tile;
        self.update_surrounds(commands, tm_storage, pos, layer);

        // Update Pathfinding nodes

        // Check if this is now a valid walking tile
        let tiles_above = [
            self.layers[FRONT].get(pos.x as isize, pos.x as isize + 1),
            self.layers[FRONT].get(pos.x as isize, pos.x as isize + 2),
            self.layers[FRONT].get(pos.x as isize, pos.x as isize + 3),
        ];

        if (tiles_above[0].is_none() || tiles_above[0] == Some(&Tile::EMPTY))
            && (tiles_above[1].is_none() || tiles_above[1] == Some(&Tile::EMPTY))
            && (tiles_above[2].is_none() || tiles_above[2] == Some(&Tile::EMPTY))
        {
            self.nodes[(pos.x, pos.y)] = node::PathTile::Walkable;
        }

        // Check that this added tile hasn't obstructed any other nodes
        if let Some(node) = self.nodes.get_mut(pos.x as isize, pos.y as isize - 1) {
            *node = node::PathTile::NonWalkable;
        }

        if let Some(node) = self.nodes.get_mut(pos.x as isize, pos.y as isize - 2) {
            *node = node::PathTile::NonWalkable;
        }
    }

    pub fn remove_tile(
        &mut self,
        commands: &mut Commands,
        tm_storage: &mut TileStorage,
        layer: usize,
        pos: TilePos,
    ) -> Option<()> {
        // Remove the tile's entity
        let entity = tm_storage.get(&pos)?;

        tm_storage.remove(&pos);
        commands.entity(entity).despawn_recursive();
        self.layers[layer][(pos.x, pos.y)] = Tile::EMPTY;

        // Update surrounding tiles - only on fore and background
        if layer == MIDDLE {
            return Some(());
        }

        self.update_surrounds(commands, tm_storage, pos, layer);

        // Update Pathfinding nodes

        // This node could now belong to the air-space of three
        // other nodes
        let ground_tiles = [
            self.layers[FRONT].get(pos.x as isize, pos.y as isize - 1),
            self.layers[FRONT].get(pos.x as isize, pos.y as isize - 2),
            self.layers[FRONT].get(pos.x as isize, pos.y as isize - 3),
        ];

        for i in 0..ground_tiles.len() {
            // Check for a valid floor tile
            if let Some(tile) = ground_tiles[i] && *tile != Tile::EMPTY {
                // Now check the 3 air tiles above it
                let y = (pos.y - 1 - i as u32) as isize;

                let tiles_above = [
                    self.layers[FRONT].get(pos.x as isize, y + 1),
                    self.layers[FRONT].get(pos.x as isize, y as isize + 2),
                    self.layers[FRONT].get(pos.x as isize, y as isize + 3),
                ];

                if (tiles_above[0].is_none() || tiles_above[0] == Some(&Tile::EMPTY))
                    && (tiles_above[1].is_none() || tiles_above[1] == Some(&Tile::EMPTY))
                    && (tiles_above[2].is_none() || tiles_above[2] == Some(&Tile::EMPTY))
                {
                    self.nodes[(pos.x, y as u32)] = PathTile::Walkable;
                }
            }
        }

        Some(())
    }
}

// Generate a tilemap with a randomly generated world
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Generate world
    let mut terrain = Terrain::new(None, GenerationSettings::FOREST, WORLD_SIZE.0, WORLD_SIZE.1);

    terrain.generate();

    for i in 0..TOTAL_LAYERS {
        terrain.spawn_layer_tilemap(&mut commands, &asset_server, i)
    }

    commands.insert_resource(terrain)
}
