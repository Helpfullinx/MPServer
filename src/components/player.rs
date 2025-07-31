use crate::components::camera::CameraInfo;
use crate::components::common::{Id, Vec3};
use crate::network::net_message::BitMask;
use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::math::Quat;
use bevy::prelude::EulerRot::YXZ;
use bevy::prelude::{Component, Query, Transform, With};
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component, Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Player {
    pub position: Vec3,
    pub linear_velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Player {
    pub fn new(position: Vec3, linear_velocity: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            linear_velocity,
            yaw,
            pitch,
        }
    }
}

const MOVE_SPEED: f32 = 5.0;

pub fn apply_player_movement_input(
    encoded_input: BitMask,
    linear_velocity: &mut LinearVelocity,
    rotation: &mut Rotation,
    yaw: &f32,
) {
    let mut vector = bevy::math::Vec3::ZERO;

    if encoded_input & 1 > 0 {
        vector.z -= 1.0;
    }
    if encoded_input & 2 > 0 {
        vector.z += 1.0;
    }
    if encoded_input & 4 > 0 {
        vector.x += 1.0;
    }
    if encoded_input & 8 > 0 {
        vector.x -= 1.0;
    }
    if encoded_input & 16 > 0 {
        linear_velocity.y += 1.0;
    }

    let normalized_rotated_velocity =
        Quat::from_euler(YXZ, *yaw, 0.0, 0.0).mul_vec3(vector.normalize_or_zero());

    linear_velocity.x = normalized_rotated_velocity.x * MOVE_SPEED;
    linear_velocity.z = normalized_rotated_velocity.z * MOVE_SPEED;

    rotation.0 = Quat::from_euler(YXZ, *yaw, 0.0, 0.0);
}
