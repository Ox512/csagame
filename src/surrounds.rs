use bitflags::bitflags;

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
        const ML = 0b00001_000; // Decor left
        const MR = 0b00010_000; // Decor right
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

    pub fn get_texture_offset(&self) -> (u32, u32) {
        TEXT_OFFSETS[self.bits() as usize]
    }
}

const TEXT_OFFSETS: [(u32, u32); 256] = [
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (3, 1),
    (3, 1),
    (14, 1),
    (14, 1),
    (3, 1),
    (0, 2),
    (0, 2),
    (0, 2),
    (6, 2),
    (6, 2),
    (21, 1),
    (19, 1),
    (6, 2),
    (6, 2),
    (17, 1),
    (1, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (3, 1),
    (3, 1),
    (14, 1),
    (14, 1),
    (3, 1),
    (3, 1),
    (14, 1),
    (0, 2),
    (6, 2),
    (6, 2),
    (21, 1),
    (19, 1),
    (6, 2),
    (6, 2),
    (17, 1),
    (1, 2),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (15, 0),
    (15, 0),
    (21, 0),
    (17, 0),
    (15, 0),
    (15, 0),
    (21, 0),
    (17, 0),
    (14, 0),
    (14, 0),
    (20, 1),
    (20, 1),
    (14, 0),
    (14, 0),
    (18, 1),
    (18, 1),
    (20, 0),
    (20, 0),
    (4, 1),
    (10, 1),
    (20, 0),
    (20, 0),
    (10, 1),
    (11, 0),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (2, 0),
    (2, 0),
    (19, 0),
    (2, 1),
    (2, 0),
    (2, 0),
    (19, 0),
    (2, 1),
    (14, 0),
    (14, 0),
    (20, 1),
    (20, 1),
    (14, 0),
    (14, 0),
    (18, 1),
    (18, 1),
    (16, 0),
    (16, 0),
    (10, 0),
    (13, 1),
    (16, 0),
    (16, 0),
    (12, 1),
    (7, 0),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (3, 1),
    (3, 1),
    (14, 1),
    (14, 1),
    (3, 1),
    (3, 1),
    (0, 2),
    (0, 2),
    (6, 2),
    (6, 2),
    (21, 1),
    (19, 1),
    (6, 2),
    (6, 2),
    (17, 1),
    (1, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (6, 0),
    (6, 0),
    (4, 2),
    (4, 2),
    (5, 1),
    (5, 1),
    (15, 1),
    (2, 2),
    (5, 1),
    (5, 1),
    (17, 1),
    (2, 2),
    (3, 1),
    (3, 1),
    (14, 1),
    (14, 1),
    (3, 1),
    (3, 1),
    (0, 2),
    (0, 2),
    (6, 2),
    (6, 2),
    (21, 1),
    (19, 1),
    (6, 2),
    (6, 2),
    (17, 1),
    (1, 2),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (15, 0),
    (15, 0),
    (21, 0),
    (17, 0),
    (15, 0),
    (15, 0),
    (21, 0),
    (2, 2),
    (0, 0),
    (0, 0),
    (16, 1),
    (16, 1),
    (0, 0),
    (0, 0),
    (0, 1),
    (0, 1),
    (18, 0),
    (17, 0),
    (9, 0),
    (12, 0),
    (18, 0),
    (17, 0),
    (13, 0),
    (8, 0),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (4, 0),
    (4, 0),
    (6, 1),
    (6, 1),
    (2, 0),
    (2, 0),
    (19, 0),
    (2, 1),
    (2, 0),
    (2, 0),
    (19, 0),
    (2, 1),
    (0, 0),
    (0, 0),
    (16, 1),
    (16, 1),
    (0, 0),
    (2, 0),
    (0, 1),
    (0, 1),
    (1, 0),
    (1, 0),
    (11, 1),
    (7, 1),
    (1, 0),
    (1, 0),
    (8, 1),
    (1, 1),
];