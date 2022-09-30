use bitflags::bitflags;

// All tiles present in the game
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Tile {
    Null, // Should never be present
    Air,
    Regular(RegularTile),
    Background(BackgroundTile),
}

// Regular (block/square) tiles
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RegularTile {
    Grass,
    Dirt,
    Stone,
}

// Background tiles
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BackgroundTile {
    Dirt,
    Stone,
}

// Info about the tiles surrounding a tile. This is stored as a u8 as it is useful for indexing
// (See texture selection)
//
// Represented as such:
// TL TM TR
// ML ** MR
// BL BM BR
bitflags! {
    pub struct Surrounds: u8 {
        const TL = 0b00000001; // Top left
        const TM = 0b00000010; // Top middle
        const TR = 0b00000100; // Top right
        const ML = 0b00001_000; // Middle left
        const MR = 0b00010_000; // Middle right
        const BL = 0b00100_000; // Bottom left
        const BM = 0b01_000_000; // Bottom middle
        const BR = 0b10_000_000; // Bottom right
    }
}

impl Surrounds {
    // Returns the number of solid blocks surrounding a tile
    pub fn count(&self) -> u32 {
        let mut count = 0;

        for i in 0..8 {
            count += (self.bits & (1 << i)) >> i;
        }

        count as u32
    }
}

pub const TILESET_SIZE: (u32, u32) = (22, 16);

pub fn get_texture_index(tile: &Tile, surrounds: Option<Surrounds>) -> u32 {
    match tile {
        // Non-regular tiles are handled manually
        Tile::Null => TILESET_SIZE.0 * TILESET_SIZE.1 - 1,
        Tile::Air => TILESET_SIZE.0 * TILESET_SIZE.1 - 2,

        // Regular tiles need to have their surrounds taken into account
        Tile::Regular(t) => {
            // Top left coner of texture array
            let mut offset = match t {
                RegularTile::Dirt => 0,
                RegularTile::Grass => 3 * TILESET_SIZE.0,
                RegularTile::Stone => 6 * TILESET_SIZE.0,
            };

            // Offset into array based of surroundings
            if let Some(s) = surrounds {
                offset += TEXT_OFFSETS[s.bits as usize];
            }

            offset
        }

        // The same applies to background tiles, however they
        // use a different tileset (BackgroundTiles.png)
        Tile::Background(t) => {
            let mut offset = match t {
                BackgroundTile::Dirt => 0,
                BackgroundTile::Stone => 3 * TILESET_SIZE.0,
            };

            // Offset into array based of surroundings
            if let Some(s) = surrounds {
                offset += TEXT_OFFSETS[s.bits as usize];
            }

            offset
        }
    }
}

// The textures for all regular tiles have a common layout.
// To determine which texture to choose based off of surrounding
// tiles, the following table can be indexed with a Surrounds struct
// These offsets are all manually calculated. Yes, this is an act of masochism
// This idea was inspired by marching cubes
const TEXT_OFFSETS: [u32; 256] = [
    6, // 0
    6, 48, 48, 6, 6, 48, 48, // 7
    27, // 8
    27, 37, 46, 27, 27, 37, 46, // 15
    25, // 16
    25, 36, 36, 25, 44, 44, 44, // 23
    50, // 24
    50, 43, 41, 50, 50, 39, 45, // 31
    6,  // 32
    6, 48, 48, 6, 6, 48, 48, // 39
    27, // 40
    27, 37, 46, 27, 27, 37, 46, // 47
    25, // 48
    25, 36, 36, 25, 25, 36, 44, // 55
    50, // 56
    50, 43, 41, 50, 50, 39, 45, // 63
    4,  // 64
    4, 28, 28, 4, 4, 28, 28, // 71
    15, // 72
    15, 21, 17, 15, 15, 21, 17, // 79
    14, // 80
    14, 42, 42, 14, 14, 40, 40, // 87
    20, // 88
    20, 26, 32, 20, 20, 32, 11, // 95
    4,  // 96
    4, 28, 28, 4, 4, 28, 28, // 103
    2,  // 104
    2, 19, 24, 2, 2, 19, 24, // 111
    14, // 112
    14, 42, 42, 14, 14, 40, 40, // 119
    16, // 120
    16, 10, 35, 16, 16, 34, 7, // 127
    6, // 128
    6, 48, 48, 6, 6, 48, 48, // 135
    // Good below here
    27, // 136
    27, 37, 46, 27, 27, 37, 46, // 143
    25, // 144
    25, 36, 36, 25, 25, 44, 44, // 151
    50, // 152
    50, 43, 41, 50, 50, 39, 45, // 159
    6,  // 160
    6, 48, 48, 6, 6, 48, 48, // 167
    27, // 168
    27, 37, 46, 27, 27, 39, 46, // 175
    25, // 176
    25, 36, 36, 25, 25, 44, 44, // 183
    50, // 184
    50, 43, 41, 50, 50, 39, 45, // 191
    4,  // 192
    4, 28, 28, 4, 4, 28, 28, // 199
    15, // 200
    15, 21, 17, 15, 15, 21, 46, // 207
    0,  // 208
    0, 38, 38, 0, 0, 22, 22, // 215
    18, // 216
    18, 9, 12, 18, 18, 13, 8, // 223
    4, // 224
    4, 28, 28, 4, 4, 28, 28, // 231
    2,  // 232
    2, 19, 24, 2, 2, 19, 24, // 239
    0,  // 240
    0, 38, 38, 0, 2, 22, 22, // 247
    1,  // 248
    1, 33, 29, 1, 1, 30, 23, // 255
];
