use crate::components::camera::{CameraInfo, apply_player_camera_input};
use crate::components::chat::{Chat, add_chat_message};
use crate::components::common::{Id, Vec3};
use crate::components::player::{Player, PlayerMarker, apply_player_movement_input};
use crate::network::net_manage::{TcpConnection, UdpConnection};
use crate::network::net_message::{NetworkMessage, SequenceNumber, TCP, UDP};
use crate::network::server::server_join::handle_join;
use avian3d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::{Changed, Commands, Fixed, Query, Res, Single, Time, Transform, With};
use bincode::config;
use std::cmp::min;
use std::collections::HashMap;

const MESSAGE_PER_TICK_MAX: usize = 20;

pub fn handle_udp_message(
    mut connections: Query<&mut UdpConnection>,
    mut players: Query<
        (&Id, &mut LinearVelocity, &mut Rotation, &mut CameraInfo),
        With<PlayerMarker>,
    >,
) {
    for mut c in connections.iter_mut() {
        if c.input_packet_buffer.is_empty() {
            continue;
        }

        let mut seq_num = -1;

        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded_message: (Vec<UDP>, usize) =
                        match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("Couldn't decode UDP message: {:?}", e);
                                continue;
                            }
                        };

                    for m in decoded_message.0.iter() {
                        match m {
                            UDP::Sequence { sequence_number } => {
                                if *sequence_number as i32 > seq_num {
                                    seq_num = sequence_number.clone() as i32;
                                }
                            }
                            _ => {}
                        }
                    }

                    for m in decoded_message.0.iter() {
                        match m {
                            UDP::Input {
                                keymask,
                                mouse_delta,
                                player_id,
                            } => {
                                for mut p in players.iter_mut() {
                                    if player_id == p.0 {
                                        if *keymask != 0 {
                                            apply_player_movement_input(
                                                *keymask, &mut p.1, &mut p.2, &p.3.yaw,
                                            );
                                        }

                                        apply_player_camera_input(*mouse_delta, &mut p.3);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None => {}
            }
        }

        if seq_num != -1 {
            c.add_message(NetworkMessage(UDP::Sequence {
                sequence_number: seq_num as SequenceNumber,
            }));
        };

        if !c.input_packet_buffer.is_empty() {
            c.input_packet_buffer.clear()
        }
    }
}

pub fn handle_tcp_message(
    mut chat: Query<&mut Chat>,
    mut connections: Query<&mut TcpConnection>,
    mut commands: Commands,
) {
    for mut c in connections.iter_mut() {
        if c.input_packet_buffer.is_empty() {
            continue;
        }

        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let mut decoded_message: (Vec<TCP>, usize) =
                        match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
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
    players: Query<
        (&Id, &Transform, &LinearVelocity, &CameraInfo),
        With<PlayerMarker>, /*, Changed<Transform>*/
    >,
) {
    let changed_players: HashMap<Id, Player> = players
        .iter()
        .map(|(i, t, l, c)| {
            let player = Player::new(
                Vec3::new(t.translation.x, t.translation.y, t.translation.z),
                Vec3::new(l.x, l.y, l.z),
                c.yaw,
                c.pitch,
            );

            (*i, player)
        })
        .collect();

    for mut c in connections.iter_mut() {
        if c.contains_message_type(UDP::Sequence { sequence_number: 0 }) {
            c.add_message(NetworkMessage(UDP::Players {
                players: changed_players.clone(),
            }));
        }
    }
}
