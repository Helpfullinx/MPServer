use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, Query, With};
use bincode::{config, encode_into_slice};
use tokio::sync::mpsc::error::{TryRecvError, TrySendError};
use crate::Communication;
use crate::components::common::{Id, Position};
use crate::network::net_manage::{TcpPacket, UdpPacket};
use crate::network::net_message::{build_message, NetworkMessage, NetworkMessageType, SequenceNumber, TcpMessage, UdpMessage};
use crate::network::net_message::NetworkMessageType::Sequence;
use crate::network::net_tasks::{handle_input, handle_join};

pub fn udp_net_receive(
    mut comm: ResMut<Communication>,
    mut query: Query<(&Id, &mut Position)>,
    mut commands: Commands
) {
    while !comm.udp_rx.is_empty() {
        match comm.udp_rx.try_recv() {
            Ok((bytes, addr)) => {
                let decoded: ((SequenceNumber, Vec<NetworkMessageType>), usize) = bincode::serde::decode_from_slice(&bytes, config::standard()).unwrap();
                handle_input(decoded.0.1, &mut query, &mut commands);
                
                commands.spawn((UdpMessage, NetworkMessage(Sequence{ sequence_number: decoded.0.0})));
                commands.spawn(UdpPacket {bytes, addr});
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break
        }
    }
}

pub fn udp_net_send(
    comm: ResMut<Communication>,
    mut packets: Query<(Entity, &UdpPacket)>,
    messages: Query<(Entity, &NetworkMessage), With<UdpMessage>>,
    mut commands: Commands
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
            Err(TrySendError::Closed(_)) => break
        }
    }
}

pub fn tcp_net_receive(
    mut commands: Commands,
    mut comm: ResMut<Communication>,
) {
    // TODO: There should be a global lobby hashmap that will hold a bunch of these hashsets, and then figure out a way to work with that
    // Hashset of all player uuid's in lobby
    // let mut players: HashSet<Uuid> = HashSet::new();
    
    while !comm.tcp_rx.is_empty() {
        match comm.tcp_rx.try_recv() {
            Ok((bytes, stream)) => {
                // Decodes lobby_id and returns back a player_id
                handle_join(bytes, &mut commands, &stream);
                
                commands.spawn(TcpPacket{ bytes: vec![], tcp_stream: stream });
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break
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
        match comm.tcp_tx.try_send((message.clone(), packet.tcp_stream.clone())) {
            Ok(()) => {
                commands.entity(e).remove::<TcpPacket>();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break
        }
    }
}