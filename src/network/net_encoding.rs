use crate::components::common::Position;
use crate::components::entity::Entity;
use crate::components::player::PlayerBundle;
use crate::network::net_message::UDP;
use bincode::config;
use std::collections::HashMap;

#[test]
fn test_encode_decode() {
    let buf = encode();
    let res = decode(buf);

    println!("{:#?}", res);
}

fn encode() -> Vec<u8> {
    let mut players = HashMap::new();
    for i in 1..=3 {
        players.insert(i, PlayerBundle::default());
    }

    let entities = vec![
        (Entity::new(0), Position::new(0.0, 0.0)),
        (Entity::new(1), Position::new(0.0, 0.0)),
    ];

    let msg: Vec<UDP> = vec![
        UDP::Players { players },
        UDP::Entities { entities },
    ];

    let buf = bincode::serde::encode_to_vec(msg, config::standard()).unwrap();

    println!("{:?}", buf);

    buf
}

fn decode(buf: Vec<u8>) -> Vec<UDP> {
    bincode::serde::decode_from_slice(&buf, config::standard())
        .unwrap()
        .0
}
