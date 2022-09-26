use bevy::prelude::*;

// Module containing all controls related to moving the camera

pub const CAMERA_MOVE_SPEED: f32 = 75.0; // Distance per second

// Contructs the camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::new_with_far(1.0));
}

// Moves the camera in response to WASD input
pub fn move_camera(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let speed = time.delta_seconds() * CAMERA_MOVE_SPEED;
    let mut dir = Vec3::new(0.0, 0.0, 0.0);
    let mut transform = query.single_mut();

    if input.pressed(KeyCode::W) {
        dir += Vec3::new(0.0, speed, 0.0);
    }

    if input.pressed(KeyCode::A) {
        dir += Vec3::new(-speed, 0.0, 0.0);
    }

    if input.pressed(KeyCode::S) {
        dir += Vec3::new(0.0, -speed, 0.0);
    }

    if input.pressed(KeyCode::D) {
        dir += Vec3::new(speed, 0.0, 0.0);
    }

    if dir != Vec3::ZERO {
        transform.translation += dir;
    }
}
