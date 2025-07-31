use crate::components::camera::CameraInfo;
use crate::components::common::Id;
use crate::components::player::PlayerMarker;
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, TCP};
use crate::util::generate_random_u32;
use avian3d::prelude::{Collider, Friction, LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::{Commands, Transform};

pub fn handle_join(lobby_id: Id, connection: &mut TcpConnection, commands: &mut Commands) {
    println!("Trying to join lobby: {:?}", lobby_id);
    // Generate an ID

    let player_id = generate_random_u32();

    println!("Player joined: {:?}", player_id);

    commands.spawn((
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        Friction::new(1.0),
        LinearVelocity::default(),
        LockedAxes::new()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
        Transform::from_xyz(0.0, 3.0, 0.0),
        CameraInfo {
            yaw: 0.0,
            pitch: 0.0,
        },
        Id(player_id),
        PlayerMarker,
    ));

    connection.add_message(NetworkMessage(TCP::PlayerId {
        player_uid: Id(player_id),
    }));
}
