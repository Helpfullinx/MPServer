use crate::components::common::{Id, Position};
use crate::components::entity::Entity;
use crate::components::player::PlayerBundle;
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::components::chat::ChatMessage;

pub trait NetworkMessageType {}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct NetworkMessage<T: NetworkMessageType>(pub T);

pub type SequenceNumber = u32;
pub type BitMask = u8;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UDP {
    Sequence {
        sequence_number: SequenceNumber,
    },
    Players {
        players: HashMap<Id, PlayerBundle>,
    },
    Input {
        keymask: BitMask,
        player_id: Id,
    },
}

impl NetworkMessageType for UDP {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TCP {
    ChatMessage {
        player_id: Id,
        message: ChatMessage,
    },
    Chat {
        messages: Vec<(Id, ChatMessage)>
    },
    Join {
        lobby_id: Id,
    },
    PlayerId {
        player_uid: Id,
    },
}

impl NetworkMessageType for TCP {}
