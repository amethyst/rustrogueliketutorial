# Simple Traps

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Most roguelikes, like their D&D precursors, feature traps in the dungeon. Walk down an innocent looking hallway, and *oops* - an arrow flies out and hits you. This chapter will implement some simple traps, and then examine some of the game implications they bring.

## What is a trap?

Most traps follow the pattern of: you might see the trap (or you might not!), you enter the tile anyway, the trap goes off and something happens (damage, teleport, etc.). So traps can be logically divided into three sections:

* An appearance (which we already support), which may or may not be discovered (which we don't, yet).
* A *trigger* - if you enter the trap's tile, something happens.
* An *effect* - which we've touched on with magic items.

Let's work our way through getting components into place for these, in turn.

## Rendering a basic bear trap

A lot of roguelikes use `^` for a trap, so we'll do the same. We have all the components required to render a basic object, so we'll make a new spawning function (in `spawners.rs`). It's pretty much the minimum to put a glyph on the map:

```rust
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Bear Trap".to_string() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

We'll also add it into the list of things that can spawn:

```rust
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 2)
}
```

```rust
match spawn.1.as_ref() {
    "Goblin" => goblin(ecs, x, y),
    "Orc" => orc(ecs, x, y),
    "Health Potion" => health_potion(ecs, x, y),
    "Fireball Scroll" => fireball_scroll(ecs, x, y),
    "Confusion Scroll" => confusion_scroll(ecs, x, y),
    "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
    "Dagger" => dagger(ecs, x, y),
    "Shield" => shield(ecs, x, y),
    "Longsword" => longsword(ecs, x, y),
    "Tower Shield" => tower_shield(ecs, x, y),
    "Rations" => rations(ecs, x, y),
    "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
    "Bear Trap" => bear_trap(ecs, x, y),
    _ => {}
}
```

If you `cargo run` the project now, occasionally you will run into a red `^` - and it will be labeled "Bear Trap" on the mouse-over. Not massively exciting, but a good start! Note that for testing, we'll up the spawn frequency from 2 to 100 - LOTS of traps, making debugging easier. Remember to lower it later!

## But you don't always spot the trap!

It is pretty easy if you can *always* know that a trap awaits you! So we want to make traps *hidden* by default, and come up with a way to sometimes locate traps when you are near them. Like most things in an ECS driven world, analyzing the text gives a great clue as to what components you need. In this case, we need to go into `components.rs` and create a new component - `Hidden`:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}
```

As usual, we need to register it in `main.rs` and in `saveload_system.rs`. We'll also give the property to our new bear trap:

```rust
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Bear Trap".to_string() })
        .with(Hidden{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

Now, we want to modify the object renderer to not show things that are *hidden*. The [Specs Book](https://specs.amethyst.rs/docs/tutorials/08_join.html) provides a great clue as to how to *exclude* a component from a join, so we do that (in `main.rs`):

```rust
let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
for (pos, render, _hidden) in data.iter() {
    let idx = map.xy_idx(pos.x, pos.y);
    if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
}
```

Notice that we've added a `!` ("not" symbol) to the join - we're saying that entities must *not* have the `Hidden` component if we are to render them.

If you `cargo run` the project now, the bear traps are no longer visible. However, they show up in tool tips (which may be perhaps as well, we know they are there!). We'll exclude them from tool-tips also. In `gui.rs`, we amend the `draw_tooltips` function:

```rust
fn draw_tooltips(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }
    let mut tooltip : Vec<String> = Vec::new();
    for (name, position, _hidden) in (&names, &positions, !&hidden).join() {
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }
    ...
```

Now if you `cargo run`, you'll have no idea that traps are present. Since they don't *do* anything yet - they may as well not exist!

## Adding entry triggers

A trap should *trigger* when an entity walks onto them. So in `components.rs`, we'll create an `EntryTrigger` (as usual, we'll also register it in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}
```

We'll give bear traps a trigger (in `spawner.rs`):

```rust
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Bear Trap".to_string() })
        .with(Hidden{})
        .with(EntryTrigger{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

We also need to have traps fire their trigger when an entity enters them. We'll add *another* component, `EntityMoved` to indicate that an entity has moved this turn. In `components.rs` (and remembering to register in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}
```

Now, we scour the codebase to add an `EntityMoved` component every time an entity moves. In `player.rs`, we handle player movement in the `try_move_player` function. At the top, we'll gain write access to the relevant component store:

```rust
let mut entity_moved = ecs.write_storage::<EntityMoved>();
```

Then when we've determined that the player did, in fact, move - we'll insert the `EntityMoved` component:

```rust
entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
```

The other location that features movement is the Monster AI. So in `monster_ai_system.rs`, we do something similar. We add a `WriteResource` for the `EntityMoved` component, and insert one after the monster moves. The source code for the AI is getting a bit long, so I recommend you look at the source file directly for this one ([here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-22-simpletraps)).

Lastly, we need a *system* to make triggers actually *do something*. We'll make a new file, `trigger_system.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use super::{EntityMoved, Position, EntryTrigger, Hidden, Map, Name, gamelog::GameLog};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        WriteStorage<'a, Hidden>,
                        ReadStorage<'a, Name>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, mut hidden, names, entities, mut log) = data;

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
                                log.entries.push(format!("{} triggers!", &name.name));
                            }

                            hidden.remove(*entity_id); // The trap is no longer hidden
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

This is relatively straightforward if you've been through the previous chapters:

1. We iterate all entities that have a `Position` and an `EntityMoved` component.
2. We obtain the map index for their location.
3. We iterate the `tile_content` index to see what's in the new tile.
4. We look to see if there is a trap there.
5. If there is, we get its name and notify the player (via the log) that a trap activated.
6. We remove the `hidden` component from the trap, since we now know that it is there.

We also have to go into `main.rs` and insert code to run the system. It goes after the Monster AI, since monsters can move - but we might output damage, so that system needs to run later:

```rust
...
let mut mob = MonsterAI{};
mob.run_now(&self.ecs);
let mut triggers = trigger_system::TriggerSystem{};
triggers.run_now(&self.ecs);
...
```

## Traps that hurt

So that gets us a long way: traps can be sprinkled around the level, and trigger when you enter their target tile. It would help if the trap *did something*! We actually have a decent number of component types to describe the effect. In `spawner.rs`, we'll extend the bear trap to include some damage:

```rust
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Bear Trap".to_string() })
        .with(Hidden{})
        .with(EntryTrigger{})
        .with(InflictsDamage{ damage: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

We'll also extend the `trigger_system` to apply the damage:

```rust
// If the trap is damage inflicting, do it
let damage = inflicts_damage.get(*entity_id);
if let Some(damage) = damage {
    particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
    inflict_damage.insert(entity, SufferDamage{ amount: damage.damage }).expect("Unable to do damage");
}
```

If you `cargo run` now, you can move around - and walking into a trap will damage you. If a monster walks into a trap, it damages them too! It even plays the particle effect for attacking.

## Bear traps only snap once

Some traps, like a bear trap (think a spring with spikes) really only fire once. That seems like a useful property to model for our trigger system, so we'll add a new component (to `components.rs`, `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}
```

We'll also add it to the Bear Trap function in `spawner.rs`:

```rust
.with(SingleActivation{})
```

Now we modify the `trigger_system` to apply it. Note that we remove the entities *after* looping through them, to avoid confusing our iterators.

```rust
extern crate specs;
use specs::prelude::*;
use super::{EntityMoved, Position, EntryTrigger, Hidden, Map, Name, gamelog::GameLog, 
    InflictsDamage, particle_system::ParticleBuilder, SufferDamage, SingleActivation};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        WriteStorage<'a, Hidden>,
                        ReadStorage<'a, Name>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, SingleActivation>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, mut hidden, 
            names, entities, mut log, inflicts_damage, mut particle_builder,
            mut inflict_damage, single_activation) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities : Vec<Entity> = Vec::new();
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
                                log.entries.push(format!("{} triggers!", &name.name));
                            }

                            hidden.remove(*entity_id); // The trap is no longer hidden

                            // If the trap is damage inflicting, do it
                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                                inflict_damage.insert(entity, SufferDamage{ amount: damage.damage }).expect("Unable to do damage");
                            }

                            // If it is single activation, it needs to be removed
                            let sa = single_activation.get(*entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(*entity_id);
                            }
                        }
                    }
                }
            }
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
```

If you `cargo run` now (I recommend `cargo run --release` - it's getting slower!), you can be hit by a bear trap - take some damage, and the trap goes away.

## Spotting Traps

We have a pretty functional trap system now, but it's *annoying* to randomly take damage for no apparent reason - because you had no way to know that a trap was there. It's also quite unfair, since there's no way to guard against it. We'll implement a chance to spot traps. At some point in the future, this might be tied to an attribute or skill - but for now, we'll go with a dice roll. That's a bit nicer than asking everyone to carry a 10 foot pole with them at all times (like some early D&D games!).

Since the `visibility_system` already handles *revealing* tiles, why not make it potentially reveal hidden things, too? Here's the code for `visibility_system.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Player, Hidden, gamelog::GameLog};
extern crate rltk;
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                        WriteStorage<'a, Hidden>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, 
            mut hidden, mut rng, mut log, names) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );

                // If this is the player, reveal what they can see
                let _p : Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Chance to reveal hidden things
                        for e in map.tile_content[idx].iter() {
                            let maybe_hidden = hidden.get(*e);
                            if let Some(_maybe_hidden) = maybe_hidden {
                                if rng.roll_dice(1,24)==1 {
                                    let name = names.get(*e);
                                    if let Some(name) = name {
                                        log.entries.push(format!("You spotted a {}.", &name.name));
                                    }
                                    hidden.remove(*e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

So why a 1 in 24 chance to spot traps? I played around until it felt about right. 1 in 6 (my first choice) was too good. Since your viewshed updates whenever you move, you have a *high* chance of spotting traps as you move around. Like a lot of things in game design: sometimes you just have to play with it until it feels right!

If you `cargo run` now, you can walk around - and sometimes spot traps. Monsters won't reveal traps, unless they fall into them.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-22-simpletraps)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-22-simpletraps/)
---

Copyright (C) 2019, Herbert Wolverson.

---