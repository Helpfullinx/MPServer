use crate::components::chat::send_chat_to_all_connections;
use crate::network::net_manage::{Communication, start_tcp_task, start_udp_task};
use crate::network::net_system::{tcp_net_receive, tcp_net_send, udp_net_receive, udp_net_send};
use crate::network::net_tasks::{
    build_connection_messages, handle_tcp_message, handle_udp_message,
};
use bevy::app::{App, FixedPostUpdate, FixedPreUpdate, PreStartup};
use bevy::prelude::{Commands, IntoScheduleConfigs, Plugin, Res};
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub mod client;
pub mod net_encoding;
pub mod net_manage;
pub mod net_message;
pub mod net_system;
pub mod net_tasks;
pub mod net_utils;
pub mod server;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TokioTasksPlugin::default())
            .add_systems(PreStartup, setup_communications)
            .add_systems(
                FixedPreUpdate,
                (
                    udp_net_receive,
                    tcp_net_receive,
                    handle_udp_message.after(udp_net_receive),
                    handle_tcp_message.after(tcp_net_receive),
                    send_chat_to_all_connections.after(handle_tcp_message),
                    build_connection_messages.after(handle_udp_message),
                ),
            )
            .add_systems(
                FixedPostUpdate,
                (
                    udp_net_send.after(build_connection_messages),
                    tcp_net_send.after(tcp_net_receive),
                ),
            );
    }
}

fn setup_communications(mut commands: Commands, runtime: Res<TokioTasksRuntime>) {
    let (udp_send_tx, udp_send_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (udp_receive_tx, udp_receive_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (tcp_send_tx, tcp_send_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);
    let (tcp_receive_tx, tcp_receive_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);

    runtime.spawn_background_task(|_| async move {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4444);
        println!("Server starting; listening on 0.0.0.0:4444...");

        start_tcp_task(addr, tcp_send_rx, tcp_receive_tx)
            .await
            .unwrap();
        start_udp_task(addr, udp_send_rx, udp_receive_tx, 8)
            .await
            .unwrap();
    });

    commands.insert_resource(Communication::new(
        udp_send_tx,
        udp_receive_rx,
        tcp_send_tx,
        tcp_receive_rx,
    ))
}
