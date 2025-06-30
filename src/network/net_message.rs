use crate::components::common::{Id, Position};
use crate::components::entity::Entity;
use crate::components::player::PlayerBundle;
use crate::network::net_message::NetworkMessageType::Sequence;
use bevy_ecs::prelude::{Commands, Component, Query, With};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct NetworkMessage(pub NetworkMessageType);

#[derive(Component)]
pub struct UdpMessage;

#[derive(Component)]
pub struct TcpMessage;

pub type SequenceNumber = u32;
pub type BitMask = u8;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessageType {
    Sequence {
        sequence_number: SequenceNumber,
    },
    Spawn {
        player_uid: Vec<Id>,
    },
    Players {
        players: HashMap<u128, PlayerBundle>,
    },
    Entities {
        entities: Vec<(Entity, Position)>,
    },
    Input {
        keymask: BitMask,
        player_id: u128,
    },
    Join {
        lobby_id: u128,
    },
    PlayerId {
        player_uid: u128,
    },
}

pub fn build_message(
    messages: Vec<(bevy_ecs::entity::Entity, &NetworkMessage)>,
    commands: &mut Commands,
) -> (SequenceNumber, Vec<NetworkMessage>) {
    let mut net_message = Vec::new();
    let mut seq_num = 0;
    for n in messages {
        match n.1.0 {
            Sequence { sequence_number } => {
                seq_num = sequence_number;
                commands.entity(n.0).despawn();
            }
            _ => {}
        }
        net_message.push(n.1.clone());
        commands.entity(n.0).despawn();
    }

    (seq_num, net_message)
}
