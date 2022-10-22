use std::ops::Range;

#[derive(Default)]
pub struct GenerationSettings {
    pub surface: SurfaceSettings,
    pub caves: CaveSettings,
    pub decor: DecorSettings,
    pub trees: TreeSettings,

    pub dirt_height: f32,

    // Affects the change between stone and dirt
    pub stone_blur: u32,
    pub stone_jitter: u32,

    pub background_offset: u32,

    pub ore_height: f32,

    // Lower = more frequent ore spawn rates
    pub ore_rate: u32,
}

impl GenerationSettings {
    pub const FOREST: Self = Self {
        surface: SurfaceSettings {
            amplitude: 24.0,
            scale: 2.0,
            persistence: 0.5,
            lacunarity: 0.5,
            octaves: 6,
            height_offset: 0.75,
        },

        caves: CaveSettings {
            solid_density: 0.1,
            smooth_iters: 4,
            convert_min: 4,
            falloff: 2.0,
        },

        decor: DecorSettings {
            surface: 0..4,
            surface_rate: 0.4,
        },

        trees: TreeSettings {
            trunk_height_range: 3..5,
            trunk_variants: 5,
            spawn_rate: 0.4,
        },

        dirt_height: 0.60,
        stone_blur: 18,
        stone_jitter: 6,
        background_offset: 2,
        ore_height: 0.575,
        ore_rate: 3,
    };
}

#[derive(Default)]
pub struct SurfaceSettings {
    // Perlin noise values
    pub scale: f32,
    pub amplitude: f32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub octaves: usize,

    pub height_offset: f32, // Offset at which height will start [0; 1] (percentage based on world size)
}

#[derive(Default)]
pub struct CaveSettings {
    pub solid_density: f32, // [-1; 1] Density at which a tile is considered solid
    pub smooth_iters: u32,  // (0; 8] No. of times to smooth cave gen
    pub convert_min: u32,   // [0; 8] No. of like neighbours required to convert a tile
    pub falloff: f32,
}

#[derive(Default)]
pub struct DecorSettings {
    pub surface: Range<usize>, // Single tile decor
    pub surface_rate: f32,
}

#[derive(Default)]
pub struct TreeSettings {
    pub trunk_height_range: Range<u32>, // Possible sizes of tree trunks
    pub trunk_variants: u32,            // Different trunk tile variants
    pub spawn_rate: f32,
}
