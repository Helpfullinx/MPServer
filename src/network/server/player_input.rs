use crate::components::common::Id;
use crate::network::net_message::BitMask;
use bevy_ecs::prelude::Query;
use crate::components::player::Player;

pub fn handle_input(keymask: BitMask, playerid: Id, players: &mut Query<(&Id, &mut Player)>) {
    let move_speed = 0.1;

    for (id, mut player) in players.iter_mut() {
        if *id == playerid {
            if keymask & 1 > 0 {
                player.position.x -= move_speed;
            }
            if keymask & 2 > 0 {
                player.position.x += move_speed;
            }
            if keymask & 4 > 0 {
                player.position.z -= move_speed;
            }
            if keymask & 8 > 0 {
                player.position.z += move_speed;
            }
        }
    }
}
