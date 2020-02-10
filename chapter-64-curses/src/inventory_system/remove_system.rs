use specs::prelude::*;
use super::{InBackpack, Equipped, WantsToRemoveItem, CursedItem, Name};

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
                        Entities<'a>,
                        WriteStorage<'a, WantsToRemoveItem>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
                        ReadStorage<'a, CursedItem>,
                        WriteExpect<'a, crate::gamelog::GameLog>,
                        ReadStorage<'a, Name>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack, cursed, mut gamelog, names) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            if cursed.get(to_remove.item).is_some() {
                gamelog.entries.push(format!("You cannot remove {}, it is cursed", names.get(to_remove.item).unwrap().name));
            } else {
                equipped.remove(to_remove.item);
                backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            }
        }

        wants_remove.clear();
    }
}
