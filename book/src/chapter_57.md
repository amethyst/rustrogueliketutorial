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



...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-57-ai)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-57-ai)
---

Copyright (C) 2019, Herbert Wolverson.

---