# Spatial Mapping

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

You may have noticed that this chapter is "57A" by filename. Some problems emerged with the spatial indexing system, after the AI changes in chapter 57. Rather than change an already oversized chapter with what is a decent topic in and of itself, I decided that it would be better to insert a section. In this chapter, we're going to revise the `map_indexing_system` and associated data. We have a few goals:

* The stored locations of entities, and the "blocked" system should be easy to update mid-turn.
* We want to eliminate entities sharing a space.
* We want to fix the issue of not being able to enter a tile after an entity is slain.
* We'd like to retain good performance.

That's a fairly high bar!

## Building a Spatial Indexing API

Rather than scattering map's `tile_content`, the `blocked` list, the periodically updated system, and calls to these data structures everywhere, it would be a *lot* cleaner to move it behind a unified API. We could then access the API, and functionality changes automatically get pulled in as things improve. That way, we just have to remember to call the API - not remember how it works.

We'll start by making a module. Create a `src\spatial` directory, and put an empty `mod.rs` file in it. Then we'll "stub out" our spatial back-end, adding some content:

```rust
use std::sync::Mutex;
use specs::prelude::*;

struct SpatialMap {
    blocked : Vec<bool>,
    tile_content : Vec<Vec<Entity>>
}

impl SpatialMap {
    fn new() -> Self {
        Self {
            blocked: Vec::new(),
            tile_content: Vec::new()
        }
    }
}

lazy_static! {
    static ref SPATIAL_MAP : Mutex<SpatialMap> = Mutex::new(SpatialMap::new());
}
```

The `SpatialMap` struct contains the spatial information we are storing in `Map`. It's deliberately not public: we want to stop sharing the data directly, and use an API instead. Then we create a `lazy_static`: a mutex-protected global variable, and use that to store the spatial information. Storing it this way allows us to access it without burdening Specs' resources system - and makes it easier to offer access both from within systems and from the outside. Since we're mutex-protecting the spatial map, we also benefit from thread safety; that removes the resource from Specs' threading plan. This makes it easier for the program as a whole to use thread the dispatchers.

### Map API Replacement

We'll need a way to resize the spatial map, when the map changes. In `spatial/mod.rs`:

```rust
pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![false; map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}
```

That's a bit inefficient in that it reallocates - but we don't do it often, so it should be ok. We also need a way to clear the spatial contents:

```rust
pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.clear();
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}
```

And we need an analogue for the map's current `populate_blocked` function (which builds a list of which tiles are blocked *by terrain*):

```rust
pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i,tile) in map.tiles.iter().enumerate() {
        lock.blocked[i] = !tile_walkable(*tile);
    }
}
```

## Update the Map

Update the two map functions that handle spatial mapping to use the new API. In `map/mod.rs`:

```rust
pub fn populate_blocked(&mut self) {
    crate::spatial::populate_blocked_from_map(self);
}

pub fn clear_content_index(&mut self) {
    crate::spatial::clear();
}
```

## Populating the Spatial Index

We already have `map_indexing_system.rs`, handling initial (per-frame, so it doesn't get far out of sync) population of the spatial map. Since we're changing how we're storing the data, we also need to change the system. The indexing system performs two functions on the map's spatial data: it sets tiles as blocked, and it adds indexed entities. We've already created the `clear` and `populate_blocked_from_map` functions it needs. Replace the body of the `MapIndexingSystem`'s `run` function with:

```rust
use super::{Map, Position, BlocksTile, spatial};
...

fn run(&mut self, data : Self::SystemData) {
    let (mut map, position, blockers, entities) = data;

    spatial::clear();
    spatial::populate_blocked_from_map(&*map);
    for (entity, position) in (&entities, &position).join() {
        let idx = map.xy_idx(position.x, position.y);

        // If they block, update the blocking list
        let _p : Option<&BlocksTile> = blockers.get(entity);
        if let Some(_p) = _p {
            spatial::set_blocked(idx);
        }

        // Push the entity to the appropriate index slot. It's a Copy
        // type, so we don't need to clone it (we want to avoid moving it out of the ECS!)
        spatial::index_entity(entity, idx);
    }
}
```

In `spatial/mod.rs`, add the `index_entity` function:

```rust
pub fn index_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].push(entity);
}
```

The map's constructor also needs to tell the spatial system to resize itself. Add the following to the constructor:

```rust
pub fn new<S : ToString>(new_depth : i32, width: i32, height: i32, name: S) -> Map {
    let map_tile_count = (width*height) as usize;
    crate::spatial::set_size(map_tile_count);
    ...
```

## Remove the old spatial data from the map

Time to break stuff! This will cause issues throughout the source-base. Remove `blocked` and `tile_content` from the map. The new `Map` definition is as follows:

```rust
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,
    pub view_blocked : HashSet<usize>,
    pub name : String,
    pub outdoors : bool,
    pub light : Vec<rltk::RGB>,
}
```

You also need to remove these entries from the constructor:

```rust
pub fn new<S : ToString>(new_depth : i32, width: i32, height: i32, name: S) -> Map {
    let map_tile_count = (width*height) as usize;
    crate::spatial::set_size(map_tile_count);
    Map{
        tiles : vec![TileType::Wall; map_tile_count],
        width,
        height,
        revealed_tiles : vec![false; map_tile_count],
        visible_tiles : vec![false; map_tile_count],
        depth: new_depth,
        bloodstains: HashSet::new(),
        view_blocked : HashSet::new(),
        name : name.to_string(),
        outdoors : true,
        light: vec![rltk::RGB::from_f32(0.0, 0.0, 0.0); map_tile_count]
    }
}
```

The `is_exit_valid` function in `Map` breaks, because it accesses `blocked`. In `spatial/mod.rs` we'll make a new function to provide this functionality:

```rust
pub fn is_blocked(idx: usize) -> bool {
    SPATIAL_MAP.lock().unwrap().blocked[idx]
}
```

This allows us to fix the map's `is_exit_valid` function:

```rust
fn is_exit_valid(&self, x:i32, y:i32) -> bool {
    if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
    let idx = self.xy_idx(x, y);
    !crate::spatial::is_blocked(idx)
}
```

### Fixing map/dungeon.rs

The `get_map` function in `map/dungeon.rs` creates a new (unused) `tile_content` entry. We don't need that anymore, so we'll remove it. The new function is:

```rust
pub fn get_map(&self, depth : i32) -> Option<Map> {
    if self.maps.contains_key(&depth) {
        let mut result = self.maps[&depth].clone();
        Some(result)
    } else {
        None
    }
}
```

### Fixing the AI

Looking through the AI functions, we're often querying `tile_content` directly. Since we're trying for an API now, we can't do that! The most common use-case is iterating the vector representing a tile. We'd like to avoid the mess that results from returning a lock, and then ensuring that it is freed - this leaks too much implementation detail from an API. Instead, we'll provide a means of iterating tile content with a closure. Add the following to `spatial/mod.rs`:

```rust
pub fn for_each_tile_content<F>(idx: usize, f: F) 
where F : Fn(Entity)
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        f(*entity);
    }
}
```

The `f` variable is a generic parameter, using `where` to specify that it must be a mutable function that accepts an `Entity` as a parameter. This gives us a similar interface to `for_each` on iterators: you can run a function on each entity in a tile, relying on closure capture to let you handle local state when calling it.

Open up `src/ai/adjacent_ai_system.rs`. The `evaluate` function was broken by our change. With the new API, fixing it is quite straightforward:

```rust
fn evaluate(idx : usize, map : &Map, factions : &ReadStorage<Faction>, my_faction : &str, reactions : &mut Vec<(Entity, Reaction)>) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                other_entity,
                crate::raws::faction_reaction(my_faction, &faction.name, &crate::raws::RAWS.lock().unwrap())
            ));
        }
    });
}
```

I like this API - it's very similar to the old setup, but cleanly wrapped!

### Approach API: Some Nasty Code!

> If you were wondering why I defined the API, and then changed it: it's so that you can see how the sausage is made. API building like this is always an iterative process, and it's good to see how things evolve.

Look at `src/ai/approach_ai_system.rs`. The code is pretty gnarly: we're manually changing `blocked` when the entity moves. Worse, we may not be doing it right! It simply unsets `blocked`; if for some reason the tile were still blocked, the result would be incorrect. That won't work; we need a *clean* way of moving entities around, and preserving the `blocked` status.

Adding a `BlocksTile` check to everything whenever we move things is going to be slow, and pollute our already-large Specs lookups with even more references. Instead, we'll change how we are storing entites. We'll also change how we are storing `blocked`. In `spatial/mod.rs`:

```rust
struct SpatialMap {
    blocked : Vec<(bool, bool)>,
    tile_content : Vec<Vec<(Entity, bool)>>
}
```

The `blocked` vector now contains a tuple of two bools. The first is "does the map block it?", the second is "is it blocked by an entity?". This requires that we change a few other functions. We're also going to *delete* the `set_blocked` function and make it automatic from the `populate_blocked_from_map` and `index_entity` functions. Automatic is good: there are fewer opportunities to shoot one's foot!

```rust
pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![(false, false); map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}

pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.iter_mut().for_each(|b| { b.0 = false; b.1 = false; });
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}

pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i,tile) in map.tiles.iter().enumerate() {
        lock.blocked[i].0 = !tile_walkable(*tile);
    }
}

pub fn index_entity(entity: Entity, idx: usize, blocks_tile: bool) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].push((entity, blocks_tile));
    if blocks_tile {
        lock.blocked[idx].1 = true;
    }
}

pub fn is_blocked(idx: usize) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked[idx].0 || lock.blocked[idx].1
}

pub fn for_each_tile_content<F>(idx: usize, mut f: F)
where F : FnMut(Entity)
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        f(entity.0);
    }
}
```

That requires that we tweak the `map_indexing_system` again. The great news is that it keeps getting shorter:

```rust
fn run(&mut self, data : Self::SystemData) {
    let (mut map, position, blockers, entities) = data;

    spatial::clear();
    spatial::populate_blocked_from_map(&*map);
    for (entity, position) in (&entities, &position).join() {
        let idx = map.xy_idx(position.x, position.y);
        spatial::index_entity(entity, idx, blockers.get(entity).is_some());
    }
}
```

So with that done, let's go back to `approach_ai_system`. Looking at the code, with the best of intentions we were *trying* to update `blocked` based on an entity having moved. We naievely cleared `blocked` from the source tile, and set it in the destination tile. We use that pattern a few times, so let's create an API function (in `spatial/mod.rs`) that actually works consistently:

```rust
pub fn move_entity(entity: Entity, moving_from: usize, moving_to: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    let mut entity_blocks = false;
    lock.tile_content[moving_from].retain(|(e, blocks) | {
        if *e == entity {
            entity_blocks = *blocks;
            false
        } else {
            true
        }
    });
    lock.tile_content[moving_to].push((entity, entity_blocks));

    // Recalculate blocks for both tiles
    let mut from_blocked = false;
    let mut to_blocked = false;
    lock.tile_content[moving_from].iter().for_each(|(_,blocks)| if *blocks { from_blocked = true; } );
    lock.tile_content[moving_to].iter().for_each(|(_,blocks)| if *blocks { to_blocked = true; } );
    lock.blocked[moving_from].1 = from_blocked;
    lock.blocked[moving_to].1 = to_blocked;
}
```

This allows us to fix `ai/approach_ai_system.rs` with a much cleaner bit of code:

```rust
if path.success && path.steps.len()>1 {
    let idx = map.xy_idx(pos.x, pos.y);
    pos.x = path.steps[1] as i32 % map.width;
    pos.y = path.steps[1] as i32 / map.width;
    entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
    let new_idx = map.xy_idx(pos.x, pos.y);
    crate::spatial::move_entity(entity, idx, new_idx);
    viewshed.dirty = true;
}
```

The file `ai/chase_ai_system.rs` has the same issue. The fix is nearly identical:

```rust
if path.success && path.steps.len()>1 && path.steps.len()<15 {
    let idx = map.xy_idx(pos.x, pos.y);
    pos.x = path.steps[1] as i32 % map.width;
    pos.y = path.steps[1] as i32 / map.width;
    entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
    let new_idx = map.xy_idx(pos.x, pos.y);
    viewshed.dirty = true;
    crate::spatial::move_entity(entity, idx, new_idx);
    turn_done.push(entity);
} else {
    end_chase.push(entity);
}
```

### Fixing up ai/default_move_system.rs

This file is a little more complicated. The first broken section both queries and updates the blocked index. Change it to:

```rust
if x > 0 && x < map.width-1 && y > 0 && y < map.height-1 {
    let dest_idx = map.xy_idx(x, y);
    if !crate::spatial::is_blocked(dest_idx) {
        let idx = map.xy_idx(pos.x, pos.y);
        pos.x = x;
        pos.y = y;
        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
        crate::spatial::move_entity(entity, idx, dest_idx);
        viewshed.dirty = true;
    }
}
```

The `RandomWaypoint` option is a very similar change:

```rust
if path.len()>1 {
    if !crate::spatial::is_blocked(path[1] as usize) {
        pos.x = path[1] as i32 % map.width;
        pos.y = path[1] as i32 / map.width;
        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
        let new_idx = map.xy_idx(pos.x, pos.y);
        crate::spatial::move_entity(entity, idx, new_idx);
        viewshed.dirty = true;
        path.remove(0); // Remove the first step in the path
    }
    // Otherwise we wait a turn to see if the path clears up
} else {
    mode.mode = Movement::RandomWaypoint{ path : None };
}
```

### Fixing ai/flee_ai_system.rs

This is very similar to the default move change:

```rust
if let Some(flee_target) = flee_target {
    if !crate::spatial::is_blocked(flee_target as usize) {
        crate::spatial::move_entity(entity, my_idx, flee_target);
        viewshed.dirty = true;
        pos.x = flee_target as i32 % map.width;
        pos.y = flee_target as i32 / map.width;
        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
    }
}
```

### Fixing ai/visible_ai_system.rs

The AI's visibility system uses an `evaluate` function, like the one in the adjacent AI setup. It can be changed to use a closure:

```rust
fn evaluate(idx : usize, map : &Map, factions : &ReadStorage<Faction>, my_faction : &str, reactions : &mut Vec<(usize, Reaction, Entity)>) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                idx,
                crate::raws::faction_reaction(my_faction, &faction.name, &crate::raws::RAWS.lock().unwrap()),
                other_entity
            ));
        }
    });
}
```

### The various Inventory Systems

In `inventory_system.rs`, the `ItemUseSystem` performs a spatial lookup. This is another one that can be replaced with the closure system:

Change:
```rust
for mob in map.tile_content[idx].iter() {
    targets.push(*mob);
}
```

To:

```rust
crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob) );
```

Further down, there's another one.

```rust
for mob in map.tile_content[idx].iter() {
    targets.push(*mob);
}
```

Becomes:

```rust
crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
```

### Fixing player.rs

The function `try_move_player` does a really big query of the spatial indexing system. It also sometimes returns mid-calculation, which our API doesn't currently support. We'll add a new function to our `spatial/mod.rs` file to enable this:

```rust
pub fn for_each_tile_content_with_gamemode<F>(idx: usize, mut f: F) -> RunState
where F : FnMut(Entity)->Option<RunState>
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        if let Some(rs) = f(entity.0) {
            return rs;
        }
    }

    RunState::AwaitingInput
}
```

This function runs like the other one, but accepts an optional game mode from the closure. If the game mode is `Some(x)`, then it returns `x`. If it hasn't received any modes by the end, it returns `AwaitingInput`.

 Replacing it with the new API is mostly a matter of using the new functions, and performing the index check inside the closure. Here's the new function:

```rust
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Attributes>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let mut result = RunState::AwaitingInput;

    let mut swap_entities : Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return RunState::AwaitingInput; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_gamemode(destination_idx, |potential_target| {
            let mut hostile = true;
            if combat_stats.get(potential_target).is_some() {
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap()
                    );
                    if reaction != Reaction::Attack { hostile = false; }
                }
            }
            if !hostile {
                // Note that we want to move the bystander
                swap_entities.push((potential_target, pos.x, pos.y));

                // Move the player
                pos.x = min(map.width-1 , max(0, pos.x + delta_x));
                pos.y = min(map.height-1, max(0, pos.y + delta_y));
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");

                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
                return Some(RunState::Ticking);
            } else {
                let target = combat_stats.get(potential_target);
                if let Some(_target) = target {
                    wants_to_melee.insert(entity, WantsToMelee{ target: potential_target }).expect("Add target failed");
                    return Some(RunState::Ticking);
                }
            }
            let door = doors.get_mut(potential_target);
            if let Some(door) = door {
                door.open = true;
                blocks_visibility.remove(potential_target);
                blocks_movement.remove(potential_target);
                let glyph = renderables.get_mut(potential_target).unwrap();
                glyph.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
                return Some(RunState::Ticking);
            }
            None
        });

        if !crate::spatial::is_blocked(destination_idx) {
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = min(map.width-1 , max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));
            let new_idx = map.xy_idx(pos.x, pos.y);
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
            crate::spatial::move_entity(entity, old_idx, new_idx);

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            result = RunState::Ticking;
            match map.tiles[destination_idx] {
                TileType::DownStairs => result = RunState::NextLevel,
                TileType::UpStairs => result = RunState::PreviousLevel,
                _ => {}
            }
        }
    }

    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = m.1;
            their_pos.y = m.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            crate::spatial::move_entity(m.0, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }

    result
}
```

Notice the `TODO`: we're going to want to look at that before we are done. We're moving entities around - and not updating the spatial map.

The `skip_turn` also needs to replace direct iteration of `tile_content` with the new closure-based setup:

```rust
crate::spatial::for_each_tile_content(idx, |entity_id| {
    let faction = factions.get(entity_id);
    match faction {
        None => {}
        Some(faction) => {
            let reaction = crate::raws::faction_reaction(
                &faction.name,
                "Player",
                &crate::raws::RAWS.lock().unwrap()
            );
            if reaction == Reaction::Attack {
                can_heal = false;
            }
        }
    }
});
```

### Fixing the Trigger System

`trigger_system.rs` also needs some love. This is just another direct `for` loop replacement with the new closure:

```rust
crate::spatial::for_each_tile_content(idx, |entity_id| {
    if entity != entity_id { // Do not bother to check yourself for being a trap!
        let maybe_trigger = entry_trigger.get(entity_id);
        match maybe_trigger {
            None => {},
            Some(_trigger) => {
                // We triggered it
                let name = names.get(entity_id);
                if let Some(name) = name {
                    log.entries.push(format!("{} triggers!", &name.name));
                }

                hidden.remove(entity_id); // The trap is no longer hidden

                // If the trap is damage inflicting, do it
                let damage = inflicts_damage.get(entity_id);
                if let Some(damage) = damage {
                    particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage, false);
                }

                // If it is single activation, it needs to be removed
                let sa = single_activation.get(entity_id);
                if let Some(_sa) = sa {
                    remove_entities.push(entity_id);
                }
            }
        }
    }
});
```

### More of the same in the Visibility System

The `visibility_system.rs` needs a very similar fix. `for e in map.tile_content[idx].iter() {` and associated body becomes:

```rust
crate::spatial::for_each_tile_content(idx, |e| {
    let maybe_hidden = hidden.get(e);
    if let Some(_maybe_hidden) = maybe_hidden {
        if rng.roll_dice(1,24)==1 {
            let name = names.get(e);
            if let Some(name) = name {
                log.entries.push(format!("You spotted a {}.", &name.name));
            }
            hidden.remove(e);
        }
    }
});
```

### Saving and Loading

The `saveload_system.rs` file also needs some tweaking. Replace:

```rust
worldmap.tile_content = vec![Vec::new(); (worldmap.height * worldmap.width) as usize];
```

With:

```rust
crate::spatial::set_size((worldmap.height * worldmap.width) as usize);
```

If you `cargo build`, it now compiles! That's progress. Now `cargo run` the project, and see how it goes. The game runs at a decent speed, and is playable. There are still a few issues - we'll resolve these in turn.

## Cleaning up the dead

We'll start with the "dead still bock tiles" problem. The problem occurs because entities don't go away until `delete_the_dead` is called, and the whole map reindexes. That may not occur in time to help with moving into the target tile. Add a new function to our spatial API (in `spatial/mod.rs`):

```rust
pub fn remove_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].retain(|(e, _)| *e != entity );
    let mut from_blocked = false;
    lock.tile_content[idx].iter().for_each(|(_,blocks)| if *blocks { from_blocked = true; } );
    lock.blocked[idx].1 = from_blocked;
}
```

Then modify the `damage_system` to handle removing entities on death:

```rust
if stats.hit_points.current < 1 && dmg.1 {
    xp_gain += stats.level * 100;
    if let Some(pos) = pos {
        let idx = map.xy_idx(pos.x, pos.y);
        crate::spatial::remove_entity(entity, idx);
    }
}
```

That sounds good - but running it shows that we *still* have the problem. A bit of heavy debugging showed that `map_indexing_system` is running inbetween the events, and restoring the incorrect data. We don't want the dead to show up on our indexed map, so we edit the indexing system to check. The fixed indexing system looks like this: we've added a check for dead people.

```rust
use specs::prelude::*;
use super::{Map, Position, BlocksTile, Pools, spatial};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( ReadExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        ReadStorage<'a, Pools>,
                        Entities<'a>,);

    fn run(&mut self, data : Self::SystemData) {
        let (map, position, blockers, pools, entities) = data;

        spatial::clear();
        spatial::populate_blocked_from_map(&*map);
        for (entity, position) in (&entities, &position).join() {
            let mut alive = true;
            if let Some(pools) = pools.get(entity) {
                if pools.hit_points.current < 1 {
                    alive = false;
                }
            }
            if alive {
                let idx = map.xy_idx(position.x, position.y);
                spatial::index_entity(entity, idx, blockers.get(entity).is_some());
            }
        }
    }
}
```

You can now move into the space occupied by the recently deceased.

## Handling entity swaps

Remember that we marked a `TODO` in the player handler, for when we want to swap entities positions? Let's get that figured out. Here's a version that updates the destinations:

```rust
for m in swap_entities.iter() {
    let their_pos = positions.get_mut(m.0);
    if let Some(their_pos) = their_pos {
        let old_idx = map.xy_idx(their_pos.x, their_pos.y);
        their_pos.x = m.1;
        their_pos.y = m.2;
        let new_idx = map.xy_idx(their_pos.x, their_pos.y);
        crate::spatial::move_entity(m.0, old_idx, new_idx);
        result = RunState::Ticking;
    }
}
```

## Wrap-Up

It still isn't absolutely perfect, but it's a *lot* better. I played for a while, and on release mode it is zoomy. Issues with not being able to enter tiles are gone, hit detection is working. Equally importantly, we've cleaned up some hacky code.

> Note: this chapter is in alpha. I'm still applying these fixes to subsequent chapters, and will update this when it is done.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-57a-spatial)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-57a-spatial)
---

Copyright (C) 2019, Herbert Wolverson.

---