use specs::prelude::*;
use crate::{MyTurn, Chasing, Position, Map, ApplyMove, TileSize};
use std::collections::HashMap;

pub struct ChaseAI {}

impl<'a> System<'a> for ChaseAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Chasing>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        ReadStorage<'a, TileSize>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut chasing, positions, mut map,
            entities, mut apply_move, sizes) = data;

        let mut targets : HashMap<Entity, (i32, i32)> = HashMap::new();
        let mut end_chase : Vec<Entity> = Vec::new();
        for (entity, _turn, chasing) in (&entities, &turns, &chasing).join() {
            let target_pos = positions.get(chasing.target);
            if let Some(target_pos) = target_pos {
                targets.insert(entity, (target_pos.x, target_pos.y));
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        end_chase.clear();

        let mut turn_done : Vec<Entity> = Vec::new();
        for (entity, pos, _chase, _myturn) in
            (&entities, &positions, &chasing, &turns).join()
        {
            turn_done.push(entity);
            let target_pos = targets[&entity];
            let path;

            if let Some(size) = sizes.get(entity) {
                let mut map_copy = map.clone();
                map_copy.populate_blocked_multi(size.x, size.y);
                path = rltk::a_star_search(
                    map_copy.xy_idx(pos.x, pos.y),
                    map_copy.xy_idx(target_pos.0, target_pos.1),
                    &mut map_copy
                );
            } else {
                path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(target_pos.0, target_pos.1),
                    &mut *map
                );
            }
            if path.success && path.steps.len()>1 && path.steps.len()<15 {
                apply_move.insert(entity, ApplyMove{ dest_idx: path.steps[1] }).expect("Unable to insert");
                turn_done.push(entity);
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
