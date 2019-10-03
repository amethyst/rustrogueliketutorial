# Map Construction Test Harness

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

As we're diving into generating new and interesting maps, it would be helpful to provide a way to *see* what the algorithms are doing. This chapter will build a test harness to accomplish this, and extend the `SimpleMapBuilder` from the previous chapter to support it. This is going to be a relatively large task, and we'll learn some new techniques along the way!

## Cleaning up map creation - Do Not Repeat Yourself

In `main.rs`, we essentially have the same code three times. When the program starts, we insert a map into the world. When we change level, or finish the game - we do the same. The last two have different semantics (since we're updating the world rather than inserting for the first time) - but it's basically redundant repetition.

We'll start by changing the first one to insert *placeholder* values rather than the actual values we intend to use. This way, the `World` has the slots for the data - it just isn't all that useful yet. Here's a version with the old code commented out:

```rust
gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

gs.ecs.insert(Map::new(1));
gs.ecs.insert(Point::new(0, 0));
gs.ecs.insert(rltk::RandomNumberGenerator::new());

/*let mut builder = map_builders::random_builder(1);
builder.build_map();
let player_start = builder.get_starting_position();
let map = builder.get_map();
let (player_x, player_y) = (player_start.x, player_start.y);
builder.spawn_entities(&mut gs.ecs);
gs.ecs.insert(map);
gs.ecs.insert(Point::new(player_x, player_y));*/

let player_entity = spawner::player(&mut gs.ecs, 0, 0);
gs.ecs.insert(player_entity);
```

So instead of building the map, we put a placeholder into the `World` resources. That's obviously not very useful for actually starting the game, so we also need a function to do the actual building and update the resources. Not entirely coincidentally, that function is the same as the other two places from which we currently update the map! In other words, we can roll those into this function, too. So in the implementation of `State`, we add:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    let mut builder = map_builders::random_builder(new_depth);
    builder.build_map();
    let player_start;
    {
        let mut worldmap_resource = self.ecs.write_resource::<Map>();
        *worldmap_resource = builder.get_map();
        player_start = builder.get_starting_position();
    }

    // Spawn bad guys
    builder.spawn_entities(&mut self.ecs);

    // Place the player and update resources
    let (player_x, player_y) = (player_start.x, player_start.y);
    let mut player_position = self.ecs.write_resource::<Point>();
    *player_position = Point::new(player_x, player_y);
    let mut position_components = self.ecs.write_storage::<Position>();
    let player_entity = self.ecs.fetch::<Entity>();
    let player_pos_comp = position_components.get_mut(*player_entity);
    if let Some(player_pos_comp) = player_pos_comp {
        player_pos_comp.x = player_x;
        player_pos_comp.y = player_y;
    }

    // Mark the player's visibility as dirty
    let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
    let vs = viewshed_components.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    } 
}
```

Now we can get rid of the commented out code, and simplify our first call quite a bit:

```rust
gs.ecs.insert(Map::new(1));
gs.ecs.insert(Point::new(0, 0));
gs.ecs.insert(rltk::RandomNumberGenerator::new());
let player_entity = spawner::player(&mut gs.ecs, 0, 0);
gs.ecs.insert(player_entity);
gs.ecs.insert(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });
gs.ecs.insert(particle_system::ParticleBuilder::new());
gs.ecs.insert(rex_assets::RexAssets::new());

gs.generate_world_map(1);
```

We can also go to the various parts of the code that call the same code we just added to `generate_world_map` and greatly simplify them by using the new function. We can replace `goto_next_level` with:

```rust
fn goto_next_level(&mut self) {
    // Delete entities that aren't the player or his/her equipment
    let to_delete = self.entities_to_remove_on_level_change();
    for target in to_delete {
        self.ecs.delete_entity(target).expect("Unable to delete entity");
    }

    // Build a new map and place the player
    let current_depth;
    {
        let worldmap_resource = self.ecs.fetch::<Map>();
        current_depth = worldmap_resource.depth;
    }
    self.generate_world_map(current_depth + 1);

    // Notify the player and give them some health
    let player_entity = self.ecs.fetch::<Entity>();
    let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
    gamelog.entries.insert(0, "You descend to the next level, and take a moment to heal.".to_string());
    let mut player_health_store = self.ecs.write_storage::<CombatStats>();
    let player_health = player_health_store.get_mut(*player_entity);
    if let Some(player_health) = player_health {
        player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
    }
}
```

Likewise, we can clean up `game_over_cleanup`:

```rust
fn game_over_cleanup(&mut self) {
    // Delete everything
    let mut to_delete = Vec::new();
    for e in self.ecs.entities().join() {
        to_delete.push(e);
    }
    for del in to_delete.iter() {
        self.ecs.delete_entity(*del).expect("Deletion failed");
    }

    // Spawn a new player
    {
        let player_entity = spawner::player(&mut self.ecs, 0, 0);
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player_entity;
    }

    // Build a new map and place the player
    self.generate_world_map(1);                                          
}
```

And there we go - `cargo run` gives the same game we've had for a while, and we've cut out a bunch of code. Refactors that make things smaller rock!

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-24-map-testing)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-24-map-testing/)
---

Copyright (C) 2019, Herbert Wolverson.

---