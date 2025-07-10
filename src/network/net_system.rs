use crate::Communication;
use crate::network::net_manage::{UdpConnection, Packet, TcpConnection};
use crate::network::server::server_join::handle_join;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::prelude::{Commands, Query};
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
                
                // println!("{:?}", c);
                
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
        if c.output_message.is_empty() {
            continue;
        }

        let message = bincode::serde::encode_to_vec(&c.output_message, config::standard()).unwrap();

        match comm.udp_tx.try_send((message.clone(), c.socket)) {
            Ok(()) => {
                c.output_message.clear();
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
        if c.output_message.is_empty() {
            continue;
        }

        let message = bincode::serde::encode_to_vec(&c.output_message, config::standard()).unwrap();

        match comm.tcp_tx.try_send((message.clone(), c.stream.clone())) {
            Ok(()) => {
                println!("OK");
                c.output_message.clear();
            }
            Err(TrySendError::Full(_)) => break,
            Err(TrySendError::Closed(_)) => break,
        }
    }
}

fn same_stream(a: &TcpStream, b: &TcpStream) -> bool {
    a.peer_addr().ok() == b.peer_addr().ok() && a.local_addr().ok() == b.local_addr().ok()
}
