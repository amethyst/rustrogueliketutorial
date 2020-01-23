use specs::prelude::*;
use super::{Name, InBackpack, gamelog::GameLog, WantsToUseItem, Equippable, Equipped, 
    EquipmentChanged, IdentifiedItem};

pub struct ItemEquipOnUse {}

impl<'a> System<'a> for ItemEquipOnUse {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                        WriteStorage<'a, IdentifiedItem>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_use, names, equippable, 
            mut equipped, mut backpack, mut dirty, mut identified_item) = data;

        let mut remove_use : Vec<Entity> = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;

                // Identify the item
                if target == *player_entity {
                    identified_item.insert(target, IdentifiedItem{ name: names.get(useitem.item).unwrap().name.clone() })
                        .expect("Unable to insert");
                }

                // Remove any items the target has in the item's slot
                let mut to_unequip : Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                        if target == *player_entity {
                            gamelog.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                }
                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                }

                // Wield the item
                equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                backpack.remove(useitem.item);
                if target == *player_entity {
                    gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                }

                // Done with item
                remove_use.push(target);
            }
        }

        remove_use.iter().for_each(|e| { 
            dirty.insert(*e, EquipmentChanged{}).expect("Unable to insert");
            wants_use.remove(*e).expect("Unable to remove"); 
        });
    }
}
