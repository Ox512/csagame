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
        let tm_entity = commands.spawn().id();

        // Place tiles
        for x in 0..tm_size.x {
            for y in 0..tm_size.y {
                let pos = TilePos { x, y };

                let entity = commands
                    .spawn()
                    .insert_bundle(TileBundle {
                        position: pos,
                        tilemap_id: TilemapId(tm_entity),
                        texture: TileTexture(self.layers[layer].get_tile(x, y).get_texture_index()),
                        ..Default::default()
                    })
                    .id();

                storage.set(&pos, Some(entity));
            }
        }

        let mut transform =
            get_tilemap_center_transform(&tm_size, &GRID_SIZE, -(layer as usize as f32));

        transform.translation.y += offset;

        // Add the tilemap to bevy
        commands.entity(tm_entity).insert_bundle(TilemapBundle {
            tile_size: TILE_SIZE,
            grid_size: GRID_SIZE,
            size: tm_size,
            texture: TilemapTexture(atlas.texture),
            transform,
            ..Default::default()
        });
    }
}

// Generate a tilemap with a randomly generated world
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Generate world
    let mut terrain = Terrain::new(
        None,
        GenerationSettings::FOREST,
        DEFAULT_SIZE.0,
        DEFAULT_SIZE.1,
    );

    terrain.generate();

    for i in 0..Layer::TOTAL_LAYERS {
        terrain.spawn_layer_tilemap(&mut commands, &asset_server, i)
    }
}
