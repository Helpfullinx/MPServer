use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_message::{BitMask, NetworkMessage, NetworkMessageType, UdpMessage};
use bevy_ecs::prelude::{Commands, Query};
use std::collections::HashMap;

pub fn handle_input(
    keymask: BitMask,
    playerid: u128,
    players: &mut Query<(&Id, &mut Position)>,
    commands: &mut Commands,
) {
    let move_speed = 3.0;

    let mut all_pos = HashMap::new();

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
            if keymask != 0 {
                all_pos.insert(id.0, PlayerBundle::new(*pos));
            }
        }
    }

    commands.spawn((
        UdpMessage,
        NetworkMessage(NetworkMessageType::Players { players: all_pos }),
    ));
}
