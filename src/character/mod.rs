pub mod animation;
pub mod collision;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use self::animation::*;
use self::collision::WorldCollider;

#[derive(Bundle)]
pub struct CharacterBundle {
    id: CharacterId,
    collider: Collider,
    world_collider: WorldCollider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    sprite: SpriteSheetBundle,
    timer: AnimationTimer,
    state: AnimationState,
}

impl CharacterBundle {
    pub fn from_id(id: CharacterId, pos: Vec2, handles: &SpriteSheetHandles) -> Self {
        let desc = CharacterDesc::from_id(id);

        Self {
            id,
            collider: Collider::cuboid(desc.col_size.0, desc.col_size.1),
            world_collider: WorldCollider::default(),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: desc.sheet_size.0 * AnimationState::Walking as usize,
                    ..Default::default()
                },
                texture_atlas: handles[id as usize].clone(),
                transform: Transform::from_translation(pos.extend(-1.0)),
                ..Default::default()
            },
            timer: AnimationTimer::new(0.1),
            state: AnimationState::Walking,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub enum CharacterId {
    HumanMale = 0,
}

// Describes the properties of a character
// Same as TileDescriptor system
pub struct CharacterDesc {
    pub id: CharacterId,
    pub sprite_sheet: &'static str,
    pub sprite_size: Vec2,
    pub sheet_size: (usize, usize),
    pub col_size: (f32, f32),
}

impl CharacterDesc {
    pub fn from_id(id: CharacterId) -> &'static Self {
        &Self::DESCRIPTORS[id as usize]
    }

    const DESCRIPTORS: [Self; 1] = [Self {
        id: CharacterId::HumanMale,
        sprite_sheet: "Characters/HumanMale1.png",
        sprite_size: Vec2::new(16.0, 26.0),
        sheet_size: (8, 3),
        col_size: (8.0, 12.0),
    }];
}
