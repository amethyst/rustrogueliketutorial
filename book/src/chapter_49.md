# Bringing NPCs to Life

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

I'd like to suggest dark incantations and candles to breathe life into NPCs, but in reality - it's more code. We don't want our bystanders to stand around, dumb as rocks anymore. They don't have to behave particularly sensibly, but it would be good if they at least roam around a bit (other than vendors, that gets annoying - "where did the blacksmith go?") and tell you about their day.

## New components - dividing vendors from bystanders

First, we're going to make a new component - the `Vendor`. In `components.rs`, add the following component type:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Vendor {}
```

*Don't forget to register it in `main.rs` and `saveload_system.rs`!*

Now we'll adjust our raw files (`spawns.json`); all of the merchants who feature `"ai" : "bystander"` need to be changed to `"ai" : "vendor"`. So we'll change it for our Barkeep, Alchemist, Clothier, Blacksmith and Shady Salesman.

Next, we adjust our `raws/rawmaster.rs`'s function `spawn_named_mob` to also spawn vendors:

```rust
match mob_template.ai.as_ref() {
    "melee" => eb = eb.with(Monster{}),
    "bystander" => eb = eb.with(Bystander{}),
    "vendor" => eb = eb.with(Vendor{}),
    _ => {}
}
```

Finally, we'll adjust the `try_move_player` function in `player.rs` to also not attack vendors:

```rust
...
let vendors = ecs.read_storage::<Vendor>();

let mut swap_entities : Vec<(Entity, i32, i32)> = Vec::new();

for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
    let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

    for potential_target in map.tile_content[destination_idx].iter() {
        let bystander = bystanders.get(*potential_target);
        let vendor = vendors.get(*potential_target);
        if bystander.is_some() || vendor.is_some() {
...
```

## A System for Moving Bystanders

We want bystanders to wander around the town. We won't have them open doors, to keep things consistent (so when you enter the pub, you can expect patrons - and they won't have wandered off to fight rats!). Make a new file, `bystander_ai_system.rs` and paste the following into it:

```rust
use specs::prelude::*;
use super::{Viewshed, Bystander, Map, Position, RunState, EntityMoved};

pub struct BystanderAI {}

impl<'a> System<'a> for BystanderAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Bystander>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, EntityMoved>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, runstate, entities, mut viewshed, bystander, mut position,
            mut entity_moved, mut rng) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _bystander, mut pos) in (&entities, &mut viewshed, &bystander, &mut position).join() {
            // Try to move randomly
            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);
            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            let dest_idx = map.xy_idx(x, y);
            if !map.blocked[dest_idx] {
                let idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                pos.x = x;
                pos.y = y;
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                map.blocked[dest_idx] = true;
                viewshed.dirty = true;
            }
        }
    }
}
```

If you remember from the systems we've made before, the first part is boilerplate telling the ECS what resources we want to access. We check to see if it is the monster's turn (really, NPCs are monsters in this setup); if it isn't, we bail out. Then we roll a dice for a random direction, see if we can go that way - and move if we can. It's pretty simple!

In `main.rs`, we need to tell it to use the new module:

```rust
pub mod bystander_ai_system;
```

We also need to add the system to our list of systems to run:

```rust
impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut bystander = bystander_ai_system::BystanderAI{};
        bystander.run_now(&self.ecs);
        let mut triggers = trigger_system::TriggerSystem{};
        triggers.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut itemuse = ItemUseSystem{};
        itemuse.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem{};
        item_remove.run_now(&self.ecs);
        let mut hunger = hunger_system::HungerSystem{};
        hunger.run_now(&self.ecs);
        let mut particles = particle_system::ParticleSpawnSystem{};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
```

If you `cargo run` the project now, you can watch NPCs bumbling around randomly. Having them move goes a *long* way to not making it feel like a town of statues!

![Screenshot](./c49-s1.gif)

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-49-town3)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-49-town3)
---

Copyright (C) 2019, Herbert Wolverson.

---