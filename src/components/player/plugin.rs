use bevy::app::App;
use bevy::prelude::{FixedUpdate, Plugin};
use crate::components::player::apply_player_inputs;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_player_inputs
        );
    }
}