use std::f32::INFINITY;

use num_derive::FromPrimitive;

pub const TILESET_SIZE: (u32, u32) = (22, 16);

// #[Component]
// Attached to every tile, used for identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TileId {
    Null, // Should never be present in a functioning world
    Empty,
    Ground(Ground),
    Ore(Ore),
    Background(Background),
    SurfaceDecor(SurfaceDecor),
    Tree(Tree),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ground {
    Grass,
    Dirt,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum Ore {
    Iron,
    Gold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Background {
    Dirt,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum SurfaceDecor {
    GrassSmall,
    Rock,
    GrassMedium,
    RockPile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum Tree {
    Wood,
    Foliage,
}

// Used during world creation and in save files
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    pub id: TileId,

    // Offset from the top left corner of the multi-tile
    pub texture_offset: Option<(u32, u32)>,
}

impl Tile {
    pub const NULL: Self = Self {
        id: TileId::Null,
        texture_offset: None,
    };

    pub const EMPTY: Self = Self {
        id: TileId::Empty,
        texture_offset: None,
    };

    pub fn new(id: TileId, offset: Option<(u32, u32)>) -> Self {
        Self {
            id,
            texture_offset: offset,
        }
    }

    // Returns the index of a this tile in its respective tileset
    pub fn get_texture_index(&self) -> u32 {
        let mut index = TileDescriptor::from_id(self.id).tileset_position;

        if let Some(offset) = self.texture_offset {
            index += offset.1 * TILESET_SIZE.0 + offset.0
        }

        index
    }
}

// Describes ore-specific properties
#[derive(Copy, Clone)]
pub struct OreDescriptor {
    pub max_height: f32,
    pub radius: u32,
}

// Contains a description of every tile
pub struct TileDescriptor {
    pub id: TileId,
    pub tileset_position: u32,

    // Used for structures that take up more than one tile
    pub dimensions: Option<(u32, u32)>,

    // Describes ore related properties
    pub ore: Option<OreDescriptor>,

    // Basic stats
    pub hardness: f32,
}

impl TileDescriptor {
    pub fn from_id(id: TileId) -> &'static Self {
        for desc in &Self::DESCRIPTORS {
            if desc.id == id {
                return desc;
            }
        }

        // None found - panic
        panic!("Error, TileDescriptor missing for tile: {:?}", id);
    }

    // Array of all tiles in the game, this will be replaced with a file
    const DESCRIPTORS: [Self; 15] = [
        Self {
            id: TileId::Null,
            tileset_position: TILESET_SIZE.0 * TILESET_SIZE.1 - 1,
            dimensions: None,
            hardness: INFINITY,
            ore: None,
        },
        Self {
            id: TileId::Empty,
            tileset_position: TILESET_SIZE.0 * TILESET_SIZE.1 - 2,
            dimensions: None,
            hardness: 0.0,
            ore: None,
        },
        Self {
            id: TileId::Ground(Ground::Grass),
            tileset_position: 3 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Ground(Ground::Dirt),
            tileset_position: 0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Ground(Ground::Stone),
            tileset_position: 6 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Ore(Ore::Iron),
            tileset_position: 9 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: Some(OreDescriptor {
                max_height: 1.0,
                radius: 4,
            }),
        },
        Self {
            id: TileId::Ore(Ore::Gold),
            tileset_position: 12 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: Some(OreDescriptor {
                max_height: 0.50,
                radius: 3,
            }),
        },
        Self {
            id: TileId::Background(Background::Dirt),
            tileset_position: 0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Background(Background::Stone),
            tileset_position: 3 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::SurfaceDecor(SurfaceDecor::GrassSmall),
            tileset_position: 1 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::SurfaceDecor(SurfaceDecor::GrassMedium),
            tileset_position: 1,
            dimensions: Some((1, 2)),
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::SurfaceDecor(SurfaceDecor::Rock),
            tileset_position: 3 * TILESET_SIZE.0,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::SurfaceDecor(SurfaceDecor::RockPile),
            tileset_position: 3 * TILESET_SIZE.0 + 1,
            dimensions: Some((2, 1)),
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Tree(Tree::Wood),
            tileset_position: 17,
            dimensions: None,
            hardness: 1.0,
            ore: None,
        },
        Self {
            id: TileId::Tree(Tree::Foliage),
            tileset_position: 1 * TILESET_SIZE.0 + 17,
            dimensions: Some((5, 6)),
            hardness: 1.0,
            ore: None,
        },
    ];
}
