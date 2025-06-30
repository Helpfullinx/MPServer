use crate::components::common::{Id, Position};
use bevy_ecs::bundle::Bundle;
use serde::{Deserialize, Serialize};

#[derive(Bundle, Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct PlayerBundle {
    pub position: Position,
}

impl PlayerBundle {
    pub fn new(position: Position) -> Self {
        Self { position }
    }
}
