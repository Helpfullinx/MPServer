use bevy::prelude::{Component, Vec2};

const LOOK_SENSITIVITY: (f32, f32) = (1.0, 1.0);

#[derive(Component, Debug)]
pub struct CameraInfo {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn apply_player_camera_input(mouse_delta: Vec2, camera_info: &mut CameraInfo) {
    camera_info.yaw += -1.0 * LOOK_SENSITIVITY.0 * mouse_delta.x * 0.001;
    camera_info.pitch += 1.0 * LOOK_SENSITIVITY.1 * mouse_delta.y * 0.001;

    camera_info.pitch = camera_info
        .pitch
        .clamp(-90.0f32.to_radians(), 90.0f32.to_radians());
    // println!("camera info: {:?}", camera_info);
}
