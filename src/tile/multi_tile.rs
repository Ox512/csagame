use crate::tile::*;

// Information about structures that consist of multiple tiles, eg: trees
pub struct MultiTileInfo {
    pub width: u32,
    pub height: u32,
    pub tile: Tile,
}

// All different multitile structure in the game. Used
// to index into MULTI_TILES array
pub enum MultiTile {
    GrassMedium = 0,
}

pub const MULTI_TILES: [MultiTileInfo; 1] = [MultiTileInfo {
    width: 1,
    height: 2,
    tile: Tile::Decor(DecorTile::GrassMedium(0, 0)),
}];
