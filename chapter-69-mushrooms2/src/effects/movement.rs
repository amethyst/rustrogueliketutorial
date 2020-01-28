use specs::prelude::*;
use super::*;
use crate::components::{ApplyTeleport};

pub fn apply_teleport(ecs: &mut World, destination: &EffectSpawner, target: Entity) {
    let player_entity = ecs.fetch::<Entity>();
    if let EffectType::TeleportTo{x, y, depth, player_only} = &destination.effect_type {
        if !player_only || target == *player_entity {
            let mut apply_teleport = ecs.write_storage::<ApplyTeleport>();
            apply_teleport.insert(target, ApplyTeleport{
                dest_x : *x,
                dest_y : *y,
                dest_depth : *depth
            }).expect("Unable to insert");
        }
    }
}
