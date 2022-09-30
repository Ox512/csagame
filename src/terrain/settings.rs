#[derive(Default)]
pub struct GenerationSettings {
    pub surface: SurfaceSettings,
    pub caves: CaveSettings,
    pub stone_height: f32,
    pub stone_blur: u32,
    pub stone_jitter: u32,
    pub background_offset: u32,
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
            falloff: 2.3,
        },

        stone_height: 0.60,
        stone_blur: 18,
        stone_jitter: 3,
        background_offset: 4,
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
