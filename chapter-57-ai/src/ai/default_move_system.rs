use specs::prelude::*;
use crate::{MyTurn, MoveMode, Movement, Position, Map, Viewshed, EntityMoved, map::tile_walkable};

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        Entities<'a>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut move_mode, mut positions, mut map,
            mut viewsheds, mut entity_moved, mut rng, entities) = data;

        let mut turn_done : Vec<Entity> = Vec::new();
        for (entity, mut pos, mut mode, mut viewshed, _myturn) in
            (&entities, &mut positions, &mut move_mode, &mut viewsheds, &turns).join()
        {
            turn_done.push(entity);

            match &mut mode.mode {
                Movement::Static => {},

                Movement::Random => {
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

                    if x > 0 && x < map.width-1 && y > 0 && y < map.height-1 {
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
                },

                Movement::RandomWaypoint{path} => {
                    if let Some(path) = path {
                        // We have a target - go there
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        if path.len()>1 {
                            if !map.blocked[path[1] as usize] {
                                map.blocked[idx] = false;
                                pos.x = path[1] as i32 % map.width;
                                pos.y = path[1] as i32 / map.width;
                                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                                idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = true;
                                viewshed.dirty = true;
                                path.remove(0); // Remove the first step in the path
                            }
                            // Otherwise we wait a turn to see if the path clears up
                        } else {
                            mode.mode = Movement::RandomWaypoint{ path : None };
                        }
                    } else {
                        let target_x = rng.roll_dice(1, map.width-2);
                        let target_y = rng.roll_dice(1, map.height-2);
                        let idx = map.xy_idx(target_x, target_y);
                        if tile_walkable(map.tiles[idx]) {
                            let path = rltk::a_star_search(
                                map.xy_idx(pos.x, pos.y),
                                map.xy_idx(target_x, target_y),
                                &mut *map
                            );
                            if path.success && path.steps.len()>1 {
                                mode.mode = Movement::RandomWaypoint{
                                    path: Some(path.steps)
                                };
                            }
                        }
                    }
                }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
