// Module containing functions that tie worldgen into
// bevy. These are seperated to keep the code modular

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::terrain::layer::*;
use crate::terrain::settings::*;
use crate::terrain::*;
use crate::tile::TILESET_SIZE;

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
            if layer == Layer::FRONT {
                handle = asset_server.load("Tiles.png");
            } else if layer == Layer::MIDDLE {
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
        let offset = if layer == Layer::MIDDLE { -3.0 } else { 0.0 };

        let tm_size = TilemapSize {
            x: self.width,
            y: self.height,
        };

        let mut storage = TileStorage::empty(tm_size);

        // Entity corresponding to the whole tilemap
        let tm_entity = commands.spawn_empty().id();

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
                    .id();

                storage.set(&pos, entity);
            }
        }

        let mut transform = get_tilemap_center_transform(
            &tm_size,
            &GRID_SIZE,
            &TilemapType::Square,
            -1.0 - (layer as usize as f32),
        );

        transform.translation.y += offset;

        // Add the tilemap to bevy
        commands
            .entity(tm_entity)
            .insert(TilemapBundle {
                tile_size: TILE_SIZE,
                grid_size: GRID_SIZE,
                size: tm_size,
                texture: TilemapTexture::Single(atlas.texture),
                transform,
                storage,
                ..Default::default()
            })
            .insert(TilemapLayer(layer));
    }
}

// Generate a tilemap with a randomly generated world
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Generate world
    let mut terrain = Terrain::new(None, GenerationSettings::FOREST, WORLD_SIZE.0, WORLD_SIZE.1);

    terrain.generate();

    for i in 0..Layer::TOTAL_LAYERS {
        terrain.spawn_layer_tilemap(&mut commands, &asset_server, i)
    }

    commands.insert_resource(terrain)
}
