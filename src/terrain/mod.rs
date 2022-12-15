pub mod bevy_connect;
//pub mod layer;
pub mod node;
pub mod settings;

use bevy::prelude::Resource;
use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Value};
use num_traits::FromPrimitive;
use rand::Rng;
use rand_seeder::{rand_core::RngCore, Seeder, SipRng};

use self::node::*;
use self::settings::*;

use crate::layer::*;
use crate::surrounds::Surrounds;
use crate::tile::*;
use crate::CHUNK_COUNT;
use crate::WORLD_SEED;

// Layer index constants
pub const FRONT: usize = 0;
pub const MIDDLE: usize = 1;
pub const BACK: usize = 2;
pub const TOTAL_LAYERS: usize = 3;

#[derive(Resource)]
pub struct Terrain {
    pub width: u32,
    pub height: u32,
    pub seed: String,

    // Noise and Rn generators
    rng: SipRng,
    value: Value,
    surface_fbm: Fbm,

    // This will be removed when biomes are introduced
    settings: GenerationSettings,

    // TileData arrays for each layer
    pub layers: [Layer<Tile>; 3],

    // All the floors in the game
    pub nodes: Layer<PathTile>,
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
            WORLD_SEED.to_string()
        };

        let layers = [
            Layer::new(width, height),
            Layer::new(width, height),
            Layer::new(width, height),
        ];

        let mut rng: SipRng = Seeder::from(seed.clone()).make_rng();

        let value = Value::new().set_seed(rng.next_u32());

        let surface_fbm = Fbm::new()
            .set_seed(rng.next_u32())
            .set_lacunarity(settings.surface.lacunarity as f64)
            .set_persistence(settings.surface.persistence as f64)
            .set_octaves(settings.surface.octaves);

        let nodes = Layer::new(width, height);

        Terrain {
            width,
            height,
            seed,
            rng,
            value,
            surface_fbm,
            settings,
            layers,
            nodes,
        }
    }

    pub fn generate(&mut self) {
        // Basic noise
        for x in 0..self.width {
            // Generate hills and mountains w/ fbm
            let max_height = (self.surface_fbm.get([
                (self.settings.surface.scale * x as f32 / self.width as f32 * CHUNK_COUNT as f32)
                    as f64,
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
                if y <= max_height && self.value.get([x as f64, y as f64]) as f32 >= solid_density {
                    self.layers[FRONT][(x, y)] = Tile::new(TileId::Ground(Ground::Stone), None);
                } else {
                    self.layers[FRONT][(x, y)] = Tile::EMPTY;
                }

                // Generate background slightly below terrain
                if y <= max_height - self.settings.background_offset {
                    self.layers[BACK][(x, y)] =
                        Tile::new(TileId::Background(Background::Stone), None);
                } else {
                    self.layers[BACK][(x, y)] = Tile::EMPTY;
                }
            }
        }

        // Cellular automata smoothening
        for _ in 0..self.settings.caves.smooth_iters {
            self.layers[FRONT] = self.smooth();
        }

        // Dirt
        for x in 0..self.width {
            // Place dirt from this level up
            let dirt_height = (self.surface_fbm.get([
                (self.settings.surface.scale * x as f32 / self.width as f32 * 4.0) as f64,
                0.0,
            ]) as f32
                * self.settings.surface.amplitude
                + self.settings.dirt_height * self.height as f32
                + self.rng.gen::<f32>() * self.settings.stone_jitter as f32)
                as u32;

            for y in dirt_height..self.height {
                if self.layers[FRONT][(x, y)] != Tile::EMPTY {
                    self.layers[FRONT][(x, y)] = Tile::new(TileId::Ground(Ground::Dirt), None);
                }

                if self.layers[BACK][(x, y)] != Tile::EMPTY {
                    self.layers[BACK][(x, y)] =
                        Tile::new(TileId::Background(Background::Dirt), None);
                }
            }
        }

        // Ores
        for _ in 0..self.width / self.settings.ore_rate {
            // Choose a random coordinate
            let (x, y) = (
                self.rng.gen_range(0..self.width - 1),
                self.rng
                    .gen_range(0..((self.height as f32 * self.settings.ore_height) as u32)),
            );

            // Don't overwrite empty tiles
            if self.layers[FRONT][(x, y)] == Tile::EMPTY {
                continue;
            }

            // Find an ore that can spawn at the current height
            // This is done by randomly choosing ores until a suitable one is found

            let mut desc = {
                let selection = self.rng.gen_range(0..std::mem::variant_count::<Ore>());
                TileDescriptor::from_id(TileId::Ore(Ore::from_usize(selection).unwrap()))
            };

            while ((desc.ore.unwrap().max_height * self.settings.ore_height * self.height as f32)
                as u32)
                < y
            {
                let selection = self.rng.gen_range(0..std::mem::variant_count::<Ore>());
                desc = TileDescriptor::from_id(TileId::Ore(Ore::from_usize(selection).unwrap()));
            }

            let ore = desc.ore.unwrap();

            // TODO: Ores might be broken - check
            for w in (x as isize - ore.radius as isize)..(x as isize + ore.radius as isize) {
                for h in (y as isize - ore.radius as isize)..(y as isize + ore.radius as isize) {
                    if let Some(tile) = self.layers[FRONT].get_mut(w, h) {
                        // Pass the distance to func
                        let dist = ((y as f32 - w as f32).powf(2.0)
                            + (y as f32 - h as f32).powf(2.0))
                        .sqrt();

                        let gen_chance = dist / ore.radius as f32;

                        if self.rng.gen::<f32>() > gen_chance {
                            *tile = Tile::new(desc.id, None);
                        }
                    }
                }
            }
        }

        // Grass - go through each column and change the first solid tile to grass
        for x in 0..self.width {
            for y in (0..self.height).rev() {
                if self.layers[FRONT][(x, y)] == Tile::EMPTY {
                    continue;
                }

                self.layers[FRONT][(x, y)] = Tile::new(TileId::Ground(Ground::Grass), None);

                break;
            }
        }

        // 'Initialize' the Middleground layer with TileId::Empty
        for x in 0..self.width {
            for y in 0..self.height {
                self.layers[MIDDLE][(x, y)] = Tile::EMPTY
            }
        }

        // TODO: The placement code for trees and surface decor
        //       is very similar. Find a way to decouple it.

        // Trees
        let mut x = 4;

        // While loop is used as the iterator needs to be advanced in loop
        while x < self.width - 1 {
            // Skip if tree should not be generated here
            if self.rng.gen::<f32>() <= self.settings.trees.spawn_rate {
                x += 1;
                continue;
            }

            for y in (0..self.height).rev() {
                // Find a solid tile
                if self.layers[FRONT][(x, y)] == Tile::EMPTY {
                    continue;
                }

                // Check the left and right side of the tile for edges
                if self.layers[FRONT][(x - 1, y)] == Tile::EMPTY
                    || self.layers[FRONT][(x + 1, y)] == Tile::EMPTY
                {
                    x += 1;
                    break;
                };

                if self.generate_tree(x, y + 1).is_some() {
                    x += 5;
                } else {
                    x += 1;
                }

                break;
            }
        }

        // Surface decor - start at one to avoid placing at the world's edge
        let mut x = 1;

        // While loop is used as the iterator needs to be advanced in loop
        while x < self.width - 2 {
            // Skip if decor should not be generated here
            if self.rng.gen::<f32>() < self.settings.decor.surface_rate {
                x += 1;
                continue;
            }

            for y in (0..self.height).rev() {
                // Find a solid tile
                if self.layers[FRONT][(x, y)] == Tile::EMPTY {
                    continue;
                }

                // Decor doesn't look great on the edge of terrain,
                // So this is checked throughout this loop

                // x .  -> This looks ugly when tiled
                // x x
                // x x x

                // Check the left side of the tile for edges
                if self.layers[FRONT][(x - 1, y)] == Tile::EMPTY {
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

                // TODO: rewrite this with if let Some =

                // Single width tiles can be directly placed
                if desc.dimensions.is_none() {
                    // Check the right side of the tile for edges
                    if self.layers[FRONT][(x + 1, y)] == Tile::EMPTY {
                        x += 1;
                        break;
                    }

                    if self.layers[MIDDLE][(x, y + 1)] != Tile::EMPTY {
                        x += 1;
                        break;
                    }

                    self.layers[MIDDLE][(x, y + 1)] = Tile::new(tile, None);
                    x += 1;
                    break;
                }

                // Check the right side of the multi tile for edges
                if self.layers[FRONT][(x + desc.dimensions.unwrap().0, y)] == Tile::EMPTY {
                    x += 1;
                    break;
                }

                // Check that there is a solid floor beneath the decor
                let mut okay = true;
                for w in 1..desc.dimensions.unwrap().0 {
                    if self.layers[FRONT][(x + w, y)] == Tile::EMPTY {
                        okay = false;
                    }
                }

                if !okay {
                    x += 1;
                    break;
                }

                // Attempt to generate a multi tile
                if let Some(_) = self.generate_multi_tile(tile, x, y + 1) {
                    x += desc.dimensions.unwrap().0;
                } else {
                    x += 1;
                }

                break;
            }
        }

        // Match tiles to their surrounds
        for x in 0..self.width {
            for y in 0..self.height {
                // Foreground
                if self.layers[FRONT][(x, y)].id != TileId::Empty {
                    self.layers[FRONT][(x, y)].texture_offset =
                        Some(self.get_surrounds(FRONT, x, y).get_texture_offset());
                }

                // Background
                if self.layers[BACK][(x, y)].id != TileId::Empty {
                    self.layers[BACK][(x, y)].texture_offset =
                        Some(self.get_surrounds(BACK, x, y).get_texture_offset());
                }
            }
        }

        // Finally
        // Generate pathfinding tiles
        self.generate_path_tiles();
    }

    fn generate_tree(&mut self, x: u32, y: u32) -> Option<()> {
        // Select a random trunk size
        let trunk_height = self
            .rng
            .gen_range(self.settings.trees.trunk_height_range.clone());

        // Generate foliage first, this has to go through multi tile checks
        let foliage = TileDescriptor::from_id(TileId::Tree(Tree::Foliage));

        self.generate_multi_tile(
            foliage.id,
            x - foliage.dimensions.unwrap().0 / 2,
            y + trunk_height,
        )?;

        for h in 0..trunk_height {
            let variant = self
                .rng
                .gen_range(0..self.settings.trees.trunk_variants - 1);

            self.layers[MIDDLE][(x, y + h)] =
                Tile::new(TileId::Tree(Tree::Wood), Some((variant, 0)));
        }

        Some(())
    }

    // Returns None if generation was obsructed
    fn generate_multi_tile(&mut self, id: TileId, x: u32, y: u32) -> Option<()> {
        let desc = TileDescriptor::from_id(id);

        // Unwrap the size. This should panic as a non-multi tile should be caught by devs
        let size = desc
            .dimensions
            .unwrap_or_else(|| panic!("Tile {:?} is not a multi tile", id));

        // Check for obstructions
        for w in 0..size.0 {
            for h in 0..size.1 {
                if self.layers[FRONT][(x + w, y + h)] != Tile::EMPTY
                    || self.layers[MIDDLE][(x + w, y + h)] != Tile::EMPTY
                {
                    return None;
                }
            }
        }

        // All good, generate
        for w in 0..size.0 {
            for h in 0..size.1 {
                self.layers[MIDDLE][(x + w, y + h)] = Tile::new(id, Some((w, size.1 - h - 1)));
            }
        }

        Some(())
    }

    pub fn get_surrounds(&self, layer: usize, x: u32, y: u32) -> Surrounds {
        let mut surrounds = Surrounds::empty();

        // TL
        if let Some(t) = self.layers[layer].get(x as isize - 1, y as isize + 1) && t.id != TileId::Empty {
                surrounds.toggle(Surrounds::TL)
        }

        // TM
        if let Some(t) = self.layers[layer].get(x as isize, y as isize + 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::TM)
        }

        // TR
        if let Some(t) = self.layers[layer].get(x as isize + 1, y as isize + 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::TR)
        }

        // ML
        if let Some(t) = self.layers[layer].get(x as isize - 1, y as isize) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::ML)
        }

        // MR
        if let Some(t) = self.layers[layer].get(x as isize + 1, y as isize) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::MR)
        }

        // BL
        if let Some(t) = self.layers[layer].get(x as isize - 1, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BL)
        }

        // BM
        if let Some(t) = self.layers[layer].get(x as isize, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BM)
        }

        // BR
        if let Some(t) = self.layers[layer].get(x as isize + 1, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BR)
        }

        surrounds
    }

    // Smooth out randomly generated noise by making each tile more similar to it's neighbour
    // Stores the result of the smooth in out to prevent tile_data from being corrupted in use
    fn smooth(&self) -> Layer<Tile> {
        let mut output = Layer::new(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                let w_count = self.get_surrounds(FRONT, x, y).count();

                if w_count >= self.settings.caves.convert_min {
                    output[(x, y)] = Tile::new(TileId::Ground(Ground::Stone), None)
                } else if w_count < self.settings.caves.convert_min {
                    output[(x, y)] = Tile::EMPTY
                }
            }
        }

        output
    }
}
