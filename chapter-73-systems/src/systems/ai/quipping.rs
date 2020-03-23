use specs::prelude::*;
use crate::{Quips, Name, MyTurn, Viewshed};

pub struct QuipSystem {}

impl<'a> System<'a> for QuipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Viewshed>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut quips, names, turns, player_pos, viewsheds) = data;

        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && crate::rng::roll_dice(1,6)==1 {
                let quip_index =
                    if quip.available.len() == 1 { 0 }
                    else { (crate::rng::roll_dice(1, quip.available.len() as i32)-1) as usize };

                crate::gamelog::Logger::new()
                    .npc_name(&name.name)
                    .append("says")
                    .item_name(&quip.available[quip_index])
                    .log();
                quip.available.remove(quip_index);
            }
        }
    }
}
