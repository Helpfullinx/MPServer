use avian3d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::{Commands, Transform};
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
        Collider::cuboid(1.0,1.0,1.0),
        LockedAxes::new().lock_rotation_x().lock_rotation_y().lock_rotation_z(),
        Transform::from_xyz(0.0,0.5,0.0),
        Id(player_id)
    ));
    
    connection.output_message.push(NetworkMessage(TCP::PlayerId {
        player_uid: Id(player_id),
    }));
}
