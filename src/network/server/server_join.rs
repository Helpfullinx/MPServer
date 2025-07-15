use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::Commands;
use crate::components::common::{Id, Vec3};
use crate::components::player::Player;
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, TCP};
use crate::util::generate_random_u32;

pub fn handle_join(
    lobby_id: Id,
    connection: &mut TcpConnection,
    commands: &mut Commands
) {
    println!("Trying to join lobby: {:?}", lobby_id);
    // Generate an ID
    
    let player_id = generate_random_u32();
    
    println!("Player joined: {:?}", player_id);

    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        Player::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0)
        ),
        Id(player_id)
    ));
    
    connection.output_message.push(NetworkMessage(TCP::PlayerId {
        player_uid: Id(player_id),
    }));
}
