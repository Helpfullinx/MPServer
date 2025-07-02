use std::collections::VecDeque;
use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_message::NetworkMessageType::PlayerId;
use crate::network::net_message::{NetworkMessage, NetworkMessageType, TcpMessage};
use bevy_ecs::prelude::Commands;
use bincode::config;
use uuid::Uuid;
use crate::network::net_manage::TcpConnection;

pub fn handle_join(
    connection: &mut TcpConnection,
    bytes: &Vec<u8>,
    commands: &mut Commands,
) {
    let decoded: (Vec<NetworkMessageType>, usize) =
        bincode::serde::decode_from_slice(bytes, config::standard()).unwrap();
    let mut lid = 0;
    for m in decoded.0 {
        match m {
            NetworkMessageType::Join { lobby_id } => {
                lid = lobby_id;
            }
            _ => {}
        }
    }

    println!("Trying to join lobby: {:?}", lid);

    // Generate an ID
    let player_id = Uuid::new_v4().as_u128();

    commands.spawn((
        PlayerBundle::new(Position::new(0.0, 0.0)),
        Id(player_id)
    ));

    println!("Player joined:  {:?}", player_id);
    
    connection.output_message.push(NetworkMessage(PlayerId { player_uid: player_id }));
}
