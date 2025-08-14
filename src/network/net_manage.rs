use crate::network::net_message::{NetworkMessage, CTcpType, CUdpType, SUdpType, STcpType};
use bevy::prelude::{Component, Resource};
use std::any::Any;
use std::collections::VecDeque;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io;
use tokio::io::Interest;
use tokio::net::{TcpSocket, TcpStream, UdpSocket};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Resource)]
pub struct Communication {
    pub udp_tx: Sender<(Vec<u8>, SocketAddr)>,
    pub udp_rx: Receiver<(Vec<u8>, SocketAddr)>,
    pub tcp_tx: Sender<(Vec<u8>, Arc<TcpStream>)>,
    pub tcp_rx: Receiver<(Vec<u8>, Arc<TcpStream>)>,
}

#[derive(Component, Debug)]
pub struct UdpConnection {
    pub socket: SocketAddr,
    pub input_packet_buffer: VecDeque<Packet>,
    output_message: Vec<NetworkMessage<SUdpType>>,
    pub ping: u32
}

#[derive(Component, Debug)]
pub struct TcpConnection {
    pub stream: Arc<TcpStream>,
    pub input_packet_buffer: VecDeque<Packet>,
    output_message: Vec<NetworkMessage<STcpType>>,
    pub ping: u32,
    pub lobby_id: u128,
}

#[derive(Component, Debug)]
pub struct Packet {
    pub bytes: Vec<u8>,
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

impl UdpConnection {
    pub fn new(ip_addrs: SocketAddr) -> Self {
        Self {
            socket: ip_addrs,
            input_packet_buffer: VecDeque::new(),
            output_message: Vec::new(),
            ping: 0
        }
    }

    pub fn add_message(&mut self, message: NetworkMessage<SUdpType>) {
        self.output_message.push(message);
    }

    pub fn get_current_messages(&self) -> &Vec<NetworkMessage<SUdpType>> {
        &self.output_message
    }

    pub fn is_empty_messages(&self) -> bool {
        self.output_message.is_empty()
    }

    pub fn clear_messages(&mut self) {
        self.output_message.clear();
    }

    pub fn contains_message_type(&self, message_type: SUdpType) -> bool {
        for m in self.output_message.iter() {
            if m.0.type_id() == message_type.type_id() {
                return true;
            }
        }
        false
    }
}

impl TcpConnection {
    pub fn new(stream: Arc<TcpStream>) -> Self {
        Self {
            stream,
            lobby_id: 0,
            input_packet_buffer: Default::default(),
            output_message: vec![],
            ping: 0,
        }
    }

    pub fn add_message(&mut self, message: NetworkMessage<STcpType>) {
        self.output_message.push(message);
    }

    pub fn get_current_messages(&self) -> &Vec<NetworkMessage<STcpType>> {
        &self.output_message
    }

    pub fn is_empty_messages(&self) -> bool {
        self.output_message.is_empty()
    }

    pub fn clear_messages(&mut self) {
        self.output_message.clear();
    }
}

pub async fn start_udp_task(
    bind_addr: SocketAddr,
    mut outbound: Receiver<(Vec<u8>, SocketAddr)>,
    inbound: Sender<(Vec<u8>, SocketAddr)>,
    pool_size: usize,
) -> io::Result<()> {
    // Create and share the socket
    let socket = Arc::new(UdpSocket::bind(bind_addr).await?); // separate handles are handy

    // Receive Loop - Creates number of tasks based on pool size specified
    for _ in 0..pool_size {
        let recv_sock = socket.clone();
        let inbound_tx = inbound.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            loop {
                match recv_sock.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let _ = inbound_tx.send((buf[..len].to_vec(), addr)).await;
                    }
                    Err(e) => {
                        eprintln!("Couldn't read: {e}");
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
                eprintln!("Couldn't write: {e}");
            }
        }
    });

    Ok(())
}

pub async fn start_tcp_task(
    bind_addr: SocketAddr,
    mut outbound: Receiver<(Vec<u8>, Arc<TcpStream>)>,
    inbound: Sender<(Vec<u8>, Arc<TcpStream>)>,
) -> Result<(), Error> {
    let socket = TcpSocket::new_v4()?;
    //TODO: Figure out the equivalent on windows. I've read that one way is to create a raw
    // socket and set the windows equivalent of this and then cast it as a tokio socket
    // https://stackoverflow.com/questions/40468685/how-to-set-the-socket-option-so-reuseport-in-rust

    // On windows, this does not work as it is unix specific
    #[cfg(unix)]
    socket.set_reuseport(true)?;

    // I believe this does the equivalent of reuseport for windows targets
    socket.bind(bind_addr)?;

    let listener = socket.listen(1024)?;

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from {}", addr);

                    let inbound_arc = inbound.clone();
                    let stream_arc_outer = Arc::new(stream);

                    tokio::spawn(async move {
                        let stream_arc_inner = stream_arc_outer.clone();
                        loop {
                            // TODO: Apparently this can create false positives and what it reads because of that may be empty, therefore we have to check that
                            let ready = stream_arc_inner.ready(Interest::READABLE).await.unwrap();

                            if ready.is_readable() {
                                let mut buf = vec![0u8; 1024];

                                match stream_arc_inner.try_read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(len) => {
                                        println!("buf: {:?}", &buf[..len]);
                                        let _ = inbound_arc
                                            .send((buf[..len].to_vec(), stream_arc_inner.clone()))
                                            .await;
                                    }
                                    Err(e) => {
                                        println!("Couldn't read: {:?}", e);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some((bytes, stream)) = outbound.recv().await {
            let ready = stream.ready(Interest::WRITABLE).await.unwrap();

            if ready.is_writable() {
                match stream.try_write(&*bytes) {
                    Ok(_) => {
                        println!("Message Sent")
                    }
                    Err(e) => {
                        println!("Couldn't write: {:?}", e)
                    }
                };
            }
        }
    });

    Ok(())
}
