pub mod bevy_connect;
pub mod layer;
pub mod settings;

use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Value};
use rand_seeder::{rand_core::RngCore, Seeder, SipRng};

use crate::terrain::layer::*;
use crate::terrain::settings::*;
use crate::tile::multi_tile::*;
use crate::tile::*;

pub const DEFAULT_SEED: &str = "Delpha 7";
pub const DEFAULT_SIZE: (u32, u32) = (128 * 5, 128);

#[derive(Default)]
pub struct Terrain {
    width: u32,
    height: u32,
    seed: String,

    // TileData arrays for each layer
    layers: [Layer; 3],
}

impl Terrain {
    pub fn new(seed: Option<String>, width: u32, height: u32) -> Self {
        let mut out = Terrain {
            width,
            height,
            ..Default::default()
        };

        // Use set seed if present
        if let Some(seed) = seed {
            out.seed = seed
        } else {
            out.seed = DEFAULT_SEED.to_string();
        }

        for i in 0..Layer::TOTAL_LAYERS {
            out.layers[i] = Layer::new(width, height)
        }

        out
    }

    // Displays the world in a text format
    pub fn to_string(&self) -> String {
        let mut out = String::new();

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                match self.layers[Layer::FRONT].get_tile(x, y) {
                    Tile::Null => out.push('E'),

                    Tile::Air => out.push(' '),
                    Tile::Ground(GroundTile::Grass) => out.push('w'),
                    Tile::Ground(GroundTile::Dirt) => out.push('x'),
                    Tile::Ground(GroundTile::Stone) => out.push('□'),

                    // Don't print background tiles or middle ground tiles
                    Tile::Background(_) => (),
                    Tile::Decor(_) => (),
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn generate(&mut self, settings: GenerationSettings) {
        // Setup rng and noise functions
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

                if y < max_height && value.get([x as f64, y as f64]) as f32 >= solid_density {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) = Tile::Ground(GroundTile::Stone);
                } else if y == max_height {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) = Tile::Ground(GroundTile::Stone);
                } else {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) = Tile::Air;
                }

                // Generate background slightly below terrain
                if y <= max_height - settings.background_offset {
                    *self.layers[Layer::BACK].get_tile_mut(x, y) =
                        Tile::Background(BackgroundTile::Stone);
                } else {
                    *self.layers[Layer::BACK].get_tile_mut(x, y) = Tile::Air;
                }
            }
        }

        // Cellular automata smoothening
        for _ in 0..settings.caves.smooth_iters {
            self.layers[Layer::FRONT] = self.smooth(settings.caves.convert_min);
        }

        // Dirt
        for x in 0..self.width {
            for y in 0..self.height {
                // Maximum height to place stone
                let stone_height = (fbm.get([
                    (settings.surface.scale * x as f32 / self.width as f32 * 4.0) as f64,
                    0.0,
                ]) as f32
                    * settings.surface.amplitude
                    + settings.stone_height * self.height as f32
                    + value.get([x as f64, 0 as f64]) as f32 * settings.stone_jitter as f32)
                    as u32;

                if y >= stone_height {
                    if *self.layers[Layer::FRONT].get_tile(x, y) != Tile::Air {
                        *self.layers[Layer::FRONT].get_tile_mut(x, y) =
                            Tile::Ground(GroundTile::Dirt);
                    }

                    if *self.layers[Layer::BACK].get_tile_mut(x, y) != Tile::Air {
                        *self.layers[Layer::BACK].get_tile_mut(x, y) =
                            Tile::Background(BackgroundTile::Dirt)
                    }
                }
            }
        }

        // Grass and Greenery decor
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                if *self.layers[Layer::FRONT].get_tile(x, y) != Tile::Air {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) = Tile::Ground(GroundTile::Grass);

                    self.generate_multi_tile(MultiTile::GrassMedium, x, y + 1);

                    // Move onto next column
                    break;
                }
            }
        }
    }

    // Generates a multi_tile structure
    fn generate_multi_tile(&mut self, multi: MultiTile, x: u32, y: u32) {
        let info = &MULTI_TILES[multi as usize];

        for i in 0..info.width {
            for j in 0..info.height {
                *self.layers[Layer::MIDDLE].get_tile_mut(x + i, y + j) = match multi {
                    MultiTile::GrassMedium => Tile::Decor(DecorTile::GrassMedium(i, j)),
                }
            }
        }
    }

    // Smooth out randomly generated noise by making each tile more similar to it's neighbour
    // Stores the result of the smooth in out to prevent tile_data from being corrupted in use
    fn smooth(&self, conv_min: u32) -> Layer {
        let mut output = Layer::new(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                let w_count = self.layers[Layer::FRONT].get_surrounds(x, y).count();

                if w_count >= conv_min {
                    *output.get_tile_mut(x, y) = Tile::Ground(GroundTile::Stone)
                } else if w_count < conv_min {
                    *output.get_tile_mut(x, y) = Tile::Air
                }
            }
        }

        output
    }
}
