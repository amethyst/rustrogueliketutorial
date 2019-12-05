# Cursed Items and Mitigation Thereof

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Now that we have a solid magical items framework, it's time to add in cursed items. These are a mainstay of the Roguelike genre, albeit one that if over-used can really annoy your players! Cursed items are part of the item identification mini-game: they provide a risk to equipping/using an item before you know what it does. If there's no risk to equipping everything you find, the player will do just that to find out what they are - and the mini-game is pointless. On the other hand, if there are *too many* cursed items, the player will become extremely conservative in item use and won't touch things until they know for sure what they are. So, like many things in life, it's a tough balance to strike.

## Your Basic Longsword -1

As a simple example, we'll start by implementing a cursed longsword. We already have a `Longsword +1`, so it's relatively easy to define the JSON (from `spawns.json`) for one that has penalties instead of benefits:

```json
{
    "name" : "Longsword -1",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "attribute" : "might",
        "base_damage" : "1d8-1",
        "hit_bonus" : -1
    },
    "weight_lbs" : 2.0,
    "base_value" : 100.0,
    "initiative_penalty" : 3,
    "vendor_category" : "weapon",
    "magic" : { "class" : "common", "naming" : "Unidentified Longsword", "cursed" : true }
},
```

You'll notice that there's a to-hit and damage penalty, more of an initiative penalty, and we've added `cursed: true` to the `magic` section. Most of this already just works, but the `cursed` part is new. To start supporting this, we open up `raws/item_structs.rs` and add in template support:

```rust
#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
    pub cursed: Option<bool>
}
```

We've made it an `Option` - so you don't have to specify it for non-cursed items. Now we need a new component to indicate that an item is, in fact, cursed. In `components.rs` (and registered in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct CursedItem {}
```

Next up, we'll adjust `spawn_named_item` in `raws/rawmaster.rs` to handle adding the `CursedItem` component to cursed items:

```rust
if let Some(magic) = &item_template.magic {
    let class = match magic.class.as_str() {
        "rare" => MagicItemClass::Rare,
        "legendary" => MagicItemClass::Legendary,
        _ => MagicItemClass::Common
    };
    eb = eb.with(MagicItem{ class });

    if !identified.contains(&item_template.name) {
        match magic.naming.as_str() {
            "scroll" => {
                eb = eb.with(ObfuscatedName{ name : scroll_names[&item_template.name].clone() });
            }
            "potion" => {
                eb = eb.with(ObfuscatedName{ name: potion_names[&item_template.name].clone() });
            }
            _ => {
                eb = eb.with(ObfuscatedName{ name : magic.naming.clone() });
            }
        }
    }

    if let Some(cursed) = magic.cursed {
        if cursed { eb = eb.with(CursedItem{}); }
    }
}
```

Let's pop back to `spawns.json` and give them a chance to spawn. For now, we'll make them appear *everywhere* so it's easy to test them:

```json
{ "name" : "Longsword -1", "weight" : 100, "min_depth" : 1, "max_depth" : 100 },
```

That gets us far enough that you can run the game, and cursed longswords will appear and have poor combat performance. Identification already works, so equipping a cursed sword tells you what it is - but there's absolutely no penalty for doing so, other than it having poor stats when you use it. That's a great start!

## Letting the player know that it's cursed

In `gui.rs`, we carefully color items by class in `get_item_color`. We'd like cursed items to go red - but *only* if you know that they are cursed (so you don't look at your inventory list and see "oh, that's cursed - better not equip it!"). So let's modify that function to provide this functionality:

```rust
pub fn get_item_color(ecs : &World, item : Entity) -> RGB {
    let dm = ecs.fetch::<crate::map::MasterDungeonMap>();
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<CursedItem>().get(item).is_some() && dm.identified_items.contains(&name.name) {
            return RGB::from_f32(1.0, 0.0, 0.0);
        }
    }

    if let Some(magic) = ecs.read_storage::<MagicItem>().get(item) {
        match magic.class {
            MagicItemClass::Common => return RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => return RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => return RGB::from_f32(0.71, 0.15, 0.93)
        }
    }
    RGB::from_f32(1.0, 1.0, 1.0)
}
```

(screenshot)

## Preventing the unequipping of cursed items

The easy case for preventing removal is in the `inventory_system/remove_system.rs`: it simply takes an item and puts it into your backpack. We can just make this conditional, and we're good to go! Here's the source code for the system:

```rust
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
                gamelog.entries.insert(0, format!("You cannot remove {}, it is cursed", names.get(to_remove.item).unwrap().name));
            } else {
                equipped.remove(to_remove.item);
                backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            }
        }

        wants_remove.clear();
    }
}
```

The case of `equip_use.rs` is a bit more complicated. We equip the item, scan for items to replace and unequip the replacement item. We'll have to change this around a bit: look for what to remove, see if its cursed (and cancel if it is, with a message), and then if it is still valid actually perform the swap. We also want to *not* identify the item if you can't equip it, to avoid giving a fun backdoor in which you equip a cursed item and then use it to identify all the other cursed items! We can adjust the system like this:

```rust
use specs::prelude::*;
use super::{Name, InBackpack, gamelog::GameLog, WantsToUseItem, Equippable, Equipped, EquipmentChanged,
    IdentifiedItem, CursedItem};

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
                        WriteStorage<'a, IdentifiedItem>,
                        ReadStorage<'a, CursedItem>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_use, names, equippable, 
            mut equipped, mut backpack, mut dirty, mut identified_item, cursed) = data;

        let mut remove_use : Vec<Entity> = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;

                // Remove any items the target has in the item's slot
                let mut can_equip = true;
                let mut log_entries : Vec<String> = Vec::new();
                let mut to_unequip : Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        if cursed.get(item_entity).is_some() {
                            can_equip = false;
                            gamelog.entries.insert(0, format!("You cannot unequip {}, it is cursed.", name.name)); 
                        } else {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                log_entries.push(format!("You unequip {}.", name.name));                                
                            }
                        }
                    }
                }

                if can_equip {
                    // Identify the item
                    if target == *player_entity {
                        identified_item.insert(target, IdentifiedItem{ name: names.get(useitem.item).unwrap().name.clone() })
                            .expect("Unable to insert");
                    }
                    

                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert backpack entry");
                    }

                    for le in log_entries.iter() {
                        gamelog.entries.insert(0, le.to_string());
                    }

                    // Wield the item
                    equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.insert(0, format!("You equip {}.", names.get(useitem.item).unwrap().name));
                    }
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
```

We've moved identification down beneath the item scanning, and added a `can_use` bool; if the switch would result in unequipping a cursed item, we cancel the job.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-64-curses)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-64-curses)
---

Copyright (C) 2019, Herbert Wolverson.

---