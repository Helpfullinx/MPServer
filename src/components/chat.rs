use std::collections::VecDeque;
use bevy_ecs::prelude::{Changed, Component, Query};
use serde::{Deserialize, Serialize};
use crate::network::net_manage::TcpConnection;
use crate::network::net_message::{NetworkMessage, TCP};

#[derive(Component)]
pub struct Chat {
    pub chat_history: VecDeque<(u128, ChatMessage)>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub message: String,
}

const CHAT_HISTORY_LEN: usize = 10;

pub fn add_chat_message(
    message: (u128, ChatMessage),
    chat: &mut Query<&mut Chat>
) {
    if let Some(mut chat) = chat.single_mut().ok() {
        while chat.chat_history.len() >= CHAT_HISTORY_LEN {
            chat.chat_history.pop_front();
        }
        
        chat.chat_history.push_back(message);
    }
}

pub fn send_chat_to_all_connections(
    chat: Query<&mut Chat, Changed<Chat>>,
    mut connections: Query<&mut TcpConnection>
) {
    if let Some(chat) = chat.single().ok() {
        for mut c in connections.iter_mut() {
            c.output_message.push(NetworkMessage(TCP::Chat {
                messages: Vec::from(chat.chat_history.clone()),
            }));
        }
    }
}