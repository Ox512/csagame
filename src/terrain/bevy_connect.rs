// Module containing functions that tie worldgen into
// bevy. These are seperated to keep the code modular

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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
        layer: Layer,
    ) {
        // Create atlas out of desired tileset
        let atlas = {
            let mut handle;
            if layer == Layer::Foreground {
                handle = asset_server.load("Tiles.png");
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
                        texture: TileTexture(get_texture_index(
                            self.get_tile_unchecked(x, y, layer),
                            Some(self.get_surrounds(x, y, layer)),
                        )),
                        ..Default::default()
                    })
                    .id();

                storage.set(&pos, Some(entity));
            }
        }

        // Add the tilemap to bevy
        commands.entity(tm_entity).insert_bundle(TilemapBundle {
            tile_size: TILE_SIZE,
            grid_size: GRID_SIZE,
            size: tm_size,
            texture: TilemapTexture(atlas.texture),
            transform: get_tilemap_center_transform(&tm_size, &GRID_SIZE, -(layer as usize as f32)),
            ..Default::default()
        });
    }
}

// Generate a tilemap with a randomly generated world
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Generate world
    let mut terrain = Terrain::new(None, DEFAULT_SIZE.0, DEFAULT_SIZE.1);

    terrain.generate(GenerationSettings::FOREST);
    terrain.spawn_layer_tilemap(&mut commands, &asset_server, Layer::Foreground);
    terrain.spawn_layer_tilemap(&mut commands, &asset_server, Layer::Background);
}
