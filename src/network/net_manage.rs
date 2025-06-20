use std::collections::HashSet;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Resource;
use bincode::config;
use tokio::io;
use tokio::io::Interest;
use tokio::net::{TcpSocket, TcpStream, UdpSocket};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::network::net_message::NetworkMessageType;

#[derive(Resource)]
pub struct Communication {
    pub udp_tx: Sender<(Vec<u8>, SocketAddr)>,
    pub udp_rx: Receiver<(Vec<u8>, SocketAddr)>,
    pub tcp_tx: Sender<(Vec<u8>, Arc<TcpStream>)>,
    pub tcp_rx: Receiver<(Vec<u8>, Arc<TcpStream>)>,
}

#[derive(Resource)]
pub struct Connections {
    pub ip_addrs: Vec<SocketAddr>,
}

#[derive(Component)]
pub struct UdpPacket {
    pub bytes: Vec<u8>,
    pub addr: SocketAddr,
}

#[derive(Component)]
pub struct TcpPacket {
    pub bytes: Vec<u8>,
    pub tcp_stream: Arc<TcpStream>,
}

impl Communication {
    pub fn new(
        udp_tx: Sender<(Vec<u8>, SocketAddr)>,
        udp_rx: Receiver<(Vec<u8>, SocketAddr)>,
        tcp_tx: Sender<(Vec<u8>, Arc<TcpStream>)>,
        tcp_rx: Receiver<(Vec<u8>, Arc<TcpStream>)>,
    ) -> Self {
        Self {
            udp_tx,
            udp_rx,
            tcp_tx,
            tcp_rx,
        }
    }
}

impl Connections {
    pub fn new() -> Self {
        Self{
            ip_addrs: Vec::new()
        }
    }
}

pub async fn start_tcp_task(
    bind_addr: SocketAddr,
    mut outbound: Receiver<(Vec<u8>, Arc<TcpStream>)>,
    inbound: Sender<(Vec<u8>, Arc<TcpStream>)>,
) -> Result<(), Error> {
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseport(true)?;
    socket.bind(bind_addr)?;

    let listener = socket.listen(1024)?;

    tokio::spawn(async move {
        tokio::spawn(async move {
            loop {
                // Accept first connection in queue
                match listener.accept().await {

                    // If valid connection, read data
                    Ok((stream, addr)) => {

                        println!("New connection from {}", addr);

                        // TODO: Apparently this can create false positives and what it reads because of that may be empty, therefore we have to check that
                        // Get the ready-ness value for the stream
                        let ready = stream.ready(Interest::READABLE).await.unwrap();

                        // If stream is in a readable state, we read in the lobby id
                        if ready.is_readable() {

                            // Buffer for holding data received through stream
                            let mut buf = vec![0u8; 200];

                            // Read in the lobby id
                            match stream.try_read(&mut buf) {
                                Ok(_) => {},
                                Err(e) => {println!("{:?}", e)},
                            }

                            inbound.send((buf, Arc::new(stream))).await.unwrap();
                        }
                    }
                    Err(e) => {eprintln!("{}", e);}
                }
            }
        });

        tokio::spawn(async move {
            while let Some((bytes, stream)) = outbound.recv().await {
                let ready = stream.ready(Interest::WRITABLE).await.unwrap();

                // If stream is writable, send the players uuid
                if ready.is_writable(){
                    match stream.try_write(&*bytes) {
                        Ok(_) => {}
                        Err(e) => { println!("{:?}", e) },
                    };
                }
            }
        });
    });

    Ok(())
}

pub async fn start_udp_task(
    bind_addr: SocketAddr,
    mut outbound: Receiver<(Vec<u8>, SocketAddr)>,
    inbound: Sender<(Vec<u8>, SocketAddr)>,
    pool_size: usize
) -> io::Result<()> {
    // Create and share the socket
    let socket = Arc::new(UdpSocket::bind(bind_addr).await?);// separate handles are handy

    // Receive Loop - Creates number of tasks based on pool size specified
    for i in 0..pool_size {
        let recv_sock = socket.clone();
        let inbound_tx = inbound.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                match recv_sock.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let _ = inbound_tx.send((buf[..len].to_vec(), addr)).await;
                    }
                    Err(e) => {
                        eprintln!("recv error: {e}");
                        break;
                    }
                }
            }
        });
    }

    // Send Loop
    let send_sock = socket.clone();
    tokio::spawn(async move {
        while let Some((bytes, addr)) = outbound.recv().await {
            if let Err(e) = send_sock.send_to(&bytes, &addr).await {
                eprintln!("send error: {e}");
            }
        }
    });

    Ok(())
}