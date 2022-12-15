use bevy::prelude::*;

use super::{CharacterId, CharacterDesc};

#[derive(Resource, Deref, DerefMut)]
pub struct SpriteSheetHandles(pub Vec<Handle<TextureAtlas>>);

// Load all the sprite sheets at init and store their handles.
// This removes the need to load them every time an entity is spawned
pub fn setup_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut handles = Vec::new();

    for desc in CharacterDesc::DESCRIPTORS {
        let texture_handle = asset_server.load(desc.sprite_sheet);
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            desc.sprite_size,
            desc.sheet_size.0,
            desc.sheet_size.1,
            None,
            None,
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        handles.push(texture_atlas_handle);
    }

    commands.insert_resource(SpriteSheetHandles(handles))
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

impl AnimationTimer {
    pub fn new(period: f32) -> Self {
        Self(Timer::from_seconds(period, TimerMode::Repeating))
    }
}

#[derive(Component)]
pub enum AnimationState {
    Idle = 0,
    Walking = 1,
    ToolSwing = 2,
}

// The number of frames in each animation
const ANIMATION_LENGTHS: [usize; 3] = [
    2,
    8,
    4,
];

pub fn update_animations(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &AnimationState,
        &CharacterId
    )>,
) {
    for (mut timer, mut sprite, state, id) in &mut query {
        timer.tick(time.delta());

        let desc = CharacterDesc::from_id(*id);

        if timer.just_finished() {
            let state_off = (*state as usize) * desc.sheet_size.0;

            if sprite.index - state_off == ANIMATION_LENGTHS[*state as usize] - 1 {
                sprite.index = state_off
            } else {
                sprite.index += 1;
            }
        }
    }
}