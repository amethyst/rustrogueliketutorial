use specs::prelude::*;
use super::{Name, InBackpack, Position, WantsToDropItem, EquipmentChanged,
    MagicItem, ObfuscatedName, MasterDungeonMap};

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                        ReadStorage<'a, MagicItem>,
                        ReadStorage<'a, ObfuscatedName>,
                        ReadExpect<'a, MasterDungeonMap>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, entities, mut wants_drop, names, mut positions,
            mut backpack, mut dirty, magic_items, obfuscated_names, dm) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            if entity == *player_entity {
                crate::gamelog::Logger::new()
                    .append("You drop the")
                    .color(rltk::CYAN)
                    .append(
                        super::obfuscate_name(to_drop.item, &names, &magic_items, &obfuscated_names, &dm)
                    )
                    .log();
            }
        }

        wants_drop.clear();
    }
}
