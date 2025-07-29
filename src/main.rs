mod components;
mod network;
mod util;

use std::collections::VecDeque;
use crate::network::net_manage::{Communication, start_tcp_task, start_udp_task};
use crate::network::net_system::{tcp_net_receive, tcp_net_send, udp_net_receive, udp_net_send};
use crate::network::net_tasks::{build_connection_messages, handle_tcp_message, handle_udp_message};
use bevy::prelude::*;
use bincode::{Decode, Encode};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use avian3d::{PhysicsPlugins, PhysicsTypeRegistrationPlugin};
use avian3d::prelude::{Collider, ColliderBackendPlugin, ColliderHierarchyPlugin, ColliderTransformPlugin, Friction, LinearVelocity, MassPropertyPlugin, NarrowPhasePlugin, Physics, PhysicsSchedulePlugin, PhysicsSet, PhysicsTime, PreparePlugin, RigidBody, Sleeping};
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use tokio::net::TcpStream;
use tokio::{io, sync::mpsc};
use crate::components::chat::{send_chat_to_all_connections, Chat};
use crate::components::player::PlayerMarker;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (udp_send_tx, udp_send_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (udp_receive_tx, udp_receive_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (tcp_send_tx, tcp_send_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);
    let (tcp_receive_tx, tcp_receive_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4444);
    println!("Server starting; listening on 0.0.0.0:4444...");

    start_tcp_task(addr, tcp_send_rx, tcp_receive_tx).await?;
    start_udp_task(addr, udp_send_rx, udp_receive_tx, 8).await?;

    App::new()
        .add_plugins((
            MinimalPlugins,
            TransformPlugin::default(),
            AssetPlugin::default(),
            ScenePlugin,
            PhysicsPlugins::default().with_length_unit(10.0)
        ))
        .init_resource::<Assets<Mesh>>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Time::<Physics>::default().with_relative_speed(1.0))
        .insert_resource(
            Communication::new(
                udp_send_tx,
                udp_receive_rx,
                tcp_send_tx,
                tcp_receive_rx,
            )
        )
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            udp_net_receive,
            tcp_net_receive,
            handle_udp_message.after(udp_net_receive),
            handle_tcp_message.after(tcp_net_receive),
            send_chat_to_all_connections.after(handle_tcp_message),
            build_connection_messages.after(handle_udp_message),
            udp_net_send.after(build_connection_messages),
            tcp_net_send.after(tcp_net_receive),
        ))
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(
        Chat {
            chat_history: VecDeque::new()
        }
    );

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(40.0, 0.5, 40.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn debug_player_sleeping(
    sleeping_players: Query<(&LinearVelocity, &PlayerMarker), With<Sleeping>>,
    nonsleeping_players: Query<(&LinearVelocity, &PlayerMarker), Without<Sleeping>>,
) {
    for p in sleeping_players.iter() {
        println!("Sleeping: {:?}", p.0);
    }

    for p in nonsleeping_players.iter() {
        println!("NonSleeping: {:?}", p.0);
    }
}