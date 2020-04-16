use specs::prelude::*;
use crate::{MyTurn, WantsToFlee, Position, Map, Viewshed, EntityMoved};

pub struct FleeAI {}

impl<'a> System<'a> for FleeAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToFlee>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut want_flee, mut positions, mut map,
            mut viewsheds, mut entity_moved, entities) = data;

        let mut turn_done : Vec<Entity> = Vec::new();
        for (entity, mut pos, flee, mut viewshed, _myturn) in
            (&entities, &mut positions, &want_flee, &mut viewsheds, &turns).join()
        {
            turn_done.push(entity);
            let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let flee_map = rltk::DijkstraMap::new(map.width as usize, map.height as usize, &flee.indices, &*map, 100.0);
                let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
                if let Some(flee_target) = flee_target {
                    if !crate::spatial::is_blocked(flee_target as usize) {
                        crate::spatial::move_entity(entity, my_idx, flee_target);
                        viewshed.dirty = true;
                        pos.x = flee_target as i32 % map.width;
                        pos.y = flee_target as i32 / map.width;
                        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                    }
                }
        }

        want_flee.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
