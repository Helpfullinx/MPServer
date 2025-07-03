use crate::components::common::{Id, Position};
use crate::network::net_message::BitMask;
use bevy_ecs::prelude::Query;

pub fn handle_input(keymask: BitMask, playerid: u128, players: &mut Query<(&Id, &mut Position)>) {
    let move_speed = 0.1;

    for (id, mut pos) in players.iter_mut() {
        if id.0 == playerid {
            if keymask & 1 > 0 {
                pos.y += move_speed;
            }
            if keymask & 2 > 0 {
                pos.y -= move_speed;
            }
            if keymask & 4 > 0 {
                pos.x += move_speed;
            }
            if keymask & 8 > 0 {
                pos.x -= move_speed;
            }
        }
    }
}
