mod components;
mod math;
mod network;
mod util;

use crate::network::net_manage::{Communication, start_tcp_task, start_udp_task};
use crate::network::net_system::{tcp_net_receive, tcp_net_send, udp_net_receive, udp_net_send};
use bevy_ecs::prelude::*;
use bincode::{Decode, Encode};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::{io, sync::mpsc};
use crate::network::net_tasks::{build_connection_messages, handle_udp_message};

#[derive(Resource)]
struct FixedTime {
    timestep: Duration,
    accumulator: Duration,
    last_update: Instant,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (udp_send_tx, udp_send_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (udp_receive_tx, udp_receive_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    let (tcp_send_tx, tcp_send_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);
    let (tcp_receive_tx, tcp_receive_rx) = mpsc::channel::<(Vec<u8>, Arc<TcpStream>)>(1_000);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4444);

    start_tcp_task(addr, tcp_send_rx, tcp_receive_tx).await?;
    start_udp_task(addr, udp_send_rx, udp_receive_tx, 8).await?;

    let mut world = World::new();

    world.insert_resource(Communication::new(
        udp_send_tx,
        udp_receive_rx,
        tcp_send_tx,
        tcp_receive_rx,
    ));
    world.insert_resource(FixedTime {
        timestep: Duration::from_secs_f64(1.0 / 60.0),
        accumulator: Duration::ZERO,
        last_update: Instant::now(),
    });

    let mut schedule = Schedule::default();
    schedule.add_systems((
        udp_net_receive,
        tcp_net_receive,
        handle_udp_message.after(udp_net_receive),
        build_connection_messages.after(handle_udp_message),
        udp_net_send.after(build_connection_messages),
        tcp_net_send.after(tcp_net_receive),
    ));

    loop {
        fixed_timestep_runner(&mut world, &mut schedule);
    }

    Ok(())
}

fn fixed_timestep_runner(world: &mut World, schedule: &mut Schedule) {
    let now = Instant::now();

    // Step 1: extract everything you need from FixedTime into locals
    {
        let mut fixed_time = world.get_resource_mut::<FixedTime>().unwrap();
        let delta = now - fixed_time.last_update;
        fixed_time.last_update = now;
        fixed_time.accumulator += delta;
    }

    // Step 2: now we reborrow to read and mutate again safely
    let timestep;
    {
        let fixed_time = world.get_resource::<FixedTime>().unwrap();
        timestep = fixed_time.timestep;
    }

    while {
        // Borrow again inside loop body only
        let mut fixed_time = world.get_resource_mut::<FixedTime>().unwrap();
        fixed_time.accumulator >= timestep
    } {
        schedule.run(world);

        // Subtract after the run
        let mut fixed_time = world.get_resource_mut::<FixedTime>().unwrap();
        fixed_time.accumulator -= timestep;
    }
}
