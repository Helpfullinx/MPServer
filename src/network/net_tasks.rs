use std::collections::HashMap;
use std::sync::Arc;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Query};
use bincode::config;
use tokio::net::TcpStream;
use uuid::Uuid;
use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_message::{NetworkMessage, NetworkMessageType, TcpMessage, UdpMessage};
use crate::network::net_message::NetworkMessageType::{PlayerId, Sequence};

#[derive(Component)]
pub struct Connection {
    lobby_id: u128,
    stream: Arc<TcpStream>,
}

pub fn handle_join(bytes: Vec<u8>, commands: &mut Commands, stream: &Arc<TcpStream>) {
    let decoded: (Vec<NetworkMessageType>, usize) = bincode::serde::decode_from_slice(&bytes, config::standard()).unwrap();
    let mut lid = 0;
    for m in decoded.0 {
        match m {
            NetworkMessageType::Join {lobby_id} => {
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
        }
    ));

    println!("Player joined:  {:?}", player_id);
    let mut net_message = Vec::new();
    net_message.push(NetworkMessage(PlayerId{ player_uid: player_id}));
    
    commands.spawn((TcpMessage, NetworkMessage(PlayerId { player_uid: player_id })));
    // commands.spawn((TcpMessage, NetworkMessage( NetworkMessageType::Spawn { player_uid: vec![ Id(player_id) ] })));
}

pub fn handle_disconnect() {
    
}

pub fn handle_input(
    message: Vec<NetworkMessageType>,
    query: &mut Query<(&Id, &mut Position)>,
    commands: &mut Commands
) {
    let move_speed = 3.0;
    
    let mut all_pos = HashMap::new();

    for m in message {
        match m {
            NetworkMessageType::Input { keymask, player_id: playerid } => {
                for (id, mut pos) in query.iter_mut() {
                    if id.0 == playerid {
                        if keymask & 1 > 0 { pos.y += move_speed; }
                        if keymask & 2 > 0 { pos.y -= move_speed; }
                        if keymask & 4 > 0 { pos.x += move_speed; }
                        if keymask & 8 > 0 { pos.x -= move_speed; }
                        if keymask != 0 { all_pos.insert(id.0, PlayerBundle::new(*pos)); }
                    }
                }
            }
            _ => {}
        }
    }
    
    commands.spawn((UdpMessage, NetworkMessage(NetworkMessageType::Players { players: all_pos })));
}