pub mod bevy_connect;
pub mod layer;
pub mod settings;

use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Value};
use num_traits::FromPrimitive;
use rand::Rng;
use rand_seeder::{rand_core::RngCore, Seeder, SipRng};

use crate::terrain::layer::*;
use crate::terrain::settings::*;
use crate::tile::*;

pub const DEFAULT_SEED: &str = "B0RK3D";
pub const DEFAULT_SIZE: (u32, u32) = (128 * 5, 128);

pub struct Terrain {
    width: u32,
    height: u32,
    seed: String,

    // Noise and Rn generators
    rng: SipRng,
    value: Value,
    fbm: Fbm,

    // This will be removed when biomes are introduced
    settings: GenerationSettings,

    // TileData arrays for each layer
    layers: [Layer; 3],
}

impl Terrain {
    pub fn new(
        seed: Option<String>,
        settings: GenerationSettings,
        width: u32,
        height: u32,
    ) -> Self {
        // Use set seed if present
        let seed = if let Some(seed) = seed {
            seed
        } else {
            DEFAULT_SEED.to_string()
        };

        let layers = [
            Layer::new(width, height),
            Layer::new(width, height),
            Layer::new(width, height),
        ];

        let mut rng: SipRng = Seeder::from(seed.clone()).make_rng();

        let value = Value::new().set_seed(rng.next_u32());

        let fbm = Fbm::new()
            .set_seed(rng.next_u32())
            .set_lacunarity(settings.surface.lacunarity as f64)
            .set_persistence(settings.surface.persistence as f64)
            .set_octaves(settings.surface.octaves);

        Terrain {
            width,
            height,
            seed,
            rng,
            value,
            fbm,
            settings,
            layers,
        }
    }

    // Displays the world in a text format
    pub fn to_string(&self) -> String {
        let mut out = String::new();

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                match self.layers[Layer::FRONT].get_tile(x, y).id {
                    TileId::Null => out.push('E'),

                    TileId::Empty => out.push(' '),
                    TileId::Ground(Ground::Grass) => out.push('w'),
                    TileId::Ground(Ground::Dirt) => out.push('x'),
                    TileId::Ground(Ground::Stone) => out.push('□'),

                    // Only print Foreground tiles
                    _ => (),
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn generate(&mut self) {
        // Basic noise
        for x in 0..self.width {
            // Generate hills and mountains w/ fbm
            let max_height = (self.fbm.get([
                (self.settings.surface.scale * x as f32 / self.width as f32 * 4.0) as f64,
                0.0,
            ]) as f32
                * self.settings.surface.amplitude
                + self.settings.surface.height_offset * self.height as f32)
                as u32;

            for y in 0..self.height {
                // The density at which a block is considered solid
                // This is decreased higher up to create a more solid surface
                let solid_density = self.settings.caves.solid_density
                    - y as f32 / max_height as f32
                        * self.settings.caves.solid_density
                        * self.settings.caves.falloff;

                // All tiles above the max height should be empty
                // It is also good to make all tiles at max_height solid
                if y < max_height && self.value.get([x as f64, y as f64]) as f32 >= solid_density {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) =
                        Tile::new(TileId::Ground(Ground::Stone), None);
                } else if y == max_height {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) =
                        Tile::new(TileId::Ground(Ground::Stone), None);
                } else {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) = Tile::EMPTY;
                }

                // Generate background slightly below terrain
                if y <= max_height - self.settings.background_offset {
                    *self.layers[Layer::BACK].get_tile_mut(x, y) =
                        Tile::new(TileId::Background(Background::Stone), None);
                } else {
                    *self.layers[Layer::BACK].get_tile_mut(x, y) = Tile::EMPTY;
                }
            }
        }

        // Cellular automata smoothening
        for _ in 0..self.settings.caves.smooth_iters {
            self.layers[Layer::FRONT] = self.smooth();
        }

        // Dirt
        for x in 0..self.width {
            // Place dirt from this level up
            let dirt_height = (self.fbm.get([
                (self.settings.surface.scale * x as f32 / self.width as f32 * 4.0) as f64,
                0.0,
            ]) as f32
                * self.settings.surface.amplitude
                + self.settings.dirt_height * self.height as f32
                + self.value.get([x as f64, 0 as f64]) as f32 * self.settings.stone_jitter as f32)
                as u32;

            for y in dirt_height..self.height {
                if *self.layers[Layer::FRONT].get_tile(x, y) != Tile::EMPTY {
                    *self.layers[Layer::FRONT].get_tile_mut(x, y) =
                        Tile::new(TileId::Ground(Ground::Dirt), None);
                }

                if *self.layers[Layer::BACK].get_tile_mut(x, y) != Tile::EMPTY {
                    *self.layers[Layer::BACK].get_tile_mut(x, y) =
                        Tile::new(TileId::Background(Background::Dirt), None);
                }
            }
        }

        // Grass - go through each column and change the first solid tile to grass
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                if *self.layers[Layer::FRONT].get_tile(x, y) == Tile::EMPTY {
                    continue;
                }

                *self.layers[Layer::FRONT].get_tile_mut(x, y) =
                    Tile::new(TileId::Ground(Ground::Grass), None);

                break;
            }
        }

        // 'Initialize' the Middleground layer with TileId::Empty
        for x in 0..self.width {
            for y in 0..self.height {
                *self.layers[Layer::MIDDLE].get_tile_mut(x, y) = Tile::EMPTY
            }
        }

        // Trees
        let mut x = 4;

        // While loop is used as the iterator needs to be advanced in loop
        while x < self.width - 4 {
            // Skip if tree should not be generated here
            if self.value.get([x as f64, 2 as f64]) <= self.settings.trees.spawn_rate as f64 {
                x += 1;
                continue;
            }

            for y in (0..self.height).rev() {
                // Find a solid tile
                if *self.layers[Layer::FRONT].get_tile(x, y) == Tile::EMPTY {
                    continue;
                }

                self.generate_tree(x, y + 1);
                x += 5;
                break;
            }
        }

        // Surface decor
        let mut x = 0;

        // While loop is used as the iterator needs to be advanced in loop
        while x < self.width - 1 {
            // Skip if decor should not be generated here
            if self.value.get([x as f64, 1 as f64]) <= self.settings.decor.surface_rate as f64 {
                x += 1;
                continue;
            }

            for y in (0..self.height).rev() {
                // Find a solid tile
                if *self.layers[Layer::FRONT].get_tile(x, y) == Tile::EMPTY {
                    continue;
                }

                // Check that this space hasn't already been taken
                if *self.layers[Layer::MIDDLE].get_tile(x, y + 1) != Tile::EMPTY {
                    x += 1;
                    break;
                }

                // Select a decor tile
                let tile = {
                    let selection = self.rng.gen_range(self.settings.decor.surface.clone());

                    let tile = SurfaceDecor::from_usize(selection)
                        .expect("Selection not within range of possible tiles");

                    TileId::SurfaceDecor(tile)
                };

                let desc = TileDescriptor::from_id(tile);

                // Single width tiles can be directly placed
                if desc.dimensions.is_none() {
                    *self.layers[Layer::MIDDLE].get_tile_mut(x, y + 1) = Tile::new(tile, None);
                    x += 1;
                    break;
                }

                // For multi tiles, we need to check there is enough ground to fit them
                // and that they they won't be obsructed
                let mut suitable = true;
                for w in 1..desc.dimensions.unwrap().0 {
                    if *self.layers[Layer::FRONT].get_tile(x + w, y) == Tile::EMPTY {
                        suitable = false;
                    }

                    if *self.layers[Layer::FRONT].get_tile(x + w, y + 1) != Tile::EMPTY {
                        suitable = false;
                    }

                    if *self.layers[Layer::MIDDLE].get_tile(x + w, y + 1) != Tile::EMPTY {
                        suitable = false;
                    }
                }

                if !suitable {
                    x += 1;
                    break;
                }

                // All is good, generate multi tile
                self.generate_multi_tile(tile, x, y + 1);
                x += desc.dimensions.unwrap().0;
                break;
            }
        }

        // Match tiles to their surrounds
        for x in 0..self.width {
            for y in 0..self.height {
                // Foreground
                if self.layers[Layer::FRONT].get_tile_mut(x, y).id != TileId::Empty {
                    self.layers[Layer::FRONT].get_tile_mut(x, y).texture_offset = Some(
                        self.layers[Layer::FRONT]
                            .get_surrounds(x, y)
                            .get_texture_offset(),
                    );
                }

                // Background
                if self.layers[Layer::BACK].get_tile_mut(x, y).id != TileId::Empty {
                    self.layers[Layer::BACK].get_tile_mut(x, y).texture_offset = Some(
                        self.layers[Layer::BACK]
                            .get_surrounds(x, y)
                            .get_texture_offset(),
                    );
                }
            }
        }
    }

    fn generate_tree(&mut self, x: u32, y: u32) {
        // Generate randomly sized trunks
        let trunk_height = self
            .rng
            .gen_range(self.settings.trees.trunk_height_range.clone());

        for h in 0..trunk_height {
            let variant = self
                .rng
                .gen_range(0..self.settings.trees.trunk_variants - 1);
            *self.layers[Layer::MIDDLE].get_tile_mut(x, y + h) =
                Tile::new(TileId::Tree(Tree::Wood), Some((variant, 0)));
        }

        // Generate foliage
        let foliage = TileDescriptor::from_id(TileId::Tree(Tree::Foliage));
        self.generate_multi_tile(foliage.id, x - foliage.dimensions.unwrap().0 / 2, y + trunk_height)
    }

    fn generate_multi_tile(&mut self, id: TileId, x: u32, y: u32) {
        let desc = TileDescriptor::from_id(id);
        let size = desc
            .dimensions
            .unwrap_or_else(|| panic!("Tile {:?} is not a multi tile", id));

        for x1 in 0..size.0 {
            for y1 in 0..size.1 {
                *self.layers[Layer::MIDDLE].get_tile_mut(x + x1, y + y1) =
                    Tile::new(id, Some((x1, size.1 - y1 - 1)))
            }
        }
    }

    // Smooth out randomly generated noise by making each tile more similar to it's neighbour
    // Stores the result of the smooth in out to prevent tile_data from being corrupted in use
    fn smooth(&self) -> Layer {
        let mut output = Layer::new(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                let w_count = self.layers[Layer::FRONT].get_surrounds(x, y).count();

                if w_count >= self.settings.caves.convert_min {
                    *output.get_tile_mut(x, y) = Tile::new(TileId::Ground(Ground::Stone), None)
                } else if w_count < self.settings.caves.convert_min {
                    *output.get_tile_mut(x, y) = Tile::EMPTY
                }
            }
        }

        output
    }
}
