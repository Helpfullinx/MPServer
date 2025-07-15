use bevy::prelude::Component;
use serde::{Deserialize, Serialize};
use crate::components::common::Vec3;

#[derive(Component, Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Player {
    pub position: Vec3,
    pub angular_velocity: Vec3
}

impl Player {
    pub fn new(position: Vec3, angular_velocity: Vec3) -> Self {
        Self { 
            position,
            angular_velocity
        }
    }
}
