use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_message::NetworkMessageType::PlayerId;
use crate::network::net_message::{NetworkMessage, NetworkMessageType, TcpMessage};
use crate::network::net_tasks::Connection;
use bevy_ecs::prelude::Commands;
use bincode::config;
use std::sync::Arc;
use tokio::net::TcpStream;
use uuid::Uuid;

pub fn handle_join(bytes: Vec<u8>, commands: &mut Commands, stream: &Arc<TcpStream>) {
    let decoded: (Vec<NetworkMessageType>, usize) =
        bincode::serde::decode_from_slice(&bytes, config::standard()).unwrap();
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
        Id(player_id),
        Connection {
            lobby_id: lid,
            stream: stream.clone(),
        },
    ));

    println!("Player joined:  {:?}", player_id);
    let mut net_message = Vec::new();
    net_message.push(NetworkMessage(PlayerId {
        player_uid: player_id,
    }));

    commands.spawn((
        TcpMessage,
        NetworkMessage(PlayerId {
            player_uid: player_id,
        }),
    ));
    // commands.spawn((TcpMessage, NetworkMessage( NetworkMessageType::Spawn { player_uid: vec![ Id(player_id) ] })));
}
