use bevy::prelude::*;

pub const CAMERA_MOVE_SPEED: f32 = 250.0; // Distance per second

#[derive(Resource)]
pub struct CursorPos(pub Vec2);

// Contructs the camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::default().with_scale(Vec3::new(0.5, 0.5, 1.0)),
        ..Default::default()
    });
}

// Moves the camera in response to WASD input
pub fn move_camera(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    // Speed to rounded to position is always whole - prevents half-pixels
    let speed = (time.delta_seconds() * CAMERA_MOVE_SPEED).round();
    let mut dir = Vec3::new(0.0, 0.0, 0.0);
    let mut transform = query.single_mut();

    if input.pressed(KeyCode::W) {
        dir += Vec3::new(0.0, 1.0, 0.0);
    }

    if input.pressed(KeyCode::A) {
        dir += Vec3::new(-1.0, 0.0, 0.0);
    }

    if input.pressed(KeyCode::S) {
        dir += Vec3::new(0.0, -1.0, 0.0);
    }

    if input.pressed(KeyCode::D) {
        dir += Vec3::new(1.0, 0.0, 0.0);
    }

    if dir != Vec3::ZERO {
        transform.translation += dir.normalize() * speed;
    }
}

pub fn update_cursor_pos(
    windows: Res<Windows>,
    cam_query: Query<(&Transform, &Camera)>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let window = windows.primary();
    let (transform, cam) = cam_query.single();

    // Cursor pos is only calculated if cursor is inside game window
    if let Some(screen_pos) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());

        // Convert to ndc co-ords [-1..1] then to world co-ords
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_word = transform.compute_matrix() * cam.projection_matrix().inverse();

        *cursor_pos = CursorPos(ndc_to_word.project_point3(ndc.extend(0.0)).truncate());
    }
}
