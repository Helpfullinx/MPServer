use std::time::{SystemTime, UNIX_EPOCH};

use crate::Communication;
use crate::components::common::{Id, Position};
use crate::network::net_manage::{TcpPacket, UdpPacket};
use crate::network::net_message::{NetworkMessage, TcpMessage, UdpMessage, build_message};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Query, With};
use bincode::config;
use tokio::sync::mpsc::error::{TryRecvError, TrySendError};
use crate::network::server::server_join::handle_join;

pub fn udp_net_receive(
    mut comm: ResMut<Communication>,
    mut commands: Commands,
) {
    let mut udp_packet = None;
    while !comm.udp_rx.is_empty() {
        match comm.udp_rx.try_recv() {
            Ok((bytes, addr)) => {
                udp_packet = Some(UdpPacket { bytes: bytes, addr });
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    if let Some(p) = udp_packet { commands.spawn(p); }
}

pub fn udp_net_send(
    comm: ResMut<Communication>,
    mut packets: Query<(Entity, &UdpPacket)>,
    messages: Query<(Entity, &NetworkMessage), With<UdpMessage>>,
    mut commands: Commands,
) {
    // Takes in all NetworkMessage that have been added to ECS and builds Network
    let net_message = build_message(messages.into_iter().collect(), &mut commands);
    let message = bincode::serde::encode_to_vec(net_message, config::standard()).unwrap();

    for (e, packet) in packets.iter_mut() {
        match comm.udp_tx.try_send((message.clone(), packet.addr)) {
            Ok(()) => {
                commands.entity(e).remove::<UdpPacket>();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break,
        }
    }
}

pub fn tcp_net_receive(mut commands: Commands, mut comm: ResMut<Communication>) {
    // TODO: There should be a global lobby hashmap that will hold a bunch of these hashsets, and then figure out a way to work with that
    // Hashset of all player uuid's in lobby
    // let mut players: HashSet<Uuid> = HashSet::new();

    while !comm.tcp_rx.is_empty() {
        match comm.tcp_rx.try_recv() {
            Ok((bytes, stream)) => {
                // Decodes lobby_id and returns back a player_id
                handle_join(bytes, &mut commands, &stream);

                commands.spawn(TcpPacket {
                    bytes: vec![],
                    tcp_stream: stream,
                });
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
}

pub fn tcp_net_send(
    comm: ResMut<Communication>,
    mut packets: Query<(Entity, &TcpPacket)>,
    messages: Query<(Entity, &NetworkMessage), With<TcpMessage>>,
    mut commands: Commands,
) {
    let net_message = build_message(messages.into_iter().collect(), &mut commands);
    let message = bincode::serde::encode_to_vec(net_message, config::standard()).unwrap();

    for (e, packet) in packets.iter_mut() {
        match comm
            .tcp_tx
            .try_send((message.clone(), packet.tcp_stream.clone()))
        {
            Ok(()) => {
                commands.entity(e).remove::<TcpPacket>();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break,
        }
    }
}
