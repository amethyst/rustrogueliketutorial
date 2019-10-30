use specs::prelude::*;
use super::{Viewshed, Bystander, Map, Position, RunState, EntityMoved};

pub struct BystanderAI {}

impl<'a> System<'a> for BystanderAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Bystander>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, EntityMoved>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, runstate, entities, mut viewshed, bystander, mut position,
            mut entity_moved, mut rng) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed,_bystander,mut pos) in (&entities, &mut viewshed, &bystander, &mut position).join() {
            // Try to move randomly
            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);
            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            let dest_idx = map.xy_idx(x, y);
            if !map.blocked[dest_idx] {
                let idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                pos.x = x;
                pos.y = y;
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                map.blocked[dest_idx] = true;
                viewshed.dirty = true;
            }
        }
    }
}