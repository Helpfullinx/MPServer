use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, TCP};
use bevy_ecs::prelude::Commands;
use bincode::config;
use uuid::Uuid;

pub fn handle_join(
    lobby_id: u128,
    connection: &mut TcpConnection,
    commands: &mut Commands
) {
    println!("Trying to join lobby: {:?}", lobby_id);
    // Generate an ID
    let player_id = Uuid::new_v4().as_u128();
    
    println!("Player joined:  {:?}", player_id);

    commands.spawn((PlayerBundle::new(Position::new(0.0, 0.0)), Id(player_id)));
    
    connection.output_message.push(NetworkMessage(TCP::PlayerId {
        player_uid: player_id,
    }));
}
