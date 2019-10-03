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

## Making a generator

It's surprisingly difficult to combine two paradigms, sometimes:

* The graphical "tick" nature of RLTK (and the underlying GUI environment) encourages you to do everything fast, in one fell swoop.
* Actually visualizing progress while you generate a map encourages you to run in lots of phases as a "state machine", yielding map results along the way.

Rust is getting support for *coroutine generators*, but it isn't in the stable language yet. That's a shame, because `yield` - and yielding progress - is *exactly* what they are designed for. Instead, we turn to `cargo` and find the `generator` crate. It is quite similar to the language proposal, so when it hits stable it shouldn't be *too* hard to migrate.

In `cargo.toml`, we add this to the dependencies:

```toml
generator = "0.6.18"
```

In `main.rs`, we have to tell it to import the macros:

```rust
#[macro_use]
extern crate generator;
```

And we refactor our map generators to run as coroutine generators. Here's the interface from `mod.rs`:

```rust
fn build_map(&mut self) -> Generator<(), Map>;
```

And here is the implementation from `simple_map.rs`:

```rust
fn build_map(&mut self) -> Generator<(), Map> {
    Gn::new_scoped(move |mut s| {
        println!("Running build map");
        self.rooms_and_corridors();
        done!();
    })
}
```

That's a bit messy; it does the following:

1. Create a new "scoped" generator, a closure.
2. It moves the generator state into the closure.
3. It prints out a note that we're making a map - for debugging purposes.
4. It calls `rooms_and_corridors` as before.
5. It calls the `generator` crate's `done!` macro to finish up.

If you were to run the project now, it would crash with no map generated. That's because the `generator` system actually returns an `iterator` - a range of yielded values. We're not actually yielding anything yet, but we need to visit `main.rs` and edit the `generate_world_map` function we just made (see? It was useful in reducing typing! Now we don't have to change it in three places). The `build_map` function call becomes:

```rust
for _i in builder.build_map() {};
```

This tells the program to go through each value of the iterator - which forces it to run. If you run the project now, you'll see "Running build map" on the console and the game plays as before.

## Actually yielding map results

Lets update `generate_world_map` again to tell us whenever we receive a yielded map update:

```rust
for _i in builder.build_map() {
    println!("Map update");
};
```

Running now will show this just the once - when the map finished. That's not really useful for iteratively updating the map, but it's a start. Now in `simple_map.rs`, we'll update the `build_map` function to pass the scope to `rooms_and_corridors`:

```rust
fn build_map(&mut self) -> Generator<(), Map> {
    Gn::new_scoped(move |mut s| {
        println!("Running build map");
        self.rooms_and_corridors(&mut s);
        done!();
    })
}
```

The only change is that we're passing `s` (the scope) as a mutable reference. We update the function signature to match:

```rust
fn rooms_and_corridors(&mut self, scope: &mut generator::Scope<(), Map>) {
```

Now, whenever we want to submit an updated map to the caller - we can call `scope.yield_(map)`! Notice the underscore after the name; `yield` is a reserved keyword in Rust, and *will* be used when they finish the generator system. Adding the underscore fixes the name collision.

So now, whenever we `push` a room to the map - we also submit an updated map to the caller:

```rust
self.rooms.push(new_room);
scope.yield_(self.map.clone());
```

We clone the map to ensure that we don't accidentally *move* it out of our function. It makes another copy in memory, which is a fast operation.

`cargo run` now shows a whole bunch of "Map update" outputs on the console: one for every room that was pushed, and one more for the finished map.

## Iteratively displaying progress

We need to incorporate map generation into our running state, so we can display each cycle on the screen. A natural first step is to add an entry to our `RunState` enum:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, 
    PreRun, 
    PlayerTurn, 
    MonsterTurn, 
    ShowInventory, 
    ShowDropItem, 
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration
}
```

This also requires handling in our `tick` function. For now, we'll just change state.

```rust
match newrunstate {
    RunState::MapGeneration => {
        newrunstate = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame };
    }
```

We also tell the game to start in the new mode:

```rust
gs.ecs.insert(RunState::MapGeneration);
```

Now, lets add some variables to our state to help us:

```rust
pub struct State {
    pub ecs: World,
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32
}
```

And in the beginning of `main` where it creates the state, give it some values:

```rust
let mut gs = State {
    ecs: World::new(),
    mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
    mapgen_index : 0,
    mapgen_history: Vec::new(),
    mapgen_timer: 0.0
};
```

Now, we adjust our `generate_world_map` to actually store the history as it is generated:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let mut builder = map_builders::random_builder(new_depth);
    for map in builder.build_map() {
        self.mapgen_history.push(map);
    };
    ...
```

That's progress! We store each map generation as a "frame", setting up the ability to render generation progress. Now we adjust our `tick` function to actually display it:

```rust
match newrunstate {
    RunState::MapGeneration => {
        ctx.cls();
        for v in self.mapgen_history[self.mapgen_index].revealed_tiles.iter_mut() {
            *v = true;
        }
        draw_map(&self.mapgen_history[self.mapgen_index], ctx);

        self.mapgen_timer += ctx.frame_time_ms;
        if self.mapgen_timer > 500.0 {
            self.mapgen_timer = 0.0;
            self.mapgen_index += 1;
            if self.mapgen_index == self.mapgen_history.len() {
                newrunstate = self.mapgen_next_state.unwrap();
            }
        }
    }
    ...
```

This is similar to the particle code: it clears the screen, ensures the whole map is visible, and passes the current iteration to the map render code. We then increment the timer by the frame time, and if 500ms have passed we reset the timer to zero and move to the next "frame" in the map history. If its the end of the list, we move to the next state.

If you `cargo run` now, you'll get to watch the map be generated:

(TODO: screenshot)

## Extending to render other transitions

We should also visualize map generation when the game ends, and when we go to the next level. Modify the following in `tick`:

```rust
RunState::GameOver => {
    let result = gui::game_over(ctx);
    match result {
        gui::GameOverResult::NoSelection => {}
        gui::GameOverResult::QuitToMenu => {
            self.game_over_cleanup();
            newrunstate = RunState::MapGeneration;
            self.mapgen_next_state = Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
        }
    }
}
```

And:

```rust
RunState::NextLevel => {
    self.goto_next_level();
    self.mapgen_next_state = Some(RunState::PreRun);
    newrunstate = RunState::MapGeneration;
}
```

## Making it optional

We probably only want to show the visualizer when we are working on maps - otherwise we are showing the player the complete level! Towards the top of `main.rs` add:

```rust
const SHOW_MAPGEN_VISUALIZER : bool = true;
```

Then we modify the map visualizer state:

```rust
RunState::MapGeneration => {
    if !SHOW_MAPGEN_VISUALIZER {
        newrunstate = self.mapgen_next_state.unwrap();
    }
    ...
```

Now you can change the global to `false` when you don't want the player to see maps being generated.

## Wrap-Up

This finishes building the test harness - you can watch maps spawn, which should make generating maps (the topic of the next few chapters) a lot more intuitive.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-24-map-testing)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-24-map-testing/)
---

Copyright (C) 2019, Herbert Wolverson.

---