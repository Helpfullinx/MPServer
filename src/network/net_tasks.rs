use std::cmp::min;
use std::collections::HashMap;
use bevy_ecs::change_detection::DetectChanges;
use crate::components::common::{Id, Position};
use crate::network::net_manage::Connection;
use crate::network::net_message::NetworkMessageType::Sequence;
use crate::network::server::player_input::handle_input;
use bevy_ecs::prelude::{Query, Ref};
use bincode::config;
use crate::components::player::PlayerBundle;
use crate::network::net_message::{NetworkMessage, NetworkMessageType};

const MESSAGE_PER_TICK_MAX: usize = 20;

pub fn handle_udp_message(
    mut connections: Query<&mut Connection>,
    mut players: Query<(&Id, &mut Position)>,
) {
    for mut c in connections.iter_mut() {
        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded: (Vec<NetworkMessageType>, usize) =
                        bincode::serde::decode_from_slice(&p.bytes, config::standard()).unwrap();

                    let mut seq_num = None;

                    for m in decoded.0.iter() {
                        match m {
                            Sequence { sequence_number } => {
                                seq_num = Some(sequence_number);
                            }
                            _ => {}
                        }
                    }

                    if seq_num.is_none() { return; };

                    for m in decoded.0.iter() {
                        match m {
                            NetworkMessageType::Input { keymask, player_id } => {
                                handle_input(*keymask, *player_id, &mut players);
                            }
                            NetworkMessageType::Join { .. } => {}
                            _ => {}
                        }
                    }

                    c.output_message.push(NetworkMessage(Sequence { sequence_number: *seq_num.unwrap() }));
                }
                None => {}
            }
        }
        
        if !c.input_packet_buffer.is_empty() { c.input_packet_buffer.clear() }
    }
}

pub fn build_connection_messages(
    mut connections: Query<&mut Connection>,
    players: Query<(&Id, Ref<Position>)>,
) {
    let all_positions: HashMap<u128, PlayerBundle> = players
        .iter()
        .filter(|(_, p)| { p.is_changed() })
        .map(|(i, p)| (i.0, PlayerBundle{position: *p}))
        .collect();
    
    for mut c in connections.iter_mut() {
        c.output_message.push(NetworkMessage(NetworkMessageType::Players { players: all_positions.clone() }));
    }
}
