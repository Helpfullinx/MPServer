use crate::components::common::{Id, Position};
use crate::network::net_manage::UdpPacket;
use crate::network::net_message::NetworkMessageType::Sequence;
use crate::network::net_message::{
    NetworkMessage, NetworkMessageType, SequenceNumber, UdpMessage,
};
use crate::network::server::player_input::handle_input;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, Query};
use bincode::config;
use std::sync::Arc;
use tokio::net::TcpStream;

#[derive(Component)]
pub struct Connection {
    pub(crate) lobby_id: u128,
    pub(crate) stream: Arc<TcpStream>,
}

pub fn handle_udp_message(
    mut commands: Commands,
    udp_packets: Query<&UdpPacket>,
    mut players: Query<(&Id, &mut Position)>,
) {
    println!("{:?}", udp_packets.iter().len());
    for p in udp_packets.iter() {
        let decoded: ((SequenceNumber, Vec<NetworkMessageType>), usize) =
            bincode::serde::decode_from_slice(&p.bytes, config::standard()).unwrap();

        for m in decoded.0.1 {
            match m {
                NetworkMessageType::Input { keymask, player_id } => {
                    handle_input(keymask, player_id, &mut players, &mut commands);
                }
                NetworkMessageType::Join { .. } => {}
                _ => {}
            }
        }

        commands.spawn((
            UdpMessage,
            NetworkMessage(Sequence { sequence_number: decoded.0.0, }),
        ));
    }
}
