use avian3d::prelude::LinearVelocity;
use bevy::prelude::{Query, Transform};
use crate::components::common::Id;
use crate::network::net_message::BitMask;
use crate::components::player::Player;

pub fn handle_input(
    keymask: BitMask,
    playerid: Id,
    players: &mut Query<(&Id, &mut Transform, &LinearVelocity)>
) {
    let move_speed = 0.1;

    for (id, mut player, lv) in players.iter_mut() {
        if *id == playerid {
            if keymask & 1 > 0 {
                player.translation.x -= move_speed;
            }
            if keymask & 2 > 0 {
                player.translation.x += move_speed;
            }
            if keymask & 4 > 0 {
                player.translation.z -= move_speed;
            }
            if keymask & 8 > 0 {
                player.translation.z += move_speed;
            }
        }
        println!("player: {:?}", player);
    }
}
