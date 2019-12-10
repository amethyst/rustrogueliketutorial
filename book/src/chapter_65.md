# More Magic Items

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

There are still a few common item types that we aren't supporting, yet. This chapter will finish that up, and lay the framework for spellcasting (in the next chapter).

## Items that improve attributes

A common type of item frequently round in D&D-likes are items that enhance (or reduce!) your attributes. For example, *Gauntlets of Ogre Power* grant a might bonus or a *Hat of Wizardry* grants intelligence. We have most of the framework in place to support these items, so let's go the final mile to make them work! Open up `spawns.json`, and we'll define what the gauntlets might look like:

```json
{
    "name" : "Gauntlets of Ogre Power",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00FF00",
        "bg" : "#000000",
        "order" : 2
    },
    "wearable" : {
        "slot" : "Hands",
        "armor_class" : 0.1,
        "might" : 5
    },
    "weight_lbs" : 1.0,
    "base_value" : 300.0,
    "initiative_penalty" : 0.0,
    "vendor_category" : "armor",
    "magic" : { "class" : "common", "naming" : "Unidentified Gauntlets" },
    "attributes" : { "might" : 5 }
}
```

Why didn't we just add this to "wearable"? We might want to grant an attribute boost to other things! To support loading this - and other attribute boosts - we need to edit `item_structs.rs`:

```rust
#[derive(Deserialize, Debug)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub wearable : Option<Wearable>,
    pub initiative_penalty : Option<f32>,
    pub weight_lbs : Option<f32>,
    pub base_value : Option<f32>,
    pub vendor_category : Option<String>,
    pub magic : Option<MagicItem>,
    pub attributes : Option<ItemAttributeBonus>
}
...
#[derive(Deserialize, Debug)]
pub struct ItemAttributeBonus {
    pub might : Option<i32>,
    pub fitness : Option<i32>,
    pub quickness : Option<i32>,
    pub intelligence : Option<i32>
}
```

As we've done before, we'll need a component to support this data. In `components.rs` (and registered in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct AttributeBonus {
    pub might : Option<i32>,
    pub fitness : Option<i32>,
    pub quickness : Option<i32>,
    pub intelligence : Option<i32>
}
```

And we'll modify `raws/rawmaster.rs`'s function `spawn_named_item` to support adding this component type:

```rust
if let Some(ab) = &item_template.attributes {
    eb = eb.with(AttributeBonus{
        might : ab.might,
        fitness : ab.fitness,
        quickness : ab.quickness,
        intelligence : ab.intelligence,
    });
}
```

Now that the component can be applied to an item, lets put it into the spawn table as ridiculously common to make testing easy:

```json
{ "name" : "Gauntlets of Ogre Power", "weight" : 100, "min_depth" : 0, "max_depth" : 100 },
```

Finally, we need to make it actually do something. We're doing something very similar in `ai/encumbrance_system.rs` - so that's the natural place to slot it in. We'll add a lot to the system, so here's the whole thing:

```rust
extern crate specs;
use specs::prelude::*;
use crate::{EquipmentChanged, Item, InBackpack, Equipped, Pools, Attributes, gamelog::GameLog, AttributeBonus,
    gamesystem::attr_bonus};
use std::collections::HashMap;

pub struct EncumbranceSystem {}

impl<'a> System<'a> for EncumbranceSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, AttributeBonus>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut equip_dirty, entities, items, backpacks, wielded,
            mut pools, mut attributes, player, mut gamelog, attrbonus) = data;

        if equip_dirty.is_empty() { return; }

        struct ItemUpdate {
            weight : f32,
            initiative : f32,
            might : i32,
            fitness : i32,
            quickness : i32,
            intelligence : i32
        }

        // Build the map of who needs updating
        let mut to_update : HashMap<Entity, ItemUpdate> = HashMap::new(); // (weight, intiative)
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, ItemUpdate{ weight: 0.0, initiative: 0.0, might: 0, fitness: 0, quickness: 0, intelligence: 0 });
        }

        // Remove all dirty statements
        equip_dirty.clear();

        // Total up equipped items
        for (item, equipped, entity) in (&items, &wielded, &entities).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                totals.weight += item.weight_lbs;
                totals.initiative += item.initiative_penalty;
                if let Some(attr) = attrbonus.get(entity) {
                    println!("Attr! {:?}", attr);
                    totals.might += attr.might.unwrap_or(0);
                    totals.fitness += attr.fitness.unwrap_or(0);
                    totals.quickness += attr.quickness.unwrap_or(0);
                    totals.intelligence += attr.intelligence.unwrap_or(0);
                }
            }
        }

        // Total up carried items
        for (item, carried, entity) in (&items, &backpacks, &entities).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                totals.weight += item.weight_lbs;
                totals.initiative += item.initiative_penalty;
            }
        }

        // Apply the data to Pools
        for (entity, item) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = item.weight;
                pool.total_initiative_penalty = item.initiative;

                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.might.modifiers = item.might;
                    attr.fitness.modifiers = item.fitness;
                    attr.quickness.modifiers = item.quickness;
                    attr.intelligence.modifiers = item.intelligence;
                    attr.might.bonus = attr_bonus(attr.might.base + attr.might.modifiers);
                    attr.fitness.bonus = attr_bonus(attr.fitness.base + attr.fitness.modifiers);
                    attr.quickness.bonus = attr_bonus(attr.quickness.base + attr.quickness.modifiers);
                    attr.intelligence.bonus = attr_bonus(attr.intelligence.base + attr.intelligence.modifiers);

                    let carry_capacity_lbs = (attr.might.base + attr.might.modifiers) * 15;
                    if pool.total_weight as i32 > carry_capacity_lbs {
                        // Overburdened
                        pool.total_initiative_penalty += 4.0;
                        if *entity == *player {
                            gamelog.entries.insert(0, "You are overburdened, and suffering an initiative penalty.".to_string());
                        }
                    }
                }
            }
        }
    }
}
```

TODO: Description



## Charged items

## Status Effects

## Damage Types

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-65-items)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-65-items)
---

Copyright (C) 2019, Herbert Wolverson.

---