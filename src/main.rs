mod components;
mod network;
mod util;

use crate::components::camera::CameraInfo;
use crate::components::chat::{Chat, send_chat_to_all_connections};
use crate::components::player::PlayerMarker;
use crate::network::NetworkPlugin;
use crate::network::net_manage::Communication;
use avian3d::PhysicsPlugins;
use avian3d::prelude::{
    Collider, ColliderBackendPlugin, ColliderHierarchyPlugin, ColliderTransformPlugin, Friction,
    LinearVelocity, MassPropertyPlugin, NarrowPhasePlugin, Physics, PhysicsSchedulePlugin,
    PhysicsSet, PhysicsTime, PreparePlugin, RigidBody, Sleeping,
};
use bevy::prelude::*;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use bincode::{Decode, Encode};
use std::collections::VecDeque;
use tokio::net::TcpStream;
use tokio::{io, sync::mpsc};

// #[tokio::main]
fn main() -> io::Result<()> {
    App::new()
        .add_plugins((
            MinimalPlugins,
            TransformPlugin::default(),
            AssetPlugin::default(),
            ScenePlugin,
            PhysicsPlugins::default().with_length_unit(10.0),
            NetworkPlugin,
        ))
        .init_resource::<Assets<Mesh>>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Time::<Physics>::default().with_relative_speed(1.0))
        .add_systems(Startup, setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn(Chat {
        chat_history: VecDeque::new(),
    });

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(40.0, 0.5, 40.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn debug_player_sleeping(
    sleeping_players: Query<(&LinearVelocity, &PlayerMarker), With<Sleeping>>,
    nonsleeping_players: Query<(&LinearVelocity, &PlayerMarker), Without<Sleeping>>,
) {
    for p in sleeping_players.iter() {
        println!("Sleeping: {:?}", p.0);
    }

    for p in nonsleeping_players.iter() {
        println!("NonSleeping: {:?}", p.0);
    }
}
