use std::collections::HashSet;
use crate::components::common::Vec3;
use crate::network::net_message::BitMask;
use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::math::Quat;
use bevy::prelude::EulerRot::YXZ;
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct PlayerMovementState(pub HashSet<MovementState>);

pub enum MovementState {
    Idle,
    Walking,
    Running,
}


#[derive(Component, Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Player {
    pub position: Vec3,
    pub linear_velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub animation_state: AnimationState
}

impl Player {
    pub fn new(position: Vec3, linear_velocity: Vec3, yaw: f32, pitch: f32, animation_state: AnimationState) -> Self {
        Self {
            position,
            linear_velocity,
            yaw,
            pitch,
            animation_state,
        }
    }
}

const WALK_SPEED: f32 = 1.5;
const RUN_SPEED: f32 = 5.0;

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

    linear_velocity.x = normalized_rotated_velocity.x * WALK_SPEED;
    linear_velocity.z = normalized_rotated_velocity.z * WALK_SPEED;

    rotation.0 = Quat::from_euler(YXZ, *yaw, 0.0, 0.0);
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
}

#[derive(Component)]
pub struct PlayerAnimationState(pub AnimationState);