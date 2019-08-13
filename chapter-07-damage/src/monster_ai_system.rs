extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Monster, Name, Map, Position};
extern crate rltk;
use rltk::{Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed,_monster,name,mut pos) in (&mut viewshed, &monster, &name, &mut position).join() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                // Attack goes here
                println!("{} shouts insults", name.name);
                return;
            }

            if viewshed.visible_tiles.contains(&*player_pos) {
                // Path to the player
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32, 
                    map.xy_idx(player_pos.x, player_pos.y) as i32, 
                    &mut *map
                );
                if path.success && path.steps.len()>1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = path.steps[1] % map.width;
                    pos.y = path.steps[1] / map.width;
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}