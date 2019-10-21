# Layering/Builder Chaining

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The last few chapters have introduced an important concept in procedural generation: chained builders. We're happily building a map, calling Waveform Collapse to mutate the map, calling our `PrefabBuilder` to change it again, and so on. This chapter will formalize this process a bit, expand upon it, and leave you with a framework that lets you *clearly* build new maps by chaining concepts together.

## A builder-based interface

Builder chaining is a pretty profound approach to procedurally generating maps, and gives us an opportunity to clean up a lot of the code we've built thus far. We want an interface similar to the way we build entities with `Specs`: a builder, onto which we can keep chaining builders and return it as an "executor" - ready to build the maps. We also want to stop builders from doing more than one thing - they should do one thing, and do it well (that's a good principle of design; it makes debugging easier, and reduces duplication). So we'll start by defining some new structures and interfaces. First of all, we'll make `BuilderMap` in `map_builders/mod.rs`:

```rust
pub struct BuilderMap {
    pub spawn_list : Vec<(usize, String)>,
    pub map : Map,
    pub starting_position : Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history : Vec<Map>
}
```

You'll notice that this has all of the data we've been building into each map builder - and nothing else. It's intentionally generic - we'll be passing it to builders, and letting them work on it. Notice that all the fields are *public* - that's because we're passing it around, and there's a good chance that anything that touches it will need to access any/all of its contents. We're going to put one function into `BuilderMap` - to handle snapshotting development:

```rust
impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}
```

This is the *same* as the `take_snapshot` code we've been mixing into our builders. Since we're using a central repository of map building knowledge, we can promote it to apply to *all* our builders.

With the basic data in place, we need a system for chaining builders. We'll add the `BuilderChain` type:

```rust
pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    new_depth : i32,
    pub build_data : BuilderMap
}
```

This is a more complicated structure, so let's go through it:

* `starter` is an `Option`, so we know if there is one. Not having a first step (a map that doesn't refer to other maps) would be an error condition, so we'll track it. We're referencing a new trait, `InitialMapBuilder`; we'll get to that in a moment.
* `builders` is a vector of `MetaMapBuilders`, another new trait (and again - we'll get to it in a moment). These are builders that operate on the results of previous maps.
* `new_depth` is the same as the map parameter we've been passing around. Rather than keep passing it everywhere, we'll store it once in the builder.
* `build_data` is a public variable (anyone can read/write it), containing the `BuilderMap` we just made.

We'll implement some functions to support it. First up, a *constructor*:

```rust
impl BuilderChain {
    pub fn new(new_depth : i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            new_depth,
            build_data : BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history : Vec::new()
            }
        }
    }
    ...
```

This is pretty simple: it makes a new `BuilderChain` with default values for everything. Now, lets permit our users to add a *starting map* to the chain. (A starting map is a first step that doesn't require a previous map as input, and results in a usable map structure which we may then modify):

```rust
...
pub fn start_with(&mut self, starter : Box<dyn InitialMapBuilder>) {
    match self.starter {
        None => self.starter = Some(starter),
        Some(_) => panic!("You can only have one starting builder.")
    };
}
...
```

There's one new concept in here: `panic!`. If the user tries to add a second starting builder, we'll crash - because that doesn't make any sense. You'd simply be overwriting your previous steps, which is a giant waste of time! We'll also permit the user to add meta-builders:

```rust
...
pub fn with(&mut self, metabuilder : Box<dyn MetaMapBuilder>) {
    self.builders.push(metabuilder);
}
...
```

This is very simple: we simply add the meta-builder to the builder vector. Since vectors remain in the order in which you add to them, your operations will remain sorted appropriately. Finally, we'll implement a function to actually construct the map:

```rust
pub fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator) {
    match &mut self.starter {
        None => panic!("Cannot run a map builder chain without a starting build system"),
        Some(starter) => {
            // Build the starting map
            starter.build_map(rng, &mut self.build_data);
        }
    }

    // Build additional layers in turn
    for metabuilder in self.builders.iter_mut() {
        metabuilder.build_map(rng, &mut self.build_data);
    }
}
```

Let's walk through the steps here:

1. We `match` on our starting map. If there isn't one, we panic - and crash the program with a message that you *have* to set a starting builder.
2. We call `build_map` on the starting map.
3. For each meta-builder, we call `build_map` on it - in the order specified.

That's not too bad! Lets look at the two trait interfaces we've defined, `InitialMapBuilder` and `MetaMapBuilder`. We made them separate types to force the user to only pick *one* starting builder, and not try to put any starting builders in the list of modification layers. The implementation for them is the same:

```rust
pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}
```

`build_map` takes a random-number generator (so we stop creating new ones everywhere!), and a mutable reference to the `BuilderMap` we are working on. So instead of each builder optionally calling the previous one, we're passing along state as we work on it.

We'll also want to implement our spawning system:

```rust
pub fn spawn_entities(&mut self, ecs : &mut World) {
    for entity in self.build_data.spawn_list.iter() {
        spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
    }
}
```

This is almost exactly the same code as our previous spawner in `MapBuilder`, but instead we're spawning from the `spawn_list` in our `build_data` structure. Otherwise, it's identical.

Finally, we'll modify `random_builder` to use our `SimpleMapBuilder` with some new types to break out the creation steps:

```rust
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder
}
```

Notice that we're now taking a `RandomNumberGenerator` parameter. That's because we'd like to use the global RNG, rather than keep making new ones. This way, if the caller sets a "seed" - it will apply to world generation. This is intended to be the topic of a future chapter. We're also now returning a `BuilderChain` rather than a boxed trait - we're hiding the messy boxing/dynamic dispatch inside the implementation, so the caller doesn't have to worry about it. There's also two new types here: `RoomBasedSpawner` and `RoomBasedStartingPosition` - as well as a changed constructor for `SimpleMapBuilder` (it no longer accepts a depth parameter). We'll be looking at that in a second - but first, lets deal with the changes to the main program resulting from the new interface.

## Nice looking interface - but you broke stuff!

In `main.rs`, we need to update our `generate_world_map` function to use the new interface:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = map_builders::random_builder(new_depth, &mut rng);
    builder.build_map(&mut rng);
    std::mem::drop(rng);
    self.mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = self.ecs.write_resource::<Map>();
        *worldmap_resource = builder.build_data.map.clone();
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }

    // Spawn bad guys
    builder.spawn_entities(&mut self.ecs);
```

1. We reset `mapgen_index`, `mapgen_timer` and the `mapgen_history` so that the progress viewer will run from the beginning.
2. We obtain the RNG from the ECS `World`.
3. We create a new `random_builder` with the new interface, passing along the random number generator.
4. We tell it to build the new maps from the chain, also utilizing the RNG.
5. We call `std::mem::drop` on the RNG. This stops the "borrow" on it - so we're no longer borrowing `self` either. This prevents borrow-checker errors on the next phases of code.
6. We *clone* the map builder history into our own copy of the world's history. We copy it so we aren't destroying the builder, yet.
7. We set `player_start` to a *clone* of the builder's determined starting position. Note that we are calling `unwrap` - so the `Option` for a starting position *must* have a value at this point, or we'll crash. That's deliberate: we'd rather crash knowing that we forgot to set a starting point than have the program run in an unknown/confusing state.
8. We call `spawn_entities` to populate the map.

## Modifying SimpleMapBuilder

We can simplify `SimpleMapBuilder` (making it worthy of the name!) quite a bit. Here's the new code:

```rust
use super::{InitialMapBuilder, BuilderMap, Rect, apply_room_to_map, 
    apply_horizontal_tunnel, apply_vertical_tunnel };
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder{})
    }

    fn rooms_and_corridors(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
        let mut rooms : Vec<Rect> = Vec::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                    if rng.range(0,1) == 1 {
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                    }
                }

                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }
        build_data.rooms = Some(rooms);
    }
}
```

Notice that we're only applying the `InitialMapBuilder` trait - `MapBuilder` is no more. We're also not setting a starting position, or spawning entities - those are now the purview of other builders in the chain. We've basically distilled it down to just the room building algorithm.

## Room-based spawning

Create a new file, `room_based_spawner.rs` in the `map_builders` directory. We're going to apply *just* the room populating system from the old `SimpleMapBuilder` here:

```rust
use super::{MetaMapBuilder, BuilderMap, spawner};
use rltk::RandomNumberGenerator;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(&build_data.map, rng, room, build_data.map.depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("Room Based Spawning only works after rooms have been created");
        }
    }
}
```

In this sub-module, we're implementing `MetaMapBuilder`: this builder requires that you already have a map. In `build`, we've copied the old room-based spawning code from `SimpleMapBuilder`, and modified it to operate on the builder's `rooms` structure. To that end, if we `if let` to obtain the inner value of the `Option`; if there isn't one, then we `panic!` and the program quits stating that room-based spawning is only going to work if you *have* rooms.

## Room-based starting position

This is very similar to room-based spawning, but places the player in the first room - just like it used to in `SimpleMapBuilder`. Create a new file, `room_based_starting_position` in `map_builders`:

```rust
use super::{MetaMapBuilder, BuilderMap, Position};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let start_pos = rooms[0].center();
            build_data.starting_position = Some(Position{ x: start_pos.0, y: start_pos.1 });
        } else {
            panic!("Room Based Staring Position only works after rooms have been created");
        }
    }
}
```

## Room-based stairs

This is also very similar to how we generated exit stairs in `SimpleMapBuilder`. Make a new file, `room_based_stairs.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let stairs_position = rooms[rooms.len()-1].center();
            let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room Based Stairs only works after rooms have been created");
        }
    }
}
```

## Putting it together to make a simple map with the new framework

Let's take another look at `random_builder`:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(SimpleMapBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

Now that we've made all of the steps, this should make sense:

1. We *start with* a map generated with the `SimpleMapBuilder` generator.
2. We *modify* the map with the *meta-builder* `RoomBasedSpawner` to spawn entities in rooms.
3. We again *modify* the map with the *meta-builder* `RoomBasedStartingPosition` to start in the first room.
4. Once again, we *modify* the map with the *meta-builder* `RoomBasedStairs` to place a down staircase in the last room.

If you `cargo run` the project now, you'll let lots of warnings about unused code - but the game should play with just the simple map from our first section. You may be wondering *why* we've taken so much effort to keep things the same; hopefully, it will become clear as we clean up more builders!

## Cleaning up the BSP Dungeon Builder

Once again, we can seriously clean-up a map builder! Here's the new version of `bsp_dungeon.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, Map, Rect, apply_room_to_map, 
    TileType, draw_corridor};
use rltk::RandomNumberGenerator;

pub struct BspDungeonBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspDungeonBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspDungeonBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BspDungeonBuilder> {
        Box::new(BspDungeonBuilder{
            rects: Vec::new(),
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push( Rect::new(2, 2, build_data.map.width-5, build_data.map.height-5) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate, &build_data.map) {
                apply_room_to_map(&mut build_data.map, &candidate);
                rooms.push(candidate);
                self.add_subrects(rect);
                build_data.take_snapshot();
            }

            n_rooms += 1;
        }

        // Now we sort the rooms
        rooms.sort_by(|a,b| a.x1.cmp(&b.x1) );

        // Now we want corridors
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect : Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
    }

    fn get_random_rect(&mut self, rng : &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;

        result.x1 += rng.roll_dice(1, 6)-1;
        result.y1 += rng.roll_dice(1, 6)-1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect : Rect, map : &Map) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > map.width-2 { can_build = false; }
                if y > map.height-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = map.xy_idx(x, y);
                    if map.tiles[idx] != TileType::Wall { 
                        can_build = false; 
                    }
                }
            }
        }

        can_build
    }
}
```

Just like `SimpleMapBuilder`, we've stripped out all the non-room building code for a much cleaner piece of code. We're referencing the `build_data` struct from the builder, rather than making our own copies of everything - and the *meat* of the code is largely the same.

Now you can modify `random_builder` to make this map type:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspDungeonBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

If you `cargo run` now, you'll get a dungeon based on the `BspDungeonBuilder`. See how you are reusing the spawner, starting position and stairs code? That's definitely an improvement over the older versions - if you change one, it can now help on multiple builders!

## Same again for BSP Interior

Yet again, we can greatly clean up a builder - this time the `BspInteriorBuilder`. Here's the code for `bsp_interior.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, Rect, TileType, draw_corridor};
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE : i32 = 8;

pub struct BspInteriorBuilder {
    rects: Vec<Rect>
}

impl InitialMapBuilder for BspInteriorBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspInteriorBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BspInteriorBuilder> {
        Box::new(BspInteriorBuilder{
            rects: Vec::new()
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push( Rect::new(1, 1, build_data.map.width-2, build_data.map.height-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng); // Divide the first room

        let rooms_copy = self.rects.clone();
        for r in rooms_copy.iter() {
            let room = *r;
            //room.x2 -= 1;
            //room.y2 -= 1;
            rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }

        // Now we want corridors
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }

        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect : Rect, rng : &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calculate boundaries
        let width  = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new( rect.x1, rect.y1, half_width-1, height );
            self.rects.push( h1 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            let h2 = Rect::new( rect.x1 + half_width, rect.y1, half_width, height );
            self.rects.push( h2 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        } else {
            // Vertical split
            let v1 = Rect::new( rect.x1, rect.y1, width, half_height-1 );
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            let v2 = Rect::new( rect.x1, rect.y1 + half_height, width, half_height );
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }
    }
}
```

You may test it by modifying `random_builder`:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspInteriorBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

`cargo run` will now take you around an interior builder.

## .. eventually .. Delete the MapBuilder Trait

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-36-layers/)
---

Copyright (C) 2019, Herbert Wolverson.

---