use specs::prelude::*;
use crate::{MyTurn, MoveMode, Movement, Position, Map, map::tile_walkable, ApplyMove};

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, ApplyMove>,
        Entities<'a>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut move_mode, positions, mut map,
            mut apply_move, entities) = data;

        let mut turn_done : Vec<Entity> = Vec::new();
        for (entity, pos, mut mode, _myturn) in
            (&entities, &positions, &mut move_mode, &turns).join()
        {
            turn_done.push(entity);

            match &mut mode.mode {
                Movement::Static => {},

                Movement::Random => {
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = crate::rng::roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x > 0 && x < map.width-1 && y > 0 && y < map.height-1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !crate::spatial::is_blocked(dest_idx) {
                            apply_move.insert(entity, ApplyMove{ dest_idx })
                                .expect("Unable to insert");
                            turn_done.push(entity);
                        }
                    }
                },

                Movement::RandomWaypoint{path} => {
                    if let Some(path) = path {
                        // We have a target - go there
                        if path.len()>1 {
                            if !crate::spatial::is_blocked(path[1] as usize) {
                                apply_move.insert(entity, ApplyMove{ dest_idx : path[1] })
                                    .expect("Unable to insert");
                                path.remove(0); // Remove the first step in the path
                                turn_done.push(entity);
                            }
                            // Otherwise we wait a turn to see if the path clears up
                        } else {
                            mode.mode = Movement::RandomWaypoint{ path : None };
                        }
                    } else {
                        let target_x = crate::rng::roll_dice(1, map.width-2);
                        let target_y = crate::rng::roll_dice(1, map.height-2);
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
