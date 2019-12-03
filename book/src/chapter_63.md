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

## A new effects module

We'll start with the basics. Make a new folder, `src/effects` and place a single file in it called `mod.rs`. As you've seen before, this creates a basic module named *effects*. Now for the fun part; we need to be able to *add* effects from anywhere, including within a system: so passing in the `World` isn't available. However, *spawning* effects will need full `World` access! So, we're going to make a queueing system. Calls in *enqueue* an effect, and a later scan of the *queue* causes effects to fire. This is basically a *message passing system*, and you'll often find something similar codified into big game engines. So here's a very simple `effects/mod.rs` (also add `pub mod effects;` to the `use` list in `main.rs` to include it in your compilation and make it available to other modules):

```rust
use std::sync::Mutex;
use specs::prelude::*;
use std::collections::VecDeque;

lazy_static! {
    pub static ref EFFECT_QUEUE : Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType { 
    Damage { amount : i32 }
}

#[derive(Clone)]
pub enum Targets {
    Single { target : Entity },
    Area { target: Vec<Entity> }
}

pub struct EffectSpawner {
    pub creator : Option<Entity>,
    pub effect_type : EffectType,
    pub targets : Targets
}

pub fn add_effect(creator : Option<Entity>, effect_type: EffectType, targets : Targets) {
    EFFECT_QUEUE
        .lock()
        .unwrap()
        .push_back(EffectSpawner{
            creator,
            effect_type,
            targets
        });
}
```

If you are using an IDE, it will complain that none of this is used. That's ok, we're building basic functionality first! The `VecDeque` is new; it's a *queue* (actually a double-ended queue) with a vector behind it for performance. It lets you add to either end, and `pop` results off of it. See [the documentation](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) to learn more about it.

## Enqueueing Damage

Let's start with a relatively simple one. Currently, whenever an entity is damaged we assign it a `SufferDamage` component. That works ok, but has the problem we discussed earlier - there can only be one source of damage at a time. We want to concurrently murder our player in many ways (only slightly kidding)! So we'll extend the base to permit inserting damage. We'll change `EffectType` to have a `Damage` type:

```rust
pub enum EffectType { 
    Damage { amount : i32 }
}
```

Notice that we're not storing the victim or the originator - those are covered in the *source* and *target* parts of the message. Now we search our code to see where we use `SufferDamage` components. The most important users are the hunger system, melee system, item use system and trigger system: they can all cause damage to occur. Open up `melee_combat_system.rs` and find the following line (it's line 106 in my source code):

```rust
inflict_damage.insert(wants_melee.target,
    SufferDamage{
        amount: damage,
        from_player: entity == *player_entity
    }
).expect("Unable to insert damage component");
```

We can replace this with a call to insert into the queue:

```rust
add_effect(
    Some(entity),
    EffectType::Damage{ amount: damage },
    Targets::Single{ target: wants_melee.target }
);
```

We can also remove all references to `inflict_damage` from the system, since we aren't using it anymore.

We should do the same for `trigger_system.rs`. We can replace the following line:

```rust
 inflict_damage.insert(entity, SufferDamage{ amount: damage.damage, from_player: false }).expect("Unable to do damage");
```

With:

```rust
add_effect(
    None,
    EffectType::Damage{ amount: damage.damage },
    Targets::Single{ target: entity }
);
```

Once again, we can also get rid of all references to `SufferDamage`.

We'll ignore `item_use_system` for a minute (we'll get to it in a moment, I promise).

## Applying Damage

So now if you hit something, you are adding damage to the queue (and nothing else happens). The next step is to read the effects queue and do something with it. We're going to adopt a *dispatcher* model for this: read the queue, and *dispatch* commands to the relevant places. We'll start with the skeleton; in `effects/mod.rs` we add the following function:

```rust
pub fn run_effects_queue(ecs : &mut World) {
    loop {
        let effect : Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            // target_applicator(ecs, &effect); // Uncomment when we write this!
        } else {
            break;        
        }
    }
}
```

This is very minimal! It acquires a lock just long enough to pop the first message from the queue, and if it has a value - does something with it. It then repeats the lock/pop cycle until the queue is completely empty. This is a useful pattern: the lock is only held for *just* long enough to read the queue, so if any systems inside want to add to the queue you won't experience a "deadlock" (two systems perpetually waiting for queue access).

It doesn't do anything with the data, yet - but this shows you how to drain the queue one message at a time. We're taking in the `World`, because we expect to be modifying it. We should add a call to use this function; in `main.rs` find `run_systems` and add it almost at the very end (with particles and lighting after it):

```rust
effects::run_effects_queue(&mut self.ecs);
let mut particles = particle_system::ParticleSpawnSystem{};
particles.run_now(&self.ecs);
let mut lighting = lighting_system::LightingSystem{};
lighting.run_now(&self.ecs);
```

Now that we're draining the queue, lets do something with it. In `effects/mod.rs`, we'll add in the commented-out function `target_applicator`. The idea is to take the `TargetType`, and extend it into calls that handle it (the function has a high "fan out" - meaning we'll call it a lot, and it will call many other functions). There's a few different ways we can affect a target, so here's several related functions:

```rust
fn target_applicator(ecs : &mut World, effect : &EffectSpawner) {
    match &effect.targets {
        Targets::Tile{tile_idx} => affect_tile(ecs, effect, *tile_idx),
        Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
        Targets::Single{target} => affect_entity(ecs, effect, *target),
        Targets::TargetList{targets} => targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true
    }
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx : i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let content = ecs.fetch::<Map>().tile_content[tile_idx as usize].clone();
        content.iter().for_each(|entity| affect_entity(ecs, effect, *entity));
    }
    // TODO: Run the effect
}

fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    // TODO: Run the effect
}
```

There's a lot to unwrap here, but it gives a *very* generic mechanism for handling effect targeting. Let's step through it:

1. `target_applicator` is called.
2. It matches on the `targets` field of the effect:
    1. If it is a `Tile` target type, it calls `Targets::tile` with the index of the target tile.
        1. `affect_tile` calls another function, `tile_effect_hits_entities` which looks at the requested effect type and determines if it should be applied to entities inside the tile. Right now, we only have `Damage` - which makes sense to pass on to entities, so it currently always returns true.
        2. If it does affect entities in the tile, then it retrieves the tile content from the map - and calls `affect_entity` on each entity in the tile. We'll look at that in a moment.
        3. If there is something to do with the tile, it happens here. Right now, it's a `TODO` comment.
    2. If it is a `Tiles` target type, it iterates through *all* of the tiles in the list, calling `affect_tile` on each of them in turn - just like a single tile (above), but covering each of them.
    3. If it is a `Single` entity target, it calls `affect_entity` for that target.
    4. If it a `TargetList` (a list of target entities), it calls `affect_entity` for each of those target entities in turn.

So this framework lets us have an effect that can hit a tile (and optionally everyone in it), a set of tiles (again, optionally including the contents), a single entity, or a list of entities. You can describe pretty much any targeting mechanism with that!

Next, in the `run_effects_queue` function, uncomment the caller (so our hard work actually runs!):

```rust
pub fn run_effects_queue(ecs : &mut World) {
    loop {
        let effect : Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, &effect);
        } else {
            break;        
        }
    }
}
```

Going back to the `Damage` type we are implementing, we need to implement it! We'll make a new file, `effects/damage.rs` and put code to apply damage into it. Damage is a one-shot, non-persistent thing - so we'll handle it immediately. Here's the bare-bones:

```rust
use specs::prelude::*;
use super::*;
use crate::components::Pools;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage{amount} = damage.effect_type {
                pool.hit_points.current -= amount;
            }
        }
    }
}
```

Notice that we're not handling blood stains, experience points or anything of the like! We are, however, applying the damage. If you `cargo run` now, you can engage in melee (and not gain any benefits to doing so).

### Blood for the blood god!

Our previous version spawned bloodstains whenever we inflicted damage. It would have been easy enough to include this in the `inflict_damage` function above, but we may have a use for bloodstains elsewhere! We also need to verify that our effects message queue really is smart enough to handle insertions during events. So we're going to make bloodstains an effect. We'll add it into the `EffectType` enum in `effects/mod.rs`:

```rust
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain
}
```

Bloodstains have no effect on entities in the (now messy) tile, so we'll update `tile_effect_hits_entities` to have a default of not doing anything (this way we can keep adding cosmetic effects without having to remember to add it each time):

```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        _ => false
    }
}
```

Likewise, `affect_entity` can ignore the event - and other cosmetic events:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        _ => {}
    }
}
```

We *do* want it to affect a tile, so we'll update `affect_tile` to call a bloodstain function.

```rust
fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx : i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let content = ecs.fetch::<Map>().tile_content[tile_idx as usize].clone();
        content.iter().for_each(|entity| affect_entity(ecs, effect, *entity));
    }
    
    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        _ => {}
    }
}
```

Now, in `effects/damage.rs` we'll write the bloodstain code:

```rust
pub fn bloodstain(ecs: &mut World, tile_idx : i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}
```

We'll also update `inflict_damage` to spawn a bloodstain:

```rust
pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage{amount} = damage.effect_type {
                pool.hit_points.current -= amount;
                if let Some(tile_idx) = entity_position(ecs, target) {
                    add_effect(None, EffectType::Bloodstain, Targets::Tile{tile_idx});
                }
            }
        }
    }
}
```

The relevant code asks a mystery function, `entity_position` for data - if it returns a value, it inserts an effect of the `Bloodstain` type with the tile index. So what is this function? We're going to be targeting a lot, so we should make some helper functions to make the process easier for the caller. Make a new file, `effects/targeting.rs` and place the following into it:

```rust
use specs::prelude::*;
use crate::components::Position;
use crate::map::Map;

pub fn entity_position(ecs: &World, target: Entity) -> Option<i32> {
    if let Some(pos) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }
    None
}
```

Now in `effects/mods.rs` add a couple of lines to expose the targeting helpers to consumers of the effects module:

```rust
mod targeting;
pub use targeting::*;
```

So what does this do? It follows a pattern we've used a lot: it checks to see if the entity *has* a position. If it does, then it obtains the tile index from the global map and returns it - otherwise, it returns `None`.

If you `cargo run` now, and attack an innocent rodent you will see blood! We've proven that the events system doesn't deadlock, and we've added an easy way to add bloodstains. You can call that event from anywhere, and blood shall rain!

### Particulate Matter

You've probably noticed that when an entity takes damage, we spawn a particle. Particles are something else we can use a *lot*, so it makes sense to have them as an event type also. Whenever we've applied damage so far, we've flashed an orange indicator over the victim. We might as well codify that in the damage system (and leave it open for improvement in a later chapter). It's likely that we'll want to launch particles for other purposes, too - so we'll come up with another quite generic setup.

We'll start in `effects/mod.rs` and extend `EffectType` to include particles:

```rust
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 }
}
```

You'll notice that once again, we aren't specifying *where* the particle goes; we'll leave that to the targeting system. Now we'll make a function to actually spawn particles. In the name of clarity, we'll put it in its own file; in a new file `effects/particles.rs` add the following:

```rust
use specs::prelude::*;
use super::*;
use crate::particle_system::ParticleBuilder;
use crate::map::Map;

pub fn particle_to_tile(ecs: &mut World, tile_idx : i32, effect: &EffectSpawner) {
    if let EffectType::Particle{ glyph, fg, bg, lifespan } = effect.effect_type {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        particle_builder.request(
            tile_idx % map.width, 
            tile_idx / map.width, 
            fg, 
            bg, 
            glyph, 
            lifespan
        );
    }
}
```

This is basically the same as our other calls to `ParticleBuilder`, but using the contents of the message to define what to build. Now we'll go back to `effects/mod.rs` and add a `mod particles;` to the using list at the top. Then we'll extend the `affect_tile` to call it:

```rust
fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx : i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let content = ecs.fetch::<Map>().tile_content[tile_idx as usize].clone();
        content.iter().for_each(|entity| affect_entity(ecs, effect, *entity));
    }
    
    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        EffectType::Particle{..} => particles::particle_to_tile(ecs, tile_idx, &effect),
        _ => {}
    }
}
```

It would also be really handy to be able to attach a particle to an entity, even if it doesn't actually have much effect. There's been a few cases where we've retrieved a `Position` component just to place an effect, so this could let us simplify the code a bit! So we extend `affect_entity` like this:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        _ => {}
    }
}
```

So now we can open up `effects/damage.rs` and both clean-up the bloodstain code and apply a damage particle:

```rust
pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
            if let EffectType::Damage{amount} = damage.effect_type {
                pool.hit_points.current -= amount;
                add_effect(None, EffectType::Bloodstain, Targets::Single{target});
                add_effect(None, 
                    EffectType::Particle{ 
                        glyph: rltk::to_cp437('‼'),
                        fg : rltk::RGB::named(rltk::ORANGE),
                        bg : rltk::RGB::named(rltk::BLACK),
                        lifespan: 200.0
                    }, 
                    Targets::Single{target}
                );
            }
        }
    }
}
```

Now open up `melee_combat_system.rs`. We can simplify it a bit by removing the particle call on damage, and replace the other calls to `ParticleBuilder` with effect calls. This lets us get rid of the whole reference to the particle system, positions AND the player entity! *This* is the kind of improvement I wanted: systems are simplifying down to what they *should* focus on! See [the source](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-63-effects/src/melee_combat_system.rs) for the changes; they are too long to include in the body text here.

If you `cargo run` now, you'll see particles if you damage something - and bloodstains should still work.

### Experience Points

So we're missing some important stuff, still: when you kill a monster, it should drop loot/cash, give experience points and so on. Rather than pollute the "damage" function with too much extraneous stuff (on the principle of a function doing one thing well), lets add a new `EffectType` - `EntityDeath`:

```rust
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath
}
```

Now in `inflict_damage`, we'll emit this event if the entity died:

```rust
if pool.hit_points.current < 1 {
    add_effect(damage.creator, EffectType::EntityDeath, Targets::Single{target});
}
```

We'll also make a new function; this is the same as the code in `damage_system` (we'll be removing most of the system when we've taken care of item usage):

```rust
pub fn death(ecs: &mut World, effect: &EffectSpawner, target : Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    let mut pools = ecs.write_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let mut map = ecs.fetch_mut::<Map>();

    if let Some(pos) = entity_position(ecs, target) {
        map.blocked[pos as usize] = false;
    }

    if let Some(source) = effect.creator {
        if ecs.read_storage::<Player>().get(source).is_some() {
            if let Some(stats) = pools.get(target) {
                xp_gain += stats.level * 100;
                gold_gain += stats.gold;
            }

            if xp_gain != 0 || gold_gain != 0.0 {
                let mut log = ecs.fetch_mut::<GameLog>();
                let mut player_stats = pools.get_mut(source).unwrap();
                let player_attributes = attributes.get(source).unwrap();
                player_stats.xp += xp_gain;
                player_stats.gold += gold_gain;
                if player_stats.xp >= player_stats.level * 1000 {
                    // We've gone up a level!
                    player_stats.level += 1;
                    log.entries.insert(0, format!("Congratulations, you are now level {}", player_stats.level));
                    player_stats.hit_points.max = player_hp_at_level(
                        player_attributes.fitness.base + player_attributes.fitness.modifiers,
                        player_stats.level
                    );
                    player_stats.hit_points.current = player_stats.hit_points.max;
                    player_stats.mana.max = mana_at_level(
                        player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                        player_stats.level
                    );
                    player_stats.mana.current = player_stats.mana.max;
    
                    let player_pos = ecs.fetch::<rltk::Point>();
                    for i in 0..10 {
                        if player_pos.y - i > 1 {
                            add_effect(None, 
                                EffectType::Particle{ 
                                    glyph: rltk::to_cp437('░'),
                                    fg : rltk::RGB::named(rltk::GOLD),
                                    bg : rltk::RGB::named(rltk::BLACK),
                                    lifespan: 400.0
                                }, 
                                Targets::Tile{ tile_idx : map.xy_idx(player_pos.x, player_pos.y - i) as i32 }
                            );
                        }
                    }
                }
            }
        }
    }
}
```

Lastly, we add the effect to `affect_entity`:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },        
        _ => {}
    }
}
```

So now if you `cargo run` the project, we're back to where we were - but with a much more flexible system for particles, damage (which now stacks!) and killing things in general.

## Item effects

Now that we have the basics of an effects system (and have cleaned up damage), it's time to really think about how items (and triggers) should work. We want them to be generic enough that you can put together entities Lego-style and build something interesting. We also want to stop defining effects in multiple places; currently we list trigger effects in one system, item effects in another - if we add spells, we'll have yet another place to debug!

We'll start by taking a look at the item usage system (`inventory_system/use_system.rs`). It's HUGE, and does far too much in one place. It handles targeting, identification, equipment switching, firing off effects for using an item and destruction of consumables! That was good for building a toy game to test with, but it doesn't scale to a "real" game.

For part of this - and in the spirit of using an ECS - we'll make some *more systems*, and have them do one thing well.

### Moving Equipment Around

Equipping (and swapping) items is currently in the item usage system because it fits there from a user interface perspective: you "use" a sword, and the logical way to use it is to hold it (and put away whatever you had in your hand). Having it be part of the item usage system makes the system overly confusing, though - the system simply does too much (and targeting really isn't an issue, since you are using it on yourself).

So we'll make a new system in the file `inventory_system/use_equip.rs` and move the functionality over to it. This leads to a compact new system:

```rust
use specs::prelude::*;
use super::{Name, InBackpack, gamelog::GameLog, WantsToUseItem, Equippable, Equipped, EquipmentChanged};

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
                        WriteStorage<'a, EquipmentChanged>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_use, names, equippable, 
            mut equipped, mut backpack, mut dirty) = data;

        let mut remove_use : Vec<Entity> = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if let Some(can_equip) = equippable.get(useitem.item) {
                let target_slot = can_equip.slot;

                // Remove any items the target has in the item's slot
                let mut to_unequip : Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                        if target == *player_entity {
                            gamelog.entries.insert(0, format!("You unequip {}.", name.name));
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
                    gamelog.entries.insert(0, format!("You equip {}.", names.get(useitem.item).unwrap().name));
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

Now go into `use_system.rs` and delete the same block. Finally, pop over to `main.rs` and add the system into `run_systems` (just before the current use system call):

```rust
let mut itemequip = inventory_system::ItemEquipOnUse{};
itemequip.run_now(&self.ecs);
...
let mut itemuse = ItemUseSystem{};
```

Go ahead and `cargo run` and switch some equipment around to make sure it still works. That's good progress - we can remove three complete component storages from our `use_system`!

### Item effects

Now that we've cleaned up inventory management into its own system, it's time to really cut to the meat of this change: item usage with effects. The goal is to have a system that understands items, but can "fan out" into generic code that we can reuse for every other effect use. We'll start in `effects/mod.rs` by adding an effect type for "I want to use an item":

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
}
```

We want these to work a little differently than regular effects (consumable use has to be handled, and targeting passes through to the actual effects rather than directly from the item). We'll add it into `target_applicator`:

```rust
fn target_applicator(ecs : &mut World, effect : &EffectSpawner) {
    if let EffectType::ItemUse{item} = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
    } else {
        match &effect.targets {
            Targets::Tile{tile_idx} => affect_tile(ecs, effect, *tile_idx),
            Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
            Targets::Single{target} => affect_entity(ecs, effect, *target),
            Targets::TargetList{targets} => targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity)),
        }
    }
}
```

This "short circuits" the calling tree, so it handles items once (the items can then emit other events into the queue, so it all gets handled). Since we've called it, now we have to write `triggers:item_trigger`! Make a new file, `effects/triggers.rs` (and in `mod.rs` add a `mod triggers;`):

```rust
pub fn item_trigger(creator : Option<Entity>, item: Entity, targets : &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(creator, item, targets, ecs);

    // If it was a consumable, then it gets deleted
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}
```

This function is the reason we have to handle items differently: it calls `event_trigger` (a local, private function) to spawn all the item's effects - and then if the item is a consumable it deletes it. Let's make a skeletal `event_trigger` function:

```rust
fn event_trigger(creator : Option<Entity>, entity: Entity, targets : &Targets, ecs: &mut World) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();
}
```

So this doesn't do anything - but the game can now compile and you can see that when you use an item it is correctly deleted. It provides enough of a placeholder to allow us to fix up the inventory system!

#### Use System Cleanup

 The `inventory_system/use_system.rs` file was the root cause of this cleanup, and we now have enough of a framework to make it into a reasonably small, lean system! We just need it to mark your equipment as having changed, build the appropriate `Targets` list, and add a usage event. Here's the entire new system:

 ```rust
 use specs::prelude::*;
use super::{Name, WantsToUseItem,Map, AreaOfEffect, EquipmentChanged, IdentifiedItem};
use crate::effects::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, EquipmentChanged>,
                        WriteStorage<'a, IdentifiedItem>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, map, entities, mut wants_use, names,
            aoe, mut dirty, mut identified_item) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            // Identify
            if entity == *player_entity {
                identified_item.insert(entity, IdentifiedItem{ name: names.get(useitem.item).unwrap().name.clone() })
                    .expect("Unable to insert");
            }

            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::ItemUse{ item : useitem.item },
                match useitem.target {
                    None => Targets::Single{ target: *player_entity },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(useitem.item) {
                            Targets::Tiles{ tiles: aoe_tiles(&*map, target, aoe.radius) }
                        } else {
                            Targets::Tile{ tile_idx : map.xy_idx(target.x, target.y) as i32 }
                        }
                    }
                }
            );

        }

        wants_use.clear();
    }
}
 ```

That's a big improvement! MUCH smaller, and quite easy to follow.

Now we need to work through the various item-related events and make them function.

#### Feeding Time

We'll start with food. Any item with a `ProvidesFood` component tag sets the eater's hunger clock back to `Well Fed`. We'll start by adding an event type for this:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
}
```

Now, we'll make a new file - `effects/hunger.rs` and put the meat of handling this into it (don't forget to add `mod hunger;` in `effects/mod.rs`!):

```rust
use specs::prelude::*;
use super::*;
use crate::components::{HungerClock, HungerState};

pub fn well_fed(ecs: &mut World, _damage: &EffectSpawner, target: Entity) {
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        hc.state = HungerState::WellFed;
        hc.duration = 20;
    }
}
```

Very simple, and straight out of the original code. We need food to affect entities rather than just locations (in case you make something like a vending machine that hands out food over an area!):


```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::WellFed => true,
        _ => false
    }
}
```

We also need to call the function:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        _ => {}
    }
}
```

Finally, we need to add it into the `event_trigger` function in `effects/triggers.rs`:

```rust
fn event_trigger(creator : Option<Entity>, entity: Entity, targets : &Targets, ecs: &mut World) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Providing food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog.entries.insert(0, format!("You eat the {}.", names.get(entity).unwrap().name));
    }
}
```

If you `cargo run` now, you can eat your rations and be well fed once more.

#### Magic Mapping

Magic Mapping is a bit of a special case, because of the need to switch back to the user interface for an update. It's also pretty simple, so we'll handle it entirely inside `event_trigger`:

```rust
// Magic mapper
if ecs.read_storage::<MagicMapper>().get(entity).is_some() {
    let mut runstate = ecs.fetch_mut::<RunState>();
    gamelog.entries.insert(0, "The map is revealed to you!".to_string());
    *runstate = RunState::MagicMapReveal{ row : 0};
}
```

Just like the code in the old item usage system: it sets the run-state to `MagicMapReveal` and plays a log message. You can `cargo run` and magic mapping will work now.

#### Town Portals

Town Portals are also a bit of a special case, so we'll also handle them in `event_trigger`:

```rust
// Town Portal
if ecs.read_storage::<TownPortal>().get(entity).is_some() {
    let map = ecs.fetch::<Map>();
    if map.depth == 1 {
        gamelog.entries.insert(0, "You are already in town, so the scroll does nothing.".to_string());
    } else {
        gamelog.entries.insert(0, "You are telported back to town!".to_string());
        let mut runstate = ecs.fetch_mut::<RunState>();
        *runstate = RunState::TownPortal;
    }
}
```

Once again, this is basically the old code - relocated.

#### Healing

Healing is a more generic effect, and it's likely that we'll use it in multiple places. It's easy to imagine a prop with an entry-trigger that heals you (a magical restoration zone, a cybernetic repair shop - your imagination is the limit!), or items that heal on use (such as potions). So we'll add `Healing` into the effect types in `mod.rs`:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Healing { amount : i32 },
    Confusion { turns : i32 }
}
```

Healing affects entities and not tiles, so we'll mark that:

```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::WellFed => true,
        EffectType::Healing{..} => true,
        _ => false
    }
}
```

Since healing is basically reversed damage, we'll add a function to handle healing into our `effects/damage.rs` file:

```rust
pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing{amount} = heal.effect_type {
            pool.hit_points.current = i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(None, 
                EffectType::Particle{ 
                    glyph: rltk::to_cp437('‼'),
                    fg : rltk::RGB::named(rltk::GREEN),
                    bg : rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0
                }, 
                Targets::Single{target}
            );
        }
    }
}
```

This is similar to the old healing code, but we've added in a green particle to show that the entity was healed. Now we need to teach `affect_entity` in `mod.rs` to apply healing:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing{..} => damage::heal_damage(ecs, effect, target),
        _ => {}
    }
}
```

Finally, we add support for `ProvidesHealing` tags in the `event_trigger` function:

```rust
// Healing
if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
    add_effect(creator, EffectType::Healing{amount: heal.heal_amount}, targets.clone());
}
```

If you `cargo run` now, your potions of healing now work.

#### Damage

We've already written the majority of what we need to handle damage, so we can just add it into `event_trigger`:

```rust
// Damage
if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
    add_effect(creator, EffectType::Damage{ amount: damage.damage }, targets.clone());
}
```

Since we've already covered area of effect and similar via targeting, and the damage code comes from the melee revamp - this will make magic missile, fireball and similar work.

#### Confusion

Confusion needs to be handled in a similar manner to hunger. We add an event type:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Healing { amount : i32 },
    Confusion { turns : i32 }
}
```

Mark it as affecting entities:

```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::WellFed => true,
        EffectType::Healing{..} => true,
        EffectType::Confusion{..} => true,
        _ => false
    }
}
```

Add a method to the `damage.rs` file:

```rust
pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion{turns} = &effect.effect_type {
        ecs.write_storage::<Confusion>().insert(target, Confusion{ turns: *turns }).expect("Unable to insert status");
    }
}
```

Include it in `affect_entity`:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing{..} => damage::heal_damage(ecs, effect, target),
        EffectType::Confusion{..} => damage::add_confusion(ecs, effect, target),
        _ => {}
    }
}
```

And lastly, support it in `event_trigger`:

```rust
// Confusion
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(entity) {
        add_effect(creator, EffectType::Confusion{ turns : confusion.turns }, targets.clone());
    }
```

That's enough to get confusion effects working.

### Triggers

Now that we've got a working system for items (it's really flexible; you can mix and match tags as you want and all the effects fire), we need to do the same for triggers. We'll start by giving them an entry point into the effects API, just like we did for items. In `effects/mod.rs` we'll further extend the item effects enum:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Healing { amount : i32 },
    Confusion { turns : i32 },
    TriggerFire { trigger: Entity }
}
```

We'll also special-case its activation:

```rust
fn target_applicator(ecs : &mut World, effect : &EffectSpawner) {
    if let EffectType::ItemUse{item} = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
    } else if let EffectType::TriggerFire{trigger} = effect.effect_type {
        triggers::trigger(effect.creator, trigger, &effect.targets, ecs);
    } else {
        match &effect.targets {
            Targets::Tile{tile_idx} => affect_tile(ecs, effect, *tile_idx),
            Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
            Targets::Single{target} => affect_entity(ecs, effect, *target),
            Targets::TargetList{targets} => targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity)),
        }
    }
}
```

Now in `effects/triggers.rs` we need to add `trigger` as a public function:

```rust
pub fn trigger(creator : Option<Entity>, trigger: Entity, targets : &Targets, ecs: &mut World) {
    // The triggering item is no longer hidden
    ecs.write_storage::<Hidden>().remove(trigger);

    // Use the item via the generic system
    event_trigger(creator, trigger, targets, ecs);

    // If it was a single activation, then it gets deleted
    if ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Delete Failed");
    }
}
```

Now that we have a framework in place, we can get into `trigger_system.rs`. Just like the item effects, it can be simplified greatly; we really just need to check that an activation happened - and call the events system:

```rust
extern crate specs;
use specs::prelude::*;
use super::{EntityMoved, Position, EntryTrigger, Map, Name, gamelog::GameLog,
    effects::*, AreaOfEffect};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        ReadStorage<'a, Name>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, AreaOfEffect>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, 
            names, entities, mut log, area_of_effect) = data;

        // Iterate the entities that moved and their final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id { // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                log.entries.insert(0, format!("{} triggers!", &name.name));
                            }

                            // Call the effects system
                            add_effect(
                                Some(entity),
                                EffectType::TriggerFire{ trigger : *entity_id },
                                if let Some(aoe) = area_of_effect.get(*entity_id) {
                                    Targets::Tiles{
                                        tiles : aoe_tiles(&*map, rltk::Point::new(pos.x, pos.y), aoe.radius)
                                    }
                                } else {
                                    Targets::Tile{ tile_idx: idx as i32 }
                                }
                            );
                        }
                    }
                }
            }
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
```

There's only one trigger we haven't already implemented as an effect: teleportation. Let's add that as an effect type in `effects/mod.rs`:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Healing { amount : i32 },
    Confusion { turns : i32 },
    TriggerFire { trigger: Entity },
    TeleportTo { x:i32, y:i32, depth: i32, player_only : bool }
}
```

It affects entities, so we'll mark that fact:

```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::WellFed => true,
        EffectType::Healing{..} => true,
        EffectType::Confusion{..} => true,
        EffectType::TeleportTo{..} => true,
        _ => false
    }
}
```

And `affect_entity` should call it:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing{..} => damage::heal_damage(ecs, effect, target),
        EffectType::Confusion{..} => damage::add_confusion(ecs, effect, target),
        EffectType::TeleportTo{..} => movement::apply_teleport(ecs, effect, target),
        _ => {}
    }
}
```

We also need to add it to `event_trigger` in `effects/triggers.rs`:

```rust
// Teleport
if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
    add_effect(
        creator, 
        EffectType::TeleportTo{ 
            x : teleport.x, 
            y : teleport.y, 
            depth: teleport.depth, 
            player_only: teleport.player_only 
        }, 
        targets.clone()
    );
}
```

Finally, we'll implement it. Make a new file, `effects/movement.rs` and paste the following into it:

```rust
use specs::prelude::*;
use super::*;
use crate::components::{ApplyTeleport};

pub fn apply_teleport(ecs: &mut World, destination: &EffectSpawner, target: Entity) {
    let player_entity = ecs.fetch::<Entity>();
    if let EffectType::TeleportTo{x, y, depth, player_only} = &destination.effect_type {
        if !player_only || target == *player_entity {
            let mut apply_teleport = ecs.write_storage::<ApplyTeleport>();
            apply_teleport.insert(target, ApplyTeleport{
                dest_x : *x,
                dest_y : *y,
                dest_depth : *depth
            }).expect("Unable to insert");
        }
    }
}
```

Now `cargo run` the project, and go forth and try some triggers. Town portal and traps being the obvious ones. You should be able to use portals and suffer trap damage, just as before.

### Limiting single use to when it *did something*

You may have noticed that we're taking your Town Portal scroll away, even if it didn't activate. We're taking away a teleporter even if it didn't actually fire (because it's player only). That needs fixing! We'll modify `event_trigger` to return `bool` - `true` if it did something, `false` if it didn't. Here's a version that does just that:

```rust
fn event_trigger(creator : Option<Entity>, entity: Entity, targets : &Targets, ecs: &mut World) -> bool {
    let mut did_something = false;
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Providing food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog.entries.insert(0, format!("You eat the {}.", names.get(entity).unwrap().name));
        did_something = true;
    }

    // Magic mapper
    if ecs.read_storage::<MagicMapper>().get(entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        gamelog.entries.insert(0, "The map is revealed to you!".to_string());
        *runstate = RunState::MagicMapReveal{ row : 0};
        did_something = true;
    }

    // Town Portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog.entries.insert(0, "You are already in town, so the scroll does nothing.".to_string());
        } else {
            gamelog.entries.insert(0, "You are telported back to town!".to_string());
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
            did_something = true;
        }
    }

    // Healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(creator, EffectType::Healing{amount: heal.heal_amount}, targets.clone());
        did_something = true;
    }

    // Damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(creator, EffectType::Damage{ amount: damage.damage }, targets.clone());
        did_something = true;
    }

    // Confusion
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(entity) {
        add_effect(creator, EffectType::Confusion{ turns : confusion.turns }, targets.clone());
        did_something = true;
    }

    // Teleport
    if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
        add_effect(
            creator, 
            EffectType::TeleportTo{ 
                x : teleport.x, 
                y : teleport.y, 
                depth: teleport.depth, 
                player_only: teleport.player_only 
            }, 
            targets.clone()
        );
        did_something = true;
    }

    did_something
}
```

Now we need to modify our entry-points to only delete an item that was actually used:

```rust
pub fn item_trigger(creator : Option<Entity>, item: Entity, targets : &Targets, ecs: &mut World) {
    // Use the item via the generic system
    let did_something = event_trigger(creator, item, targets, ecs);

    // If it was a consumable, then it gets deleted
    if did_something && ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}

pub fn trigger(creator : Option<Entity>, trigger: Entity, targets : &Targets, ecs: &mut World) {
    // The triggering item is no longer hidden
    ecs.write_storage::<Hidden>().remove(trigger);

    // Use the item via the generic system
    let did_something = event_trigger(creator, trigger, targets, ecs);

    // If it was a single activation, then it gets deleted
    if did_something && ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Delete Failed");
    }
}
```

## Cleaning Up

Now that we've got this system in place, we can clean up all manner of other systems. The first thing we can do is delete the `SufferDamage` component from `components.rs` (and remove it from `main.rs` and `saveload_system.rs`). Removing this causes the compiler to find a few places we're inflicting damage without using the effects system!

In `hunger_system.rs`, we can replace the `SufferDamage` code with:

```rust
HungerState::Starving => {
    // Inflict damage from hunger
    if entity == *player_entity {
        log.entries.insert(0, "Your hunger pangs are getting painful! You suffer 1 hp damage.".to_string());
    }
    add_effect(
        None,
        EffectType::Damage{ amount: 1},
        Targets::Single{ target: entity }
    );
}
```

We can also open `damage_system.rs` and remove the actual `DamageSystem` (but keep `delete_the_dead`). We also need to remove it from `run_systems` in `main.rs`.

## Common spawning code

In `raws/rawmaster.rs`, we're still parsing the possible effects of items repeatedly. Unfortunately, passing `EntityBuilder` objects (the `eb`) around causes some lifetime issues that make the Rust compiler reject what looks like perfectly valid code. So we'll work around that with a *macro*. Before `spawn_named_item`:

```rust
macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
        let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => $eb = $eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }),
                "ranged" => $eb = $eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }),
                "damage" => $eb = $eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }),
                "area_of_effect" => $eb = $eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }),
                "confusion" => $eb = $eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() }),
                "magic_mapping" => $eb = $eb.with(MagicMapper{}),
                "town_portal" => $eb = $eb.with(TownPortal{}),
                "food" => $eb = $eb.with(ProvidesFood{}),
                "single_activation" => $eb = $eb.with(SingleActivation{}),
                _ => println!("Warning: consumable effect {} not implemented.", effect_name)
            }
        }
    };
}
```

So this is just like a function, but it follows the rather convoluted macro syntax. Basically, we define the macro to expect `effects` and `eb` as *expressions* - that is, we don't really care what they are, we'll do text-substitution (before compiling) to insert them into the emitted code. (Macros are basically copy/pasted into your code at the call site, but with the expressions substituted). Digging down into `spawn_named_item`, you'll see that in the consumables section we are using this code. We can now replace it with:

```rust
if let Some(consumable) = &item_template.consumable {
    eb = eb.with(crate::components::Consumable{});
    apply_effects!(consumable.effects, eb);
}
```

If we go down to `spawn_named_prop`, you'll see we're doing basically the same thing:

```rust
for effect in entry_trigger.effects.iter() {
    match effect.0.as_str() {
        "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
        "single_activation" => { eb = eb.with(SingleActivation{}) }
        _ => {}
    }
}
```

We can now replace that with another call to the macro:

```rust
if let Some(entry_trigger) = &prop_template.entry_trigger {
    eb = eb.with(EntryTrigger{});
    apply_effects!(entry_trigger.effects, eb);
}
```

We'll undoubtedly add more later - for weapons "proccing", spells firing, and items that aren't consumed on use. Making this change has meant that the same definition JSON works for both entry triggers and for consumable effects - so any effect that can work with one can work with the other.

### Some examples of how this helps

Let's add a new prop to the temple: an altar that heals you. Open up `map_builders/town.rs` and find the `build_temple` function. Add an `Altar` to the list of props:

```rust
fn build_temple(&mut self,
    building: &(i32, i32, i32, i32),
    build_data : &mut BuilderMap,
    rng: &mut rltk::RandomNumberGenerator)
{
    // Place items
    let mut to_place : Vec<&str> = vec!["Priest", "Altar", "Parishioner", "Parishioner", "Chair", "Chair", "Candle", "Candle"];
    self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
}
```

Now in `spawns.json`, we add the `Altar` to the props list:

```json
{
    "name" : "Altar",
    "renderable": {
        "glyph" : "╫",
        "fg" : "#55FF55",
        "bg" : "#000000",
        "order" : 2
    },
    "hidden" : false,
    "entry_trigger" : {
        "effects" : {
            "provides_healing" : "100"
        }
    }
},
```

You can `cargo run` the project now, lose some hit points and go to the temple for a free heal. We implemented it with no additional code, because we're sharing the effect properties from other items. From now on, as we add effects - we can implement them anywhere quite readily.

## Restoring visual effects to Magic Missile and Fireball

A side-effect of our refactor is that you no longer get a fiery effect when you cast fireball (just damage indicators). You also don't get a pretty line when you zap with magic missile, or a marker when you confuse someone. This is deliberate - the previous area-of-effect code showed a fireball effect for *any* AoE attack! We can make a more flexible system by supporting effects as part of the item definition.

Let's start by decorating the two scrolls in `spawns.json` with what we want them to do:

```json
{
    "name" : "Magic Missile Scroll",
    "renderable": {
        "glyph" : ")",
        "fg" : "#00FFFF",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : { 
            "ranged" : "6",
            "damage" : "20",
            "particle_line" : "*;#00FFFF;200.0"
        }
    },
    "weight_lbs" : 0.5,
    "base_value" : 50.0,
    "vendor_category" : "alchemy",
    "magic" : { "class" : "common", "naming" : "scroll" }
},

{
    "name" : "Fireball Scroll",
    "renderable": {
        "glyph" : ")",
        "fg" : "#FFA500",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : { 
            "ranged" : "6",
            "damage" : "20",
            "area_of_effect" : "3",
            "particle" : "*;#FFA500;200.0"
        }
    },
    "weight_lbs" : 0.5,
    "base_value" : 100.0,
    "vendor_category" : "alchemy",
    "magic" : { "class" : "common", "naming" : "scroll" }
},
```

We've added two new entries - `particle` and `particle_line`. They both take a rather cryptic string (because we're passing parameters as strings). It's a semi-colon delimited list. The first parameter is the glyph, the second the color in RGB format, and the last the lifetime.

Now we need a couple of new components (in `components.rs`, and registered in `main.rs` and `saveload_system.rs`) to store this information:

```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph : u8,
    pub color : RGB,
    pub lifetime_ms : f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph : u8,
    pub color : RGB,
    pub lifetime_ms : f32
}
```

Now in `raws/rawmaster.rs` we need to parse this as an effect and attach the new components:

```rust
fn parse_particle_line(n : &str) -> SpawnParticleLine {
    let tokens : Vec<_> = n.split(';').collect();
    SpawnParticleLine{
        glyph : rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color : rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms : tokens[2].parse::<f32>().unwrap()
    }
}

fn parse_particle(n : &str) -> SpawnParticleBurst {
    let tokens : Vec<_> = n.split(';').collect();
    SpawnParticleBurst{
        glyph : rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color : rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms : tokens[2].parse::<f32>().unwrap()
    }
}

macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
        let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => $eb = $eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }),
                "ranged" => $eb = $eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }),
                "damage" => $eb = $eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }),
                "area_of_effect" => $eb = $eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }),
                "confusion" => $eb = $eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() }),
                "magic_mapping" => $eb = $eb.with(MagicMapper{}),
                "town_portal" => $eb = $eb.with(TownPortal{}),
                "food" => $eb = $eb.with(ProvidesFood{}),
                "single_activation" => $eb = $eb.with(SingleActivation{}),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                _ => println!("Warning: consumable effect {} not implemented.", effect_name)
            }
        }
    };
}
```

Implementing the particle burst is as simple as going into `effects/triggers.rs` and adding the following at the beginning of the `event_trigger` function (so it fires before damage, making the damage indicators still appear):

```rust
fn event_trigger(creator : Option<Entity>, entity: Entity, targets : &Targets, ecs: &mut World) -> bool {
    let mut did_something = false;
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Simple particle spawn
    if let Some(part) = ecs.read_storage::<SpawnParticleBurst>().get(entity) {
        add_effect(
            creator, 
            EffectType::Particle{
                glyph : part.glyph,
                fg : part.color,
                bg : rltk::RGB::named(rltk::BLACK),
                lifespan : part.lifetime_ms
            }, 
            targets.clone()
        );
    }
    ...
```

Line particle spawns are more difficult, but not too bad. One issue is that we don't actually know where the item is! We'll rectify that; in `effects/targeting.rs` we add a new function:

```rust
pub fn find_item_position(ecs: &World, target: Entity) -> Option<i32> {
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();

    // Easy - it has a position
    if let Some(pos) = positions.get(target) {
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }

    // Maybe it is carried?
    if let Some(carried) = ecs.read_storage::<InBackpack>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // Maybe it is equipped?
    if let Some(equipped) = ecs.read_storage::<Equipped>().get(target) {
        if let Some(pos) = positions.get(equipped.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // No idea - give up
    None
}
```

This function first checks to see if the item has a position (because it's on the ground). If it does, it returns it. Then it looks to see if it is in a backpack; if it is, it tries to return the position of the backpack owner. Repeat for equipped items. If it still doesn't know, it returns `None`.

We can add the following into our `event_trigger` function to handle line spawning for each targeting case:

```rust
// Line particle spawn
if let Some(part) = ecs.read_storage::<SpawnParticleLine>().get(entity) {
    if let Some(start_pos) = targeting::find_item_position(ecs, entity) {
        match targets {
            Targets::Tile{tile_idx} => spawn_line_particles(ecs, start_pos, *tile_idx, part),
            Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| spawn_line_particles(ecs, start_pos, *tile_idx, part)),
            Targets::Single{ target } => {
                if let Some(end_pos) = entity_position(ecs, *target) {
                    spawn_line_particles(ecs, start_pos, end_pos, part);
                }
            }
            Targets::TargetList{ targets } => {
                targets.iter().for_each(|target| {
                    if let Some(end_pos) = entity_position(ecs, *target) {
                        spawn_line_particles(ecs, start_pos, end_pos, part);
                    }
                });
            }
        }
    }
}
```

Each case calls `spawn_line_particles`, so lets write that too:

```rust
fn spawn_line_particles(ecs:&World, start: i32, end: i32, part: &SpawnParticleLine) {
    let map = ecs.fetch::<Map>();
    let start_pt = rltk::Point::new(start % map.width, end / map.width);
    let end_pt = rltk::Point::new(end % map.width, end / map.width);
    let line = rltk::line2d(rltk::LineAlg::Bresenham, start_pt, end_pt);
    for pt in line.iter() {
        add_effect(
            None, 
            EffectType::Particle{
                glyph : part.glyph,
                fg : part.color,
                bg : rltk::RGB::named(rltk::BLACK),
                lifespan : part.lifetime_ms
            }, 
            Targets::Tile{ tile_idx : map.xy_idx(pt.x, pt.y) as i32}
        );
    }
}
```

This is quite simple: it plots a line between start and end, and places a particle on each tile.

You can now `cargo run` and enjoy the effects of fireball and magic missile.

## Wrap-Up

This has been a big chapter of changes that don't do a lot on the surface. We've gained a lot, however:

* The Inventory System is now easy to follow.
* The generic effects system can now apply *any* effect to an item or trigger, and can be readily extended with new items without running into `Specs` limitations.
* There's a lot less distribution of responsibility: systems no longer need to remember to show a particle for damage, or even need to know about how particles work - they just request them. Systems can often not worry about position, and apply positional effects (including AoE) in a consistent manner.
* We've now got a flexible enough system to let us build big, cohesive effects - without worrying too much about the details.

This chapter has been a good example of the limitations of an ECS - and how to use that to your advantage. By using components as flags, we can easily *compose* effects - a potion that heals you and confuses you is as simple as combining two tags. However, `Specs` doesn't really play well with systems that read a ton of data storages at once - so we worked around it by adding messaging on top of the system. This is pretty common: even Amethyst, the ECS-based engine, also implements a message-passing system for this purpose.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-63-effects)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-63-effects)
---

Copyright (C) 2019, Herbert Wolverson.

---