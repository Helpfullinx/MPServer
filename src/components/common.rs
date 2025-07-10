use bevy_ecs::component::Component;
use bevy_ecs::query::QueryData;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq)]
#[derive(Eq)]
pub struct Id(pub u32);

#[derive(Component, Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
