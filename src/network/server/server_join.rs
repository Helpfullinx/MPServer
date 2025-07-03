use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, TCP};
use bevy_ecs::prelude::Commands;
use bincode::config;
use uuid::Uuid;

pub fn handle_join(connection: &mut TcpConnection, bytes: &Vec<u8>, commands: &mut Commands) {
    let decoded: (Vec<TCP>, usize) =
        bincode::serde::decode_from_slice(bytes, config::standard()).unwrap();
    for m in decoded.0 {
        match m {
            TCP::Join { lobby_id } => {
                println!("Trying to join lobby: {:?}", lobby_id);
                // Generate an ID
                let player_id = Uuid::new_v4().as_u128();

                commands.spawn((PlayerBundle::new(Position::new(0.0, 0.0)), Id(player_id)));

                println!("Player joined:  {:?}", player_id);

                connection.output_message.push(NetworkMessage(TCP::PlayerId {
                    player_uid: player_id,
                }));
            }
            _ => {}
        }
    }
}
