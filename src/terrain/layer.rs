use std::ops::Index;
use std::ops::IndexMut;

use crate::surrounds::*;
use crate::tile::*;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Layer {
    tiles: Vec<Vec<Tile>>,

    pub width: u32,
    pub height: u32,
}

impl Layer {
    // Used for IDs and indexing
    pub const FRONT: usize = 0;
    pub const MIDDLE: usize = 1;
    pub const BACK: usize = 2;
    pub const TOTAL_LAYERS: usize = 3;

    // Creates a Layer filled with Null tiles
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::with_capacity(width as usize);
        for x in 0..width {
            tiles.push(Vec::with_capacity(height as usize));

            for _ in 0..height {
                tiles[x as usize].push(Tile::NULL);
            }
        }

        Self {
            tiles,
            width,
            height,
        }
    }

    // Runs a function on all tiles within a radius
    // Note: This doesn't do accurate radius checking, just a square
    pub fn modify_in_radius<F: FnMut(&mut Tile, f32)>(
        &mut self,
        pos: (u32, u32),
        radius: u32,
        mut func: F,
    ) {
        for w in (pos.0 as isize - radius as isize)..(pos.0 as isize + radius as isize) {
            for h in (pos.1 as isize - radius as isize)..(pos.1 as isize + radius as isize) {
                if let Some(tile) = self.get_tile_mut_checked(w, h) {
                    // Pass the distance to func
                    let dist = ((pos.0 as f32 - w as f32).powf(2.0)
                        + (pos.1 as f32 - h as f32).powf(2.0))
                    .sqrt();

                    func(tile, dist)
                }
            }
        }
    }

    // The following two functions take an isize instead of a u32, this is
    // for cases such as x - 1, which could be negative, the functions
    // check for negatives and return None. isize is used as it can fit
    // the whole range of u32

    fn get_tile_checked(&self, x: isize, y: isize) -> Option<&Tile> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&self.tiles[x as usize][y as usize])
        } else {
            None
        }
    }

    fn get_tile_mut_checked(&mut self, x: isize, y: isize) -> Option<&mut Tile> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&mut self.tiles[x as usize][y as usize])
        } else {
            None
        }
    }

    pub fn get_surrounds(&self, x: u32, y: u32) -> Surrounds {
        let mut surrounds = Surrounds::empty();

        // TL
        if let Some(t) = self.get_tile_checked(x as isize - 1, y as isize + 1) && t.id != TileId::Empty {
                surrounds.toggle(Surrounds::TL)
        }

        // TM
        if let Some(t) = self.get_tile_checked(x as isize, y as isize + 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::TM)
        }

        // TR
        if let Some(t) = self.get_tile_checked(x as isize + 1, y as isize + 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::TR)
        }

        // ML
        if let Some(t) = self.get_tile_checked(x as isize - 1, y as isize) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::ML)
        }

        // MR
        if let Some(t) = self.get_tile_checked(x as isize + 1, y as isize) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::MR)
        }

        // BL
        if let Some(t) = self.get_tile_checked(x as isize - 1, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BL)
        }

        // BM
        if let Some(t) = self.get_tile_checked(x as isize, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BM)
        }

        // BR
        if let Some(t) = self.get_tile_checked(x as isize + 1, y as isize - 1) && t.id != TileId::Empty {
            surrounds.toggle(Surrounds::BR)
        }

        surrounds
    }
}

impl Index<(u32, u32)> for Layer {
    type Output = Tile;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        return &self.tiles[index.0 as usize][index.1 as usize];
    }
}

impl IndexMut<(u32, u32)> for Layer {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        return &mut self.tiles[index.0 as usize][index.1 as usize];
    }
}
