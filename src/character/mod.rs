pub mod collision;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::collision::WorldCollider;

pub fn setup_entity(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(16.0))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)))
        .insert(WorldCollider::default());

    commands
        .spawn_empty()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(16.0))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(
            150.0, 200.0, 0.0,
        )))
        .insert(WorldCollider::default());
}
