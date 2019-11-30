# Effects

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the last chapter, we added item identification to magical items - and it became clear that potentially there are *lots* of items we could create. Our inventory system is seriously overloaded - it does *way* too much in one place, ranging from equipping/unequipping items to the guts of making magic missile spells fly. Worse, we've silently run into a wall: Specs limits the number of data stores you can pass into a system (and will probably continue to do so until Rust supports C++ style variadic parameter packs). We *could* just hack around that problem, but it would be far better to solve the problem once and for all by implementing a more generic solution. It also lets us solve a problem we don't know we have yet: handling effects from things other than items, such as spells (or traps that do zany things, etc.). This is also an opportunity to fix a bug you may not have noticed; an entity can only have one component of a given type, so if two things have issued damage to a component in a given tick - only the one piece of damage actually happens!

## What is an effect?

To properly model effects, we need to think about what they are. An effect is *something doing something*. It might be a sword hitting a target, a spell summoning a great demon from Abyss, or a wand clearing summoning a bunch of flowers - pretty much anything, really! We want to keep the ability for things to cause more than one effect (if you added multiple components to an item, it would fire all of them - that's a good thing; a *staff of thunder and lightning* could easily have two or more effects!). So from this, we can deduce:

* An effect does *one* thing - but the source of an effect might spawn multiple effects. An effect, therefore, is a good candidate to be its own `Entity`.
* An effect has a source: someone has to get experience points if it kills someone, for example. It also needs to have the option to *not* have a source - it might be purely environmental.
* An effect has one or more targets; it might be self-targeted, targeted on one other, or an area-of-effect. Targets are therefore either an entity or a location.
* An effect might trigger the creation of other effects in a chain (think *chain lightning*, for example).
* An effect *does something*, but we don't really want to specify exactly what in the early planning stages!
* We want effects to be sourced from multiple places: using an item, triggering a trap, a monster's special attack, a magical weapon's "proc" effect, casting a spell, or even environmental effects!

So, we're not asking for much! Fortunately, this is well within what we can manage with an ECS. We're going to stretch the "S" (Systems) a little and use a more generic *factory* model to actually create the effects - and then reap the benefits of a relatively generic setup once we have that in place.

## Inventory System: Quick Clean Up

Before we get too far in, we should take a moment to break up the inventory system into a module. We'll retain exactly the functionality it already has (for now), but it's a monster - and monsters are generally better handled in chunks! Make a new folder, `src/inventory_system` and move `inventory_system.rs` into it - and rename it `mod.rs`. That converts it into a multi-file module. (Those steps are actually enough to get you a runnable setup - this is a good illustration of how modules work in Rust; a file named `inventory_system.rs` *is* a module, and so is `inventory_system/mod.rs`).

Now open up `inventory_system/mod.rs`, and you'll see that it has a bunch of systems in it:

* `ItemCollectionSystem`
* `ItemUseSystem`
* `ItemDropSystem`
* `ItemRemoveSystem`
* `ItemIdentificationSystem`

We're going to make a new file for each of these, cut the systems code out of `mod.rs` and paste it into its own file. We'll need to copy the `use` part of `mod.rs` to the top of these files, and then trim out what we aren't using. At the end, we'll add `mod X`, `use X::SystemName` lines in `mod.rs` to tell the compiler that the module is sharing these systems. This would be a *huge* chapter if I pasted in each of these changes, and since the largest - `ItemUseSystem` is going to change drastically, that would be a rather large waste of space. Instead, we'll go through the first - and you can [check the source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-63-effects/src/inventory_system/) to see the rest.

For example, we make a new file `inventory_system/collection_system.rs`:

```rust
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog, EquipmentChanged };

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names,
            mut backpack, mut dirty) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                gamelog.entries.insert(0, format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}
```

This is *exactly* the code from the original system, which is why we aren't repeating all of them here. The only difference is that we've gone through the `use super::` list at the top and trimmed out what we aren't using. You can do the same for `inventory_system/drop_system.rs`, `inventory_system/identification_system.rs`, `inventory_system/remove_system.rs` and `use_system.rs`. Then you tie them together into `inventory_system/mod.rs`:

```rust
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog, WantsToUseItem,
    Consumable, ProvidesHealing, WantsToDropItem, InflictsDamage, Map, SufferDamage,
    AreaOfEffect, Confusion, Equippable, Equipped, WantsToRemoveItem, particle_system,
    ProvidesFood, HungerClock, HungerState, MagicMapper, RunState, Pools, EquipmentChanged,
    TownPortal, IdentifiedItem, Item, ObfuscatedName};

mod collection_system;
pub use collection_system::ItemCollectionSystem;
mod use_system;
pub use use_system::ItemUseSystem;
mod drop_system;
pub use drop_system::ItemDropSystem;
mod remove_system;
pub use remove_system::ItemRemoveSystem;
mod identification_system;
pub use identification_system::ItemIdentificationSystem;
```

We've tweaked a couple of `use` paths to make the other components happy, and then added a pair of `mod` (to use the file) and `pub use` (to share it with the rest of the project).

If all went well, `cargo run` will give you the exact same game we had before! It should even compile a bit faster.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-63-effects)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-63-effects)
---

Copyright (C) 2019, Herbert Wolverson.

---