pub mod bevy_connect;
pub mod settings;

use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Value};
use rand_seeder::{rand_core::RngCore, Seeder, SipRng};

use crate::terrain;
use crate::terrain::settings::*;
use crate::tile::*;

pub const DEFAULT_SEED: &str = "Delpha 7";
pub const DEFAULT_SIZE: (u32, u32) = (128 * 5, 128);

type TileData = Vec<Vec<Tile>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Foreground = 0,
    Background = 1,
}

#[derive(Default)]
pub struct Terrain {
    width: u32,
    height: u32,
    seed: String,

    // TileData arrays for each layer
    tile_data: [TileData; 2],
}

impl Terrain {
    pub fn new(seed: Option<String>, width: u32, height: u32) -> Self {
        let mut world = Terrain {
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

        world.tile_data[0] = world.empty_tile_data();
        world.tile_data[1] = world.empty_tile_data();

        world
    }

    // Return a ref to a tile (no bound checking is done)
    pub fn get_tile_unchecked(&self, x: u32, y: u32, layer: Layer) -> &Tile {
        &self.tile_data[layer as usize][x as usize][y as usize]
    }

    // Return a mutable ref to a tile (no bound checking is done)
    pub fn get_tile_mut_unchecked(&mut self, x: u32, y: u32, layer: Layer) -> &mut Tile {
        &mut self.tile_data[layer as usize][x as usize][y as usize]
    }

    // The following two functions take an isize instead of a u32, this is
    // for cases such as x - 1, which could be negative, the functions
    // check for negatives and return None. isize is used as it can fit
    // the whole range of u32

    // Return a ref to a tile (bound checked)
    pub fn get_tile(&self, x: isize, y: isize, layer: Layer) -> Option<&Tile> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&self.tile_data[layer as usize][x as usize][y as usize])
        } else {
            None
        }
    }

    // Return a mutable ref to a tile (bound checked)
    pub fn get_tile_mut(&mut self, x: isize, y: isize, layer: Layer) -> Option<&mut Tile> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&mut self.tile_data[layer as usize][x as usize][y as usize])
        } else {
            None
        }
    }

    // Displays the world in a text format
    pub fn to_string(&self) -> String {
        let mut out = String::new();

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                match self.get_tile_unchecked(x, y, Layer::Foreground) {
                    Tile::Null => out.push('E'),

                    Tile::Air => out.push(' '),
                    Tile::Regular(RegularTile::Grass) => out.push('w'),
                    Tile::Regular(RegularTile::Dirt) => out.push('x'),
                    Tile::Regular(RegularTile::Stone) => out.push('□'),

                    // Don't print background tiles
                    Tile::Background(_) => (),
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

                if y <= max_height && value.get([x as f64, y as f64]) as f32 >= solid_density {
                    *self.get_tile_mut_unchecked(x, y, Layer::Foreground) =
                        Tile::Regular(RegularTile::Stone);
                } else {
                    *self.get_tile_mut_unchecked(x, y, Layer::Foreground) = Tile::Air;
                }

                // Generate background slightly below terrain
                if y <= max_height - settings.background_offset {
                    *self.get_tile_mut_unchecked(x, y, Layer::Background) =
                        Tile::Background(BackgroundTile::Stone);
                } else {
                    *self.get_tile_mut_unchecked(x, y, Layer::Background) = Tile::Air;
                }
            }
        }

        // Cellular automata smoothening
        for _ in 0..settings.caves.smooth_iters {
            self.tile_data[Layer::Foreground as usize] = self.smooth(settings.caves.convert_min);
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
                    if *self.get_tile_unchecked(x, y, Layer::Foreground) != Tile::Air {
                        *self.get_tile_mut_unchecked(x, y, Layer::Foreground) =
                            Tile::Regular(RegularTile::Dirt);
                    }

                    if *self.get_tile_mut_unchecked(x, y, Layer::Background) != Tile::Air {
                        *self.get_tile_mut_unchecked(x, y, Layer::Background) =
                            Tile::Background(BackgroundTile::Dirt)
                    }
                }
            }
        }

        // Grass
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                if *self.get_tile_unchecked(x, y, Layer::Foreground) != Tile::Air {
                    *self.get_tile_mut_unchecked(x, y, Layer::Foreground) =
                        Tile::Regular(RegularTile::Grass);

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

    pub fn get_surrounds(&self, x: u32, y: u32, layer: Layer) -> Surrounds {
        let mut surrounds = Surrounds::empty();

        // TL
        if let Some(t) = self.get_tile(x as isize - 1, y as isize + 1, layer) && *t != Tile::Air {
                surrounds.toggle(Surrounds::TL)
        }

        // TM
        if let Some(t) = self.get_tile(x as isize, y as isize + 1, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::TM)
        }

        // TR
        if let Some(t) = self.get_tile(x as isize + 1, y as isize + 1, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::TR)
        }

        // ML
        if let Some(t) = self.get_tile(x as isize - 1, y as isize, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::ML)
        }

        // MR
        if let Some(t) = self.get_tile(x as isize + 1, y as isize, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::MR)
        }

        // BL
        if let Some(t) = self.get_tile(x as isize - 1, y as isize - 1, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::BL)
        }

        // BM
        if let Some(t) = self.get_tile(x as isize, y as isize - 1, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::BM)
        }

        // BR
        if let Some(t) = self.get_tile(x as isize + 1, y as isize - 1, layer) && *t != Tile::Air {
            surrounds.toggle(Surrounds::BR)
        }

        surrounds
    }

    // Smooth out randomly generated noise by making each tile more similar to it's neighbour
    // Stores the result of the smooth in out to prevent tile_data from being corrupted in use
    fn smooth(&self, conv_min: u32) -> TileData {
        let mut out = self.empty_tile_data();

        for x in 0..self.width {
            for y in 0..self.height {
                let w_count = self.get_surrounds(x, y, Layer::Foreground).count();

                if w_count >= conv_min {
                    out[x as usize][y as usize] = Tile::Regular(RegularTile::Stone)
                } else if w_count < conv_min {
                    out[x as usize][y as usize] = Tile::Air
                }
            }
        }

        out
    }
}
