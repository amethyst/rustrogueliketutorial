# AI Cleanup and Status Effects

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the design document, we indicated that we'd like the AI to be smarter than the average rock. We've also added quite a few AI-related systems as we've worked through the chapters and (you may have noticed) some things like consistently applying *confusion* effects (and occasional problems hitting a mob that just moved) have slipped through the cracks!

As we add complexity to our monsters, it would be good to get all of this straightened out, make it easier to support new features, and handle commonalities like movement consistently.

## Chained Systems

Rather than try and do *everything* for an NPC in one system, we could break the process out into several steps. This is more typing, but has the advantage that each step is distinct, clear and does just one thing - making it *much* easier to debug. This is analogous to how we are handling `WantsToMelee` - we're indicating an intent and then handling it in its own step - which let us keep targeting and actually fighting separated.

Let's look at the steps, and see how they can be broken down:

* We determine that it is the NPC's turn.
* We check status effects - such as *Confusion* to determine if they can in fact go.
* The AI module for that AI type scans their surroundings, and determines if they want to move, attack or do nothing.
* Movement occurs, which updates various global statuses.
* Combat occurs, which can kill mobs or render them unable to act in the future.

## Modularizing the AI

We already have quite a few AI systems, and this is just going to add more. So let's move AI into a *module*. Create a new folder, `src/ai` - this will be the new AI module. Create a `mod.rs` file, and put the following into it:

```rust
mod animal_ai_system;
mod bystander_ai_system;
mod monster_ai_system;
pub use animal_ai_system::AnimalAI;
pub use bystander_ai_system::BystanderAI;
pub use monster_ai_system::MonsterAI;
```

This tells it to use the other AI modules and share them all in the `ai` namespace. Now move `animal_ai_system`, `bystander_ai_system` and `monster_ai_system` from your `src` directory into `src\ai`. In the preamble to `main.rs` (where you have all the `mod` and `use` statements), remove the `mod` and `use` statements for these systems. Replace them with a single `mod ai;` line. Finally, you can cleanup `run_systems` to reference these systems via the `ai` namespace:

```rust
impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = ai::MonsterAI{};
        mob.run_now(&self.ecs);
        let mut animal = ai::AnimalAI{};
        animal.run_now(&self.ecs);
        let mut bystander = ai::BystanderAI{};
        bystander.run_now(&self.ecs);
...
```

In your `ai/X_system` files you have lines that read `use super::{...}`. Replace the `super` with `crate`, to indicate that you want to use the components (and other types) from the parent crate.

If you `cargo run` now, you have exactly the same game as before - your refactor worked!

## Determining Whose Turn It Is - Initiative/Energy Cost

So far, we've handles our turns in a strict but inflexible manner: the player goes, and then *all* the NPCs go. Back and forth, forever. This works pretty well, but it doesn't allow for much variety: you can't have something make an entity faster than the others, all actions take the same amount of time, and it would make things like *haste* and *slow* spells impossible to implement.

*Many* roguelike games use a variant of initiative or initiative cost to determine whose turn it is, so we'll go with something similar. We don't want to be *too* random, so you don't suddenly see things speed up and slow down, but we also want to be more flexible. We also want it to be a *little* random, so that all the NPCs don't act at the same time by default - giving basically what we have already. It would also be good to slow down wearers of heavy armor/weapons, and have users of light equipment go faster (dagger users can strike more frequently than claymore users!).

In `components.rs` (and registered in `main.rs` and `saveload_system.rs`), lets make a new `Initiative` component:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Initiative {
    pub current : i32
}
```

We want the player to start with an initiative score (we'll go with 0, so they always start first). In `spawners.rs`, we simply add it to the `player` function as another component for the player:

```rust
.with(Initiative{current: 0})
```

We also want all NPCs to start with an initiative score. So in `raws/rawmaster.rs`, we add it to the `spawn_named_mob` function as another always-present component. We'll give mobs a starting initiative of 2 - so on the first turn they will all process just after the player (we'll worry about subsequent turns later).

```rust
// Initiative of 2
eb = eb.with(Initiative{current: 2});
```

That adds the component, but currently it doesn't *do* anything at all. We're going to start by making another new component in `components.rs` (and registering it in `main.rs` and `saveload_system.rs`), called `MyTurn`:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MyTurn {}
```

The idea behind `MyTurn` components is that if you have the component, then it is your turn to act - and you should be included in AI/turn control (if the player has `MyTurn` then we wait for input). If you *don't* have it, then you don't get to act. We can also use it as a filter: so things like status effects can check to see if it is your turn and you are affected by a status, and they might determine that you have to skip your turn.

Now we should make a new - simple - system for handling initiative rolls. Make a new file, `ai/initiative_system.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use crate::{Initiative, Position, MyTurn, Attributes, RunState};

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteStorage<'a, Initiative>,
                        ReadStorage<'a, Position>,
                        WriteStorage<'a, MyTurn>,
                        Entities<'a>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>,
                        ReadStorage<'a, Attributes>,
                        WriteExpect<'a, RunState>,
                        ReadExpect<'a, Entity>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut initiatives, positions, mut turns, entities, mut rng, attributes, 
            mut runstate, player) = data;

        if *runstate != RunState::Ticking { return; } // We'll be adding Ticking in a moment; use MonsterTurn if you want to test in the meantime

        // Clear any remaining MyTurn we left by mistkae
        turns.clear();

        // Roll initiative
        for (entity, initiative, _pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                // It's my turn!
                turns.insert(entity, MyTurn{}).expect("Unable to insert turn");

                // Re-roll
                initiative.current = 6 + rng.roll_dice(1, 6);

                // Give a bonus for quickness
                if let Some(attr) = attributes.get(entity) {
                    initiative.current -= attr.quickness.bonus;
                }

                // TODO: More initiative granting boosts/penalties will go here later

                // If its the player, we want to go to an AwaitingInput state
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                }
            }
        }
    }
}
```

This is pretty simple:

1. We first clear out any remaining `MyTurn` components, in case we forgot to delete one (so entities don't zoom around).
2. We iterate all entities that have an `Initiative` component (indicating they can go at all) and a `Position` component (which we don't use, but indicates they are on the current map layer and can act).
3. We subtract one from the entity's current initiative.
4. If the current initiative is 0 (or less, in case we messed up!), we apply a `MyTurn` component to them. Then we re-roll their current initiative; we're going with `6 + 1d6 + Quickness Bonus` for now. Notice how we've left a comment indicating that we're going to make this more complicated later!
5. If it is now the player's turn, we change the global `RunState` to `AwaitingInput` - it's time to process the player's instructions.

We're also checking if it is the monster's turn; we'll actually be changing that - but I didn't want the system spinning rolling initiative over and over again if we test it!

Now we need to go into `mod.rs` and add `mod initiative_system.rs; pub use initiative_system::InitiativeSystem;` pair of lines to expose it to the rest of the program. Then we open up `main.rs` and add it to `run_systems`:

```rust
impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut initiative = ai::InitiativeSystem{};
        initiative.run_now(&self.ecs);
        ...
```

We've added it before the various AI functions run, but after we obtain a map index and visibility - so they have up-to-date data to work with.

## Adjusting the game loop to use initiative

Open up `main.rs` and we'll edit `RunState` to get rid of the `PlayerTurn` and `MonsterTurn` entries - replacing them instead with `Ticking`. This is going to break a *lot* of code - but that's ok, we're actually simplifying AND gaining functionality, which is a win by most standards! Here's the new `RunState`:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput, 
    PreRun, 
    Ticking, 
    ShowInventory, 
    ShowDropItem, 
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration,
    ShowCheatMenu
}
```

In our main loop's `match` function, we can delete the `MonsterTurn` entry completely and adjust `PlayerTurn` to be a more generic `Ticking` state:

```rust
RunState::Ticking => {
    self.run_systems();
    self.ecs.maintain();
    match *self.ecs.fetch::<RunState>() {
        RunState::AwaitingInput => newrunstate = RunState::AwaitingInput,
        RunState::MagicMapReveal{ .. } => newrunstate = RunState::MagicMapReveal{ row: 0 },
        _ => newrunstate = RunState::Ticking
    }                
}
```

You'll also want to search `main.rs` for `PlayerTurn` and `MonsterTurn`; a lot of states return to one of these when they are done. They now want to return to `Ticking`.

Likewise, in `player.rs` there's a lot of places we return `RunState::PlayerTurn` - you'll want to change all of these to `Ticking`.

We'll modify the hunger clock to only tick on your turn. This actually becomes more simple; we simply join on `MyTurn` and can remove the entire "proceed" system:

```rust
use specs::prelude::*;
use super::{HungerClock, RunState, HungerState, SufferDamage, gamelog::GameLog, MyTurn};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        WriteStorage<'a, HungerClock>,
                        ReadExpect<'a, Entity>, // The player
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, SufferDamage>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, MyTurn>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage, mut log,
            turns) = data;

        for (entity, mut clock, _myturn) in (&entities, &mut hunger_clock, &turns).join() {
            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.insert(0, "You are no longer well fed.".to_string());
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.insert(0, "You are hungry.".to_string());
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.insert(0, "You are starving!".to_string());
                        }
                    }
                    HungerState::Starving => {
                        // Inflict damage from hunger
                        if entity == *player_entity {
                            log.entries.insert(0, "Your hunger pangs are getting painful! You suffer 1 hp damage.".to_string());
                        }
                        inflict_damage.insert(entity, SufferDamage{ amount: 1, from_player: false }).expect("Unable to do damage");  
                    }
                }
            }
        }
    }
}
```

That leaves the files in `ai` with errors. We'll make the bare minimum of changes to make these run for now. Delete the lines that check the game state, and add a read storage for `MyTurn`. Add the turn to the join, so the entity only acts if it is their turn. So in `ai/animal_ai_system.rs`:

```rust
impl<'a> System<'a> for AnimalAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Herbivore>,
                        ReadStorage<'a, Carnivore>,
                        ReadStorage<'a, Item>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, EntityMoved>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, MyTurn> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_entity, runstate, entities, mut viewshed, 
            herbivore, carnivore, item, mut wants_to_melee, mut entity_moved, 
            mut position, turns) = data;
    ...
    for (entity, mut viewshed, _herbivore, mut pos, _turn) in (&entities, &mut viewshed, &herbivore, &mut position, &turns).join() {
    ...
    for (entity, mut viewshed, _carnivore, mut pos, _turn) in (&entities, &mut viewshed, &carnivore, &mut position, &turns).join() {
```

Likewise, in `bystander_ai_system.rs`:

```rust
impl<'a> System<'a> for BystanderAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Bystander>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, EntityMoved>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>,
                        ReadExpect<'a, Point>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, Quips>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, MyTurn>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, runstate, entities, mut viewshed, bystander, mut position,
            mut entity_moved, mut rng, player_pos, mut gamelog, mut quips, names, turns) = data;

        for (entity, mut viewshed,_bystander,mut pos, _turn) in (&entities, &mut viewshed, &bystander, &mut position, &turns).join() {
        ...
```

And again in `monster_ai_system.rs`:

```rust
impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, Confusion>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, MyTurn>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, 
            monster, mut position, mut wants_to_melee, mut confused, mut particle_builder,
            mut entity_moved, turns) = data;

        for (entity, mut viewshed,_monster,mut pos, _turn) in (&entities, &mut viewshed, &monster, &mut position, &turns).join() {
```

That takes care of the compilation errors! Now `cargo run` the game. It runs as before, just a little more slowly. We'll worry about performance once we have the basics going - so that's great progress, we have an initiative system!

## Handling Status Effects

Right now, we check for *confusion* in the `monster_ai_system` - and actually forgot about it in bystanders, vendors and animals. Rather than copy/pasting the code everywhere, we should use this as an opportunity to create a system to handle status effect turn skipping, and clean up the other systems to benefit. Make a new file, `ai/turn_status.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use crate::{MyTurn, Confusion, RunState};

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteStorage<'a, MyTurn>,
                        WriteStorage<'a, Confusion>,
                        Entities<'a>,
                        ReadExpect<'a, RunState>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut turns, mut confusion, entities, runstate) = data;

        if *runstate != RunState::Ticking { return; }

        let mut not_my_turn : Vec<Entity> = Vec::new();
        let mut not_confused : Vec<Entity> = Vec::new();
        for (entity, _turn, confused) in (&entities, &mut turns, &mut confusion).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
            } else {
                not_my_turn.push(entity);
            }
        }

        for e in not_my_turn {
            turns.remove(e);
        }

        for e in not_confused {
            confusion.remove(e);
        }
    }
}
```

This is pretty simple: it iterates everyone who is confused, and decrements their turn counter. If they are still confused, it takes away `MyTurn`. If they have recovered, it takes away `Confusion`. You need to add a `mod` and `pub use` statement for it in `ai/mod.rs`, and add it to your `run_systems` function in `main.rs`:

```rust
let mut initiative = ai::InitiativeSystem{};
initiative.run_now(&self.ecs);
let mut turnstatus = ai::TurnStatusSystem{};
turnstatus.run_now(&self.ecs);
```

This shows the new pattern we are using: systems do one thing, and can remove `MyTurn` to prevent future execution. You can also go into `monster_ai_system` and remove everything relating to confusion.

## Quipping NPCs

Remember when we added bandits, we gave them some commentary to say for flavor? You may have noticed that they aren't actually speaking! That's because we handled quipping in the bystander AI - rather than as a general concept. Let's move the quipping into its own system. Make a new file, `ai/quipping.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use crate::{gamelog::GameLog, Quips, Name, MyTurn, Viewshed};

pub struct QuipSystem {}

impl<'a> System<'a> for QuipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, GameLog>,
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Viewshed>,
        WriteExpect<'a, rltk::RandomNumberGenerator>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut gamelog, mut quips, names, turns, player_pos, viewsheds, mut rng) = data;

        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && rng.roll_dice(1,6)==1 {
                let quip_index = 
                    if quip.available.len() == 1 { 0 } 
                    else { (rng.roll_dice(1, quip.available.len() as i32)-1) as usize };
                
                gamelog.entries.insert(0,
                    format!("{} says \"{}\"", name.name, quip.available[quip_index])
                );
                quip.available.remove(quip_index);
            }                
        }
    }
}
```

This is basically the quipping code from `bystander_ai_system`, so we don't really need to go over it in too much detail. You do want to add it into `run_systems` in `main.rs` so it functions (and add a `mod` and `pub use` statement in `ai/mod.rs`):

```rust
turnstatus.run_now(&self.ecs);
let mut quipper = ai::QuipSystem{};
quipper.run_now(&self.ecs);
```

Also go into `bystander_ai_system.rs` and remove all the quip code! It shortens it a *lot*, and if you `cargo run` now, Bandits can insult you. In fact, *any* NPC can be given quip lines now - and will merrily say things to you.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-57-ai)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-57-ai)
---

Copyright (C) 2019, Herbert Wolverson.

---