use crate::components::common::{Id, Position};
use crate::components::player::PlayerBundle;
use crate::network::net_manage::{Packet, TcpConnection, UdpConnection};
use crate::network::net_message::{NetworkMessage, TCP, UDP};
use crate::network::server::player_input::handle_input;
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::prelude::{Commands, Query, Ref};
use bincode::config;
use std::cmp::min;
use std::collections::HashMap;
use crate::components::chat::{add_chat_message, Chat};
use crate::network::server::server_join::handle_join;

const MESSAGE_PER_TICK_MAX: usize = 20;

pub fn handle_udp_message(
    mut connections: Query<&mut UdpConnection>,
    mut players: Query<(&Id, &mut Position)>,
) {
    for mut c in connections.iter_mut() {
        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded: (Vec<UDP>, usize) =
                        bincode::serde::decode_from_slice(&p.bytes, config::standard()).unwrap();

                    let mut seq_num = None;

                    for m in decoded.0.iter() {
                        match m {
                            UDP::Sequence { sequence_number } => {
                                seq_num = Some(sequence_number);
                            }
                            _ => {}
                        }
                    }

                    if seq_num.is_none() {
                        continue;
                    };

                    for m in decoded.0.iter() {
                        match m {
                            UDP::Input { keymask, player_id } => {
                                handle_input(*keymask, *player_id, &mut players);
                            }
                            _ => {}
                        }
                    }

                    c.output_message.push(NetworkMessage(UDP::Sequence {
                        sequence_number: *seq_num.unwrap(),
                    }));
                }
                None => {}
            }
        }

        if !c.input_packet_buffer.is_empty() {
            c.input_packet_buffer.clear()
        }
    }
}

pub fn handle_tcp_message(
    mut chat: Query<&mut Chat>,
    mut connections: Query<&mut TcpConnection>,
    mut commands: Commands
) {
    for mut c in connections.iter_mut() {
        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded: (Vec<TCP>, usize) =
                        bincode::serde::decode_from_slice(&p.bytes, config::standard()).unwrap();
                    
                    println!("decoded: {:?}", decoded);
                    
                    for m in decoded.0.iter() {
                        match m {
                            TCP::ChatMessage { message } => {
                                add_chat_message(message.clone(), &mut chat);
                            }
                            TCP::Join { lobby_id } => {
                                handle_join(*lobby_id, &mut c, &mut commands);
                            }
                            _ => {}
                        }
                    }
                }
                None => {}
            }
        }
    }
}

pub fn build_connection_messages(
    mut connections: Query<&mut UdpConnection>,
    players: Query<(&Id, Ref<Position>)>,
) {
    let all_positions: HashMap<u128, PlayerBundle> = players
        .iter()
        .filter(|(_, p)| p.is_changed())
        .map(|(i, p)| (i.0, PlayerBundle { position: *p }))
        .collect();

    for mut c in connections.iter_mut() {
        c.output_message
            .push(NetworkMessage(UDP::Players {
                players: all_positions.clone(),
            }));
    }
}
