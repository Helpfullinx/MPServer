use crate::components::player::Player;
use crate::network::net_manage::{TcpConnection, UdpConnection};
use crate::network::net_message::{NetworkMessage, TCP, UDP};
use crate::network::server::player_input::handle_input;
use bincode::config;
use std::cmp::min;
use std::collections::HashMap;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::{Changed, Commands, Query, Transform};
use crate::components::chat::{add_chat_message, Chat};
use crate::components::common::{Id, Vec3};
use crate::network::server::server_join::handle_join;

const MESSAGE_PER_TICK_MAX: usize = 20;

pub fn handle_udp_message(
    mut connections: Query<&mut UdpConnection>,
    mut players: Query<(&Id, &mut Transform, &LinearVelocity)>,
) {
    for mut c in connections.iter_mut() {
        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded_message: (Vec<UDP>, usize) = match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
                        Ok(m) => m,
                        Err(e) => {
                            println!("Couldn't decode UDP message: {:?}", e);
                            continue;
                        }
                    };

                    let mut seq_num = None;

                    for m in decoded_message.0.iter() {
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
                    
                    println!("sequence number: {:?}", seq_num);
                    
                    for m in decoded_message.0.iter() {
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
                    let mut decoded_message: (Vec<TCP>, usize) = match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
                        Ok(m) => m,
                        Err(e) => {
                            println!("Couldn't decode TCP message: {:?}", e);
                            continue;
                        }
                    };
                    
                    for m in decoded_message.0.iter_mut() {
                        match m {
                            TCP::ChatMessage { player_id, message } => {
                                add_chat_message((*player_id, message.clone()), &mut chat);
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
    players: Query<(&Id, &Transform, &LinearVelocity)/*, Changed<Transform>*/>,
) {
    let changed_players: HashMap<Id, Player> = players
        .iter()
        .map(|(i, t, l)| {
            let player = Player::new(
                Vec3::new(
                    t.translation.x,
                    t.translation.y,
                    t.translation.z
                ),
                Vec3::new(
                    l.x,
                    l.y,
                    l.z
                )
            );
            
            (*i, player)
        })
        .collect();

    for mut c in connections.iter_mut() {
        c.output_message
            .push(NetworkMessage(UDP::Players {
                players: changed_players.clone(),
            }));
    }
}
