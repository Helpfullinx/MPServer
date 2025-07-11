use bevy_ecs::component::{Component, StorageType};
use bevy_ecs::query::QueryData;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq)]
#[derive(Eq)]
pub struct Id(pub u32);

#[derive(Component, Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}


impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}