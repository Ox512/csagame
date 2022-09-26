pub mod settings;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Value};
use rand_seeder::{rand_core::RngCore, Seeder, SipRng};

use crate::tile::*;
use crate::world::settings::*;

pub const DEFAULT_SEED: &str = "Delpha 7";
pub const DEFAULT_SIZE: (u32, u32) = (128, 128);

type TileData = Vec<Vec<Tile>>;

#[derive(Default)]
pub struct World {
    width: u32,
    height: u32,
    seed: String,

    tile_data: TileData,
}

impl World {
    pub fn new(seed: Option<String>, width: u32, height: u32) -> Self {
        let mut world = World {
            width,
            height,
            ..Default::default()
        };

        // Use set seed if present
        if let Some(seed) = seed {
            world.seed = seed
        } else {
            world.seed = DEFAULT_SEED.to_string();
        }

        world.tile_data = world.empty_tile_data();

        world
    }

    pub fn tiles(&self, x: u32, y: u32) -> &Tile {
        &self.tile_data[x as usize][y as usize]
    }

    pub fn tiles_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        &mut self.tile_data[x as usize][y as usize]
    }

    /// Displays the world in a text format
    pub fn to_string(&self) -> String {
        let mut out = String::new();

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                match self.tiles(x, y) {
                    Tile::Null => out.push(char::from_u32(0).unwrap()),

                    Tile::Air => out.push(' '),
                    Tile::Grass => out.push('w'),
                    Tile::Dirt => out.push('x'),
                    Tile::Stone => out.push('□'),
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn generate(&mut self, settings: GenerationSettings) {
        // Setup and set noise seed
        let mut rng: SipRng = Seeder::from(self.seed.clone()).make_rng();

        let value = Value::new().set_seed(rng.next_u32());

        let fbm = Fbm::new()
            .set_seed(rng.next_u32())
            .set_lacunarity(settings.surface.lacunarity as f64)
            .set_persistence(settings.surface.persistence as f64)
            .set_octaves(settings.surface.octaves);

        // Basic noise
        for x in 0..self.width {
            // Generate hills and mountains w/ fbm
            let max_height = (fbm.get([
                (settings.surface.scale * x as f32 / self.width as f32 * 4.0) as f64,
                0.0,
            ]) as f32
                * settings.surface.amplitude
                + settings.surface.height_offset * self.height as f32)
                as u32;

            for y in 0..self.height {
                // The density at which a block is considered solid
                // This is decreased higher up to create a more solid surface
                let solid_density = settings.caves.solid_density
                    - y as f32 / max_height as f32
                        * settings.caves.solid_density
                        * settings.caves.falloff;

                if value.get([x as f64, y as f64]) as f32 > solid_density && y <= max_height {
                    *self.tiles_mut(x, y) = Tile::Stone;
                } else {
                    *self.tiles_mut(x, y) = Tile::Air;
                }
            }
        }

        // Cellular automata smoothening
        for _ in 0..settings.caves.smooth_iters {
            self.tile_data = self.smooth(settings.caves.convert_min);
        }

        // Dirt
        for x in 0..self.width {
            for y in 0..self.height {
                // Maximum height to place stone
                let stone_height = (settings.stone_height * self.height as f32
                    + fbm.get([
                        (settings.surface.scale * x as f32 / self.width as f32) as f64,
                        5.0,
                    ]) as f32
                        * settings.stone_blur as f32
                    + value.get([x as f64, 0 as f64]) as f32 * settings.stone_jitter as f32)
                    as u32;

                if y >= stone_height && *self.tiles(x, y) != Tile::Air {
                    *self.tiles_mut(x, y) = Tile::Dirt;
                }
            }
        }

        // Grass
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                if *self.tiles(x, y) != Tile::Air {
                    *self.tiles_mut(x, y) = Tile::Grass;

                    // Move onto next column
                    break;
                }
            }
        }
    }

    // Creates an initialized array able to store tile data
    fn empty_tile_data(&self) -> TileData {
        let mut out = Vec::with_capacity(self.width as usize);
        for x in 0..self.width {
            out.push(Vec::with_capacity(self.height as usize));

            for _ in 0..self.height {
                out[x as usize].push(Tile::Null);
            }
        }

        out
    }

    // Counts the number of solid tiles surrounding a tile
    fn get_surrounding_count(&self, pos: [u32; 2]) -> u32 {
        let mut count = 0;

        // Saturating sub will handle <0 bound checks
        let x_start = pos[0].saturating_sub(1);
        let y_start = pos[1].saturating_sub(1);

        let x_end;
        if pos[0] == self.width - 1 {
            x_end = self.width - 1;
        } else {
            x_end = pos[0] + 1;
        }

        let y_end;
        if pos[1] == self.height - 1 {
            y_end = self.height - 1;
        } else {
            y_end = pos[1] + 1;
        }

        for x in x_start..=x_end {
            for y in y_start..=y_end {
                // Ignore centre tile
                if x == pos[0] && y == pos[1] {
                    continue;
                }

                if *self.tiles(x, y) != Tile::Air {
                    count += 1;
                }
            }
        }
        count
    }

    // Smooth out randomly generated noise by making each tile more similar to it's neighbour
    // Stores the result of the smooth in out to prevent tile_data from being corrupted in use
    fn smooth(&self, conv_min: u32) -> TileData {
        let mut out = self.empty_tile_data();

        for x in 0..self.width {
            for y in 0..self.height {
                let w_count = self.get_surrounding_count([x, y]);

                if w_count > conv_min {
                    out[x as usize][y as usize] = Tile::Stone
                } else if w_count < conv_min {
                    out[x as usize][y as usize] = Tile::Air
                }
            }
        }

        out
    }
}

// Generate a tilemap with a randomly generated world
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tm_size = TilemapSize {
        x: 32,//DEFAULT_SIZE.0,
        y: 32,//DEFAULT_SIZE.1,
    };

    let mut tm_storage = TileStorage::empty(tm_size);

    let tm_entity = commands.spawn().id();

    let texture_handle: Handle<Image> = asset_server.load("Tiles.png");

    for x in 0..tm_size.x {
        for y in 0..tm_size.y {
            let pos = TilePos { x, y };

            let entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: pos,
                    tilemap_id: TilemapId(tm_entity),
                    ..Default::default()
                })
                .id();

            tm_storage.set(&pos, Some(entity));
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 16.0 };

    commands.entity(tm_entity).insert_bundle(TilemapBundle {
        tile_size,
        grid_size,
        texture: TilemapTexture(texture_handle),
        transform: get_centered_transform_2d(&tm_size, &tile_size, 1.0),
        ..Default::default()
    });
}
