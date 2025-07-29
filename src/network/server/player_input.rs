use avian3d::prelude::LinearVelocity;
use bevy::math::Vec3;
use bevy::prelude::Query;
use crate::components::common::Id;
use crate::network::net_message::BitMask;

const MOVE_SPEED: f32 = 5.0;

pub fn handle_input(
    keymask: BitMask,
    playerid: Id,
    players: &mut Query<(&Id, &mut LinearVelocity)>
) {
    for (id, mut lv) in players.iter_mut() {
        if *id == playerid {

            // println!("linear velo {:?}", lv);

            if keymask == 0 {
                continue;
            }
            
            let mut vector = Vec3::ZERO;
            
            if keymask & 1 > 0 {
                vector.x -= 1.0;
            }
            if keymask & 2 > 0 {
                vector.x += 1.0;
            }
            if keymask & 4 > 0 {
                vector.z -= 1.0;
            }
            if keymask & 8 > 0 {
                vector.z += 1.0;
            }
            
            let normalized_velocity = vector.normalize_or_zero();
            
            lv.x = normalized_velocity.x * MOVE_SPEED;
            lv.z = normalized_velocity.z * MOVE_SPEED;

        }
    }
}
