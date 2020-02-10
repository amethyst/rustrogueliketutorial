use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog, WantsToUseItem,
    WantsToDropItem, Map, AreaOfEffect, Equippable, Equipped, WantsToRemoveItem, EquipmentChanged,
    IdentifiedItem, Item, ObfuscatedName, MagicItem, MasterDungeonMap, CursedItem, WantsToCastSpell };

mod collection_system;
pub use collection_system::ItemCollectionSystem;
mod use_system;
pub use use_system::{ItemUseSystem, SpellUseSystem};
mod drop_system;
pub use drop_system::ItemDropSystem;
mod remove_system;
pub use remove_system::ItemRemoveSystem;
mod identification_system;
pub use identification_system::ItemIdentificationSystem;
mod equip_use;
pub use equip_use::ItemEquipOnUse;
use specs::prelude::*;

pub fn obfuscate_name(
    item: Entity, 
    names: &ReadStorage::<Name>, 
    magic_items : &ReadStorage::<MagicItem>,
    obfuscated_names : &ReadStorage::<ObfuscatedName>,
    dm : &MasterDungeonMap,
) -> String 
{
    if let Some(name) = names.get(item) {
        if magic_items.get(item).is_some() {
            if dm.identified_items.contains(&name.name) {
                name.name.clone()
            } else if let Some(obfuscated) = obfuscated_names.get(item) {
                obfuscated.name.clone()
            } else {
                "Unidentified magic item".to_string()
            }
        } else {
            name.name.clone()
        }

    } else {
        "Nameless item (bug)".to_string()
    }
}
