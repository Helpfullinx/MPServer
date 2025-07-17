use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use crate::components::common::Vec3;

#[derive(Component, Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Player {
    pub position: Vec3,
    pub linear_velocity: Vec3
}

impl Player {
    pub fn new(position: Vec3, linear_velocity: Vec3) -> Self {
        Self { 
            position,
            linear_velocity
        }
    }
}
