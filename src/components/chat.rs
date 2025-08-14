use crate::components::common::Id;
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, STcpType};
use bevy::prelude::{Changed, Component, Query};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const CHAT_HISTORY_LEN: usize = 10;
const MAX_CHAT_MESSAGE_LENGTH: usize = 50;

#[derive(Component)]
pub struct Chat {
    pub chat_history: VecDeque<(Id, ChatMessage)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub message: String,
}

pub fn add_chat_message(message: (Id, ChatMessage), chat: &mut Query<&mut Chat>) {
    if let Some(mut chat) = chat.single_mut().ok() {
        while chat.chat_history.len() >= CHAT_HISTORY_LEN {
            chat.chat_history.pop_front();
        }
        if !(message.1.message.len() > MAX_CHAT_MESSAGE_LENGTH) {
            chat.chat_history.push_back(message);
        }
    }
}

pub fn send_chat_to_all_connections(
    chat: Query<&mut Chat, Changed<Chat>>,
    mut connections: Query<&mut TcpConnection>,
) {
    if let Some(chat) = chat.single().ok() {
        for mut c in connections.iter_mut() {
            c.add_message(NetworkMessage(STcpType::Chat {
                messages: Vec::from(chat.chat_history.clone()),
            }));
        }
    }
}
