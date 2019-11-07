# Experience and Levelling

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The design document talks about using *Town Portal* to return to town, which implies that *backtracking* is possible - that is, it's possible to return to levels. This is quite a common feature of games such as Dungeon Crawl: Stone Soup (in which it is standard procedure to leave items in a "stash" where hopefully the monsters won't find them).

If we're going to support going back and forth between levels (either via entrance/exit pairs, or through mechanisms such as teleports/portals), we need to adjust the way we handle levels and transitioning between them.

## A Master Dungeon Map

We'll start by making a structure to store *all* of our maps - the `MasterDungeonMap`. Make a new file, `map/dungeon.rs` and we'll start putting it together:

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::{Map};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps : HashMap<i32, Map>
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap{ maps: HashMap::new() }
    }

    pub fn store_map(&mut self, map : &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth : i32) -> Option<Map> {
        if self.maps.contains_key(&depth) {
            let mut result = self.maps[&depth].clone();
            result.tile_content = vec![Vec::new(); (result.width * result.height) as usize];
            Some(result)
        } else {
            None
        }
    }
}
```

This is pretty easy to follow: the structure itself has a single, private (no `pub`) field - `maps`. It's a `HashMap` - a dictionary - of `Map` structures, indexed by the map depth. We provide a constructor for easy creation of the class (`new`), and functions to `store_map` (save a map) and `get_map` (retrieve one as an `Option`, with `None` indicating that we don't have one). We also added the `Serde` decorations to make the structure serializable - so you can save the game. We also remake the `tile_content` field, because we don't serialize it.

In `map/mod.rs`, you need to add a line: `pub mod dungeon;`. This tells the module to expose the dungeon to the world.

## Adding backwards exits

Let's add upwards staircases to the world. In `map/tiletype.rs` we add the new type:

```rust
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, 
    Floor, 
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs
}
```

Then in themes.rs, we add a couple of missing patterns to render it (in each theme):

```rust
TileType::UpStairs => { glyph = rltk::to_cp437('<'); fg = RGB::from_f32(0., 1.0, 1.0); }
```

## Storing New Maps As We Make Them

Currently, whenever the player enters a new level we call `generate_world_map` in `main.rs` to make a new one from scratch. Instead, we'd like to have the whole dungeon map as a global resource - and reference it when we make new maps, using the *existing* one if possible. It's also pretty messy having this in `main.rs`, so we'll take this opportunity to refactor it into our map system.

We can start by adding a a `MasterDungeonMap` resource to the ECS `World`. In your `main` function, at the top of the `ecs.insert` calls, add a line to insert a `MasterDungeonMap` into the `World` (I've included the line after it so you can see where it goes):

```rust
gs.ecs.insert(map::MasterDungeonMap::new());
gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
```

Now we'll simplify `generate_world_map` down to the basics:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let map_building_info = map::level_transition(&mut self.ecs, new_depth);
    if let Some(history) = map_building_info {
        self.mapgen_history = history;
    }
}
```

This function resets the builder information (which is good, because it's taking care of its own responsibilities - but not others), and asks a new function `map::level_transition` if it has history information. If it does, it stores it as the map building history; otherwise, it leaves the history empty.

In `map/dungeon.rs`, we'll build the outer function it is calling (and remember to add it to the `pub use` section in `map/mod.rs`!):

```rust
pub fn level_transition(ecs : &mut World, new_depth: i32) -> Option<Vec<Map>> {
    // Obtain the master dungeon map
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    // Do we already have a map?
    if dungeon_master.get_map(new_depth).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_depth);
        None
    } else {
        std::mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, new_depth))
    }
}
```

This function obtains the master map from the ECS World, and calls `get_map`. If there is one, it calls `transition_to_existing_map`. If there isn't, it calls `transition_to_new_map`. Note the `std::mem::drop` calls: obtaining `dungeon_master` from the World holds a "borrow" to it; we need to stop borrowing (drop it) before we pass the ECS on to the other functions, to avoid multiple reference issues.

The new function `transition_to_new_map` is the code from the old `generate_world_map` function, modified to not rely on `self`. It has one new section at the end:

```rust
fn transition_to_new_map(ecs : &mut World, new_depth: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = level_builder(new_depth, &mut rng, 80, 50);
    builder.build_map(&mut rng);
    if new_depth > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
            builder.build_data.map.tiles[up_idx] = TileType::UpStairs;            
        }
    }
    let mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = ecs.write_resource::<Map>();
        *worldmap_resource = builder.build_data.map.clone();
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }

    // Spawn bad guys
    std::mem::drop(rng);
    builder.spawn_entities(ecs);

    // Place the player and update resources
    let (player_x, player_y) = (player_start.x, player_start.y);
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new(player_x, player_y);
    let mut position_components = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let player_pos_comp = position_components.get_mut(*player_entity);
    if let Some(player_pos_comp) = player_pos_comp {
        player_pos_comp.x = player_x;
        player_pos_comp.y = player_y;
    }

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let vs = viewshed_components.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }

    // Store the newly minted map
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&builder.build_data.map);

    mapgen_history
}
```

At the very end, it returns the building history. Before that, it obtains access to the new `MasterDungeonMap` system and adds the new map to the stored map list. We also add an "up" staircase to the starting position.

## Retrieving maps we've visited before

Now we need to handle loading up a previous map! It's time to flesh out `transition_to_existing_map`:

```rust
fn transition_to_existing_map(ecs: &mut World, new_depth: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    // Find the down stairs and place the player
    let w = map.width;
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            let mut player_position = ecs.write_resource::<Point>();
            *player_position = Point::new(idx as i32 % w, idx as i32 / w);
            let mut position_components = ecs.write_storage::<Position>();
            let player_pos_comp = position_components.get_mut(*player_entity);
            if let Some(player_pos_comp) = player_pos_comp {
                player_pos_comp.x = idx as i32 % w;
                player_pos_comp.y = idx as i32 / w;
            }
        }
    }

    *worldmap_resource = map;

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let vs = viewshed_components.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }
}
```

So this is quite simple: we get the map from the dungeon master list, and store it as the current map in the `World`. We scan the map for a down staircase, and put the player on it. We also mark the player's visibility as dirty, so it will be recalculated for the new map.

## Input for previous level

Now we need to handle the actual transition. Since we handle going down a level with `RunState::NextLevel`, we'll add a state for going back up:

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
    PreviousLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration
}
```

We'll also need to handle it in our state matching function. We'll basically copy the "next level" option:

```rust
RunState::PreviousLevel => {
    self.goto_previous_level();
    self.mapgen_next_state = Some(RunState::PreRun);
    newrunstate = RunState::MapGeneration;
}
```

We'll copy/paste `goto_next_level()` and `goto_previous_level()` and change some numbers around:

```rust
fn goto_previous_level(&mut self) {
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
    self.generate_world_map(current_depth - 1);

    // Notify the player and give them some health
    let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
    gamelog.entries.insert(0, "You ascend to the previous level.".to_string());
}
```

Next, in `player.rs` (where we handle input) - we need to handle receiving the "go up" instruction. Again, we'll basically copy "go down":

```rust
VirtualKeyCode::Comma => {
    if try_previous_level(&mut gs.ecs) {
        return RunState::PreviousLevel;
    }
}
```

This in turn requires that we copy `try_next_level` and make `try_previous_level`:

```rust
pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.insert(0, "There is no way up from here.".to_string());
        false
    }
}
```

If you `cargo run` now, you can transition between maps. When you go back, however - it's a ghost town. There's *nobody* else on the level. Spooky, and the loss of your Mom should upset you!

## Entity freezing and unfreezing

If you think back to the first part of the tutorial, we spent some time making sure that we *delete* everything that isn't the player when we change level. It made sense: you'd never be coming back, so why waste memory on keeping them? Now that we're able to go back and forth, we need to keep track of where things are - so we can find them once again. We can also take this opportunity to clean up our transitions a bit - it's messy with all those functions!

Thinking about what we want to do, our objective is to store an entity's position *on another level*. So we need to store the level, as well as their `x/y` positions. Lets make a new component. In `components.rs` (and register in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32
}
```

We can actually make a relatively simple function to adjust our entity state. In `map/dungeon.rs`, we'll make a new function:

```rust
pub fn freeze_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    // Find positions and make OtherLevelPosition
    let mut pos_to_delete : Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player_entity {
            other_level_positions.insert(entity, OtherLevelPosition{ x: pos.x, y: pos.y, depth: map_depth }).expect("Insert fail");
            pos_to_delete.push(entity);
        }
    }

    // Remove positions
    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}
```

This is another relatively simple function: we get access to various stores, and then iterate all entities that have a position. We check that it isn't the player (since they are handled differently); if they *aren't* - we add an `OtherLevelPosition` for them, and mark them in the `pos_to_delete` vector. Then we iterate the vector, and remove `Position` components from everyone whom we marked.

Restoring them to life (thawing) is quite easy, too:

```rust
pub fn thaw_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    // Find OtherLevelPosition
    let mut pos_to_delete : Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        if entity != *player_entity && pos.depth == map_depth {
            positions.insert(entity, Position{ x: pos.x, y: pos.y }).expect("Insert fail");
            pos_to_delete.push(entity);
        }
    }

    // Remove positions
    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}
```

This is basically the same function, but with the logic reversed! We *add* `Position` components, and *delete* `OtherLevelPosition` components.

In `main.rs`, we have a mess of `goto_next_level` and `goto_previous_level` functions. Lets replace them with one generic function that understands which way we are going:

```rust
fn goto_level(&mut self, offset: i32) {
    freeze_level_entities(&mut self.ecs);

    // Build a new map and place the player
    let current_depth = self.ecs.fetch::<Map>().depth;
    self.generate_world_map(current_depth + offset, offset);

    // Notify the player
    let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
    gamelog.entries.insert(0, "You change level.".to_string());
}
```

This is a lot simpler - we call our new `freeze_level_entities` function, obtain the current depth, and call `generate_world_map` with the new depth. What's this? We're also passing the `offset`. We need to know which way you are going, otherwise you can complete whole levels by going back and then forward again - and being teleported to the "down" staircase! So we'll modify `generate_world_map` to take this parameter:

```rust
fn generate_world_map(&mut self, new_depth : i32, offset: i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
    if let Some(history) = map_building_info {
        self.mapgen_history = history;
    } else {
        map::thaw_level_entities(&mut self.ecs);
    }
}
```

Notice that we're basically calling the same code, but also passing `offset` to `level_transition` (more on that in a second). We also call `thaw` if we didn't make a new map. That way, new maps get new entities - old maps get the old ones.

You'll need to fix various calls to `generate_world_map`. You can pass `0` as the offset if you are making a new level. You'll also want to fix the two `match` entries for changing level:

```rust
RunState::NextLevel => {
    self.goto_level(1);
    self.mapgen_next_state = Some(RunState::PreRun);
    newrunstate = RunState::MapGeneration;
}
RunState::PreviousLevel => {
    self.goto_level(-1);
    self.mapgen_next_state = Some(RunState::PreRun);
    newrunstate = RunState::MapGeneration;
}
```

Lastly, we need to open up `dungeon.rs` and make a simple change to the level transition system to handle taking an offset:

```rust
pub fn level_transition(ecs : &mut World, new_depth: i32, offset: i32) -> Option<Vec<Map>> {
    // Obtain the master dungeon map
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    // Do we already have a map?
    if dungeon_master.get_map(new_depth).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_depth, offset);
        None
    } else {
        std::mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, new_depth))
    }
}
```

The only difference here is that we pass the offset to `transition_to_existing_map`. Here's that updated function:

```rust
fn transition_to_existing_map(ecs: &mut World, new_depth: i32, offset: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();    

    // Find the down stairs and place the player
    let w = map.width;
    let stair_type = if offset < 0 { TileType::DownStairs } else { TileType::UpStairs };
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == stair_type {
        ...
```

We updated the signature, and use it to determine where to place the player. If offset is less than 0, we want a down staircase - otherwise we want an up staircase.

You can `cargo run` now, and hop back and forth between levels to your heart's content - the entities on each level will be exactly where you left them!

## Saving/Loading the game

Now we need to include the dungeon master map in our save game; otherwise, reloading will keep the current map and generate a whole bunch of new ones - with invalid entity placement!

## More seamless transition

## Stair dancing

## Wrap Up

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-55-backtrack)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-55-backtrack)
---

Copyright (C) 2019, Herbert Wolverson.

---