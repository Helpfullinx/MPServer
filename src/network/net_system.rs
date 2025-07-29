use bevy::prelude::{Commands, Query, ResMut};
use crate::Communication;
use crate::network::net_manage::{UdpConnection, Packet, TcpConnection};
use bincode::config;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error::{TryRecvError, TrySendError};

pub fn udp_net_receive(
    mut comm: ResMut<Communication>,
    mut connections: Query<&mut UdpConnection>,
    mut commands: Commands,
) {
    while !comm.udp_rx.is_empty() {
        match comm.udp_rx.try_recv() {
            Ok((bytes, socket)) => {
                let c = connections.iter_mut().find(|x| (x.socket.ip() == socket.ip()) && (x.socket.port() == socket.port()));
                
                match c {
                    Some(mut c) => {
                        c.input_packet_buffer.push_back(Packet {
                            bytes: bytes.clone(),
                        });
                    }
                    None => {
                        let mut conn = UdpConnection::new(socket);
                        conn.input_packet_buffer.push_back(Packet {
                            bytes: bytes.clone(),
                        });
                        commands.spawn(conn);
                    }
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
}

pub fn udp_net_send(comm: ResMut<Communication>, mut connections: Query<&mut UdpConnection>) {
    for mut c in connections.iter_mut() {
        if c.is_empty_messages() {
            continue;
        }

        let encoded_message = match bincode::serde::encode_to_vec(c.get_current_messages(), config::standard()) {
            Ok(m) => m,
            Err(e) => {
                println!("Couldn't encode UDP message: {:?}", e);
                continue;
            }
        };

        match comm.udp_tx.try_send((encoded_message.clone(), c.socket)) {
            Ok(()) => {
                c.clear_messages();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break,
        }
    }
}

pub fn tcp_net_receive(
    mut commands: Commands,
    mut connections: Query<&mut TcpConnection>,
    mut comm: ResMut<Communication>,
) {
    while !comm.tcp_rx.is_empty() {
        match comm.tcp_rx.try_recv() {
            Ok((bytes, stream)) => {
                let c = connections
                    .iter_mut()
                    .find(|x| same_stream(&*x.stream, &*stream));
                
                match c {
                    Some(mut c) => {
                        c.input_packet_buffer.push_back(Packet {
                            bytes: bytes.clone(),
                        });
                    }
                    None => {
                        let mut conn = TcpConnection::new(stream);
                        conn.input_packet_buffer.push_back(Packet {
                            bytes: bytes.clone(),
                        });
                        commands.spawn(conn);
                    }
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
}

pub fn tcp_net_send(comm: ResMut<Communication>, mut connections: Query<&mut TcpConnection>) {
    for mut c in connections.iter_mut() {
        if c.is_empty_messages() {
            continue;
        }

        let encoded_message = match bincode::serde::encode_to_vec(c.get_current_messages(), config::standard()) {
            Ok(m) => m,
            Err(e) => {
                println!("Couldn't encode TCP message: {:?}", e);
                continue;
            }
        };

        match comm.tcp_tx.try_send((encoded_message.clone(), c.stream.clone())) {
            Ok(()) => {
                println!("OK");
                c.clear_messages();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break,
        }
    }
}

fn same_stream(a: &TcpStream, b: &TcpStream) -> bool {
    a.peer_addr().ok() == b.peer_addr().ok() && a.local_addr().ok() == b.local_addr().ok()
}
