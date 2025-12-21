use crate::components::camera::{CameraInfo, apply_player_camera_input};
use crate::components::chat::{Chat, add_chat_message};
use crate::components::common::{Id, Vec2, Vec3};
use crate::components::player::{Player, PlayerMarker, apply_player_movement_input, PlayerAnimationState, AnimationState, PendingInputs, Input};
use crate::network::net_manage::{TcpConnection, UdpConnection};
use crate::network::net_message::{NetworkMessage, SequenceNumber, CTcpType, CUdpType, SUdpType, BitMask};
use crate::network::server::server_join::handle_join;
use avian3d::prelude::{LinearVelocity, Position, Rotation};
use bevy::prelude::{warn, Commands, Query, With};
use bincode::config;
use std::cmp::min;
use std::collections::HashMap;
use std::time::SystemTime;
use bevy::log::info;
use bevy::math::Quat;
use crate::network::net_message::SUdpType::Pong;

const MESSAGE_PER_TICK_MAX: usize = 20;

struct MessageBuffer {
    sequence_number: i32,
    player_id: Option<Id>,
    keymask: BitMask,
    mouse_delta: Vec2,
    accumulated_movement: (bevy::math::Vec3, Quat),
    pong_message: Option<SUdpType>,
}

pub fn handle_udp_message(
    mut connections: Query<&mut UdpConnection>,
    mut players: Query<
        (&Id, &mut LinearVelocity, &mut Rotation, &mut CameraInfo, &mut PlayerAnimationState, &Position, &mut PendingInputs),
        With<PlayerMarker>,
    >,
) {
    for mut c in connections.iter_mut() {
        if c.input_packet_buffer.is_empty() {
            continue;
        }

        let mut current_message = MessageBuffer {
            sequence_number: -1,
            player_id: None,
            keymask: 0,
            mouse_delta: Vec2::new(0.0,0.0),
            accumulated_movement: (bevy::math::Vec3::ZERO, Quat::default()),
            pong_message: None,
        };

        for _ in 0..min(MESSAGE_PER_TICK_MAX, c.input_packet_buffer.len()) {
            match c.input_packet_buffer.pop_front() {
                Some(p) => {
                    let decoded_message: (Vec<CUdpType>, usize) =
                        match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("Couldn't decode UDP message: {:?}", e);
                                continue;
                            }
                        };

                    warn!("Received UDP message: {:?}", decoded_message);

                    for m in decoded_message.0.iter() {
                        match m {
                            CUdpType::PlayerId {id} => {
                                current_message.player_id = Some(id.clone())
                            }
                            CUdpType::Sequence { sequence_number } => {
                                info!("Received Sequence: {:?}", sequence_number);
                                current_message.sequence_number = sequence_number.clone() as i32;
                            }
                            CUdpType::Input {
                                keymask,
                                mouse_delta,
                            } => {
                                current_message.keymask = *keymask;
                                current_message.mouse_delta = *mouse_delta;
                                
                                info!("Received Keymask: {:?}", keymask);
                            },
                            CUdpType::Ping { initiation_time, last_rtt } => {
                                c.ping = *last_rtt;
                                let time_now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                                current_message.pong_message = Some(Pong{ initiation_time: *initiation_time, server_received_time: time_now as u32 });
                            }
                        }
                    }
                    
                    if let Some(id) = current_message.player_id {
                        for mut p in players.iter_mut() {
                            if id == *p.0 {
                                p.6.buffer.push_back(Input{keymask: current_message.keymask, mouse_delta: current_message.mouse_delta})
                            }
                        }
                    }
                }
                None => {}
            }
        }

        if current_message.sequence_number != -1 {
            info!("Sequence Added: {:?}", current_message.sequence_number);
            c.add_message(NetworkMessage(SUdpType::Sequence {
                sequence_number: current_message.sequence_number as SequenceNumber,
            }));
        };

        if let Some(pong) = current_message.pong_message {
            c.add_message(NetworkMessage(pong));
        }
        
        if !c.input_packet_buffer.is_empty() {
            info!("Clearing input buffer");
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
                    let mut decoded_message: (Vec<CTcpType>, usize) =
                        match bincode::serde::decode_from_slice(&p.bytes, config::standard()) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("Couldn't decode TCP message: {:?}", e);
                                continue;
                            }
                        };

                    for m in decoded_message.0.iter_mut() {
                        match m {
                            CTcpType::ChatMessage { player_id, message } => {
                                add_chat_message((*player_id, message.clone()), &mut chat);
                            }
                            CTcpType::Join { lobby_id } => {
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
        (&Id, &Position, &LinearVelocity, &CameraInfo, &PlayerAnimationState),
        With<PlayerMarker>, /*, Changed<Transform>*/
    >,
) {
    let changed_players: HashMap<Id, Player> = players
        .iter()
        .map(|(i, p, l, c, pas)| {
            let player = Player::new(
                p.0.into(),
                p.0.into(),
                c.yaw,
                c.pitch,
                pas.0
            );

            (*i, player)
        })
        .collect();

    for mut c in connections.iter_mut() {
        if c.contains_message_type(SUdpType::Sequence { sequence_number: 0 }) {
            c.add_message(NetworkMessage(SUdpType::Players {
                players: changed_players.clone(),
            }));
        }
    }
}
