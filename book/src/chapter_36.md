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

## Cellular Automata

You should understand the basic idea here, now - we're breaking up builders into small chunks, and implementing the appropriate traits for the map type. Looking at Cellular Automata maps, you'll see that we do things a little differently:

* We make a map as usual. This obviously belongs in `CellularAutomotaBuilder`.
* We search for a starting point close to the middle. This looks like it should be a separate step.
* We search the map for unreachable areas and cull them. This also looks like a separate step.
* We place the exit far from the starting position. That's *also* a different algorithm step.

The good news is that the last three of those are used in lots of other builders - so implementing them will let us reuse the code, and not keep repeating ourselves. The bad news is that if we run our cellular automata builder with the existing room-based steps, it will crash - we don't *have* rooms!

So we'll start by constructing the basic map builder. Like the others, this is mostly just rearranging code to fit with the new trait scheme. Here's the new `cellular_automota.rs` file:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct CellularAutomotaBuilder {}

impl InitialMapBuilder for CellularAutomotaBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomotaBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<CellularAutomotaBuilder> {
        Box::new(CellularAutomotaBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 { build_data.map.tiles[idx] = TileType::Floor } 
                else { build_data.map.tiles[idx] = TileType::Wall }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automota rules
        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height-1 {
                for x in 1..build_data.map.width-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    }
                    else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}
```

### Non-Room Starting Points

It's entirely possible that we don't actually *want* to start in the middle of the map. Doing so presents lots of opportunities (and helps ensure connectivity), but maybe you would rather the player trudge through lots of map with less opportunity to pick the wrong direction. Maybe your story makes more sense if the player arrives at one end of the map and leaves via another. Lets implement a starting position system that takes a *preferred* starting point, and picks the closest valid tile. Create `area_starting_points.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, Position, TileType};
use rltk::RandomNumberGenerator;

#[allow(dead_code)]
pub enum XStart { LEFT, CENTER, RIGHT }

#[allow(dead_code)]
pub enum YStart { TOP, CENTER, BOTTOM }

pub struct AreaStartingPosition {
    x : XStart, 
    y : YStart
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
    #[allow(dead_code)]
    pub fn new(x : XStart, y : YStart) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition{
            x, y
        })
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let seed_x;
        let seed_y;

        match self.x {
            XStart::LEFT => seed_x = 1,
            XStart::CENTER => seed_x = build_data.map.width / 2,
            XStart::RIGHT => seed_x = build_data.map.width - 2
        }

        match self.y {
            YStart::TOP => seed_y = 1,
            YStart::CENTER => seed_y = build_data.map.height / 2,
            YStart::BOTTOM => seed_y = build_data.map.height - 2
        }

        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if *tiletype == TileType::Floor {
                available_floors.push(
                    (
                        idx,
                        rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                            rltk::Point::new(seed_x, seed_y)
                        )
                    )
                );
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;

        build_data.starting_position = Some(Position{x : start_x, y: start_y});
    }
}
```

We've covered the boilerplate enough to not need to go over it again - so lets step through the *build* function:

1. We are taking in a couple of `enum` types: preferred position on the X and Y axes.
2. So we set `seed_x` and `seed_y` to a point closest to the specified locations.
3. We iterate through the whole map, adding floor tiles to `available_floors` - and calculating the distance to the preferred starting point.
4. We sort the available tile list, so the lower distances are first.
5. We pick the first one on the list.

Note that we also `panic!` if there are no floors at all.

The great part here is that this will work for *any* map type - it searches for floors to stand on, and tries to find the closest starting point.

### Culling Unreachable Areas

We've previously had good luck with culling areas that can't be reached from the starting point. So lets formalize that into its own meta-builder. Create `cull_unreachable.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
    #[allow(dead_code)]
    pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x, 
            starting_pos.y
        );
        build_data.map.populate_blocked();
        let map_starts : Vec<i32> = vec![start_idx as i32];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width, build_data.map.height, &map_starts , &build_data.map, 1000.0);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
```

You'll notice this is almost exactly the same as `remove_unreachable_areas_returning_most_distant` from `common.rs`, but without returning a Dijkstra map. That's the intent: we remove areas the player can't get to, and *only* do that.

### Voronoi-based spawning

We also need to replicate the functionality of Voronoi-based spawning. Create `voronoi_spawning.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType, spawner};
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl VoronoiSpawning {
    #[allow(dead_code)]
    pub fn new() -> Box<VoronoiSpawning> {
        Box::new(VoronoiSpawning{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut noise_areas : HashMap<i32, Vec<usize>> = HashMap::new();
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1 .. build_data.map.height-1 {
            for x in 1 .. build_data.map.width-1 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }

        // Spawn the entities
        for area in noise_areas.iter() {
            spawner::spawn_region(&build_data.map, rng, area.1, build_data.map.depth, &mut build_data.spawn_list);
        }
    }
}
```

TODO - text!

### Spawning a distant exit

TODO...

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DistantExit {
    #[allow(dead_code)]
    pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x, 
            starting_pos.y
        );
        build_data.map.populate_blocked();
        let map_starts : Vec<i32> = vec![start_idx as i32];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width, build_data.map.height, &map_starts , &build_data.map, 1000.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start != std::f32::MAX {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }

        // Place a staircase
        let stairs_idx = exit_tile.0;
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
```

### Testing Cellular Automata

We've finally got all the pieces together, so lets give it a test. In `random_builder`, we'll use the new builder chains:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(CellularAutomotaBuilder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

If you `cargo run` now, you'll get to play in a Cellular Automata generated map.

## Updating Drunkard's Walk

You should have a pretty good picture of what we're doing now, so we'll gloss over `drunkard.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType, Position, paint, Symmetry};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DrunkSpawnMode { StartingPoint, Random }

pub struct DrunkardSettings {
    pub spawn_mode : DrunkSpawnMode,
    pub drunken_lifetime : i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry
}

pub struct DrunkardsWalkBuilder {
    settings : DrunkardSettings
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    #[allow(dead_code)]
    pub fn new(settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            settings
        }
    }

    #[allow(dead_code)]
    pub fn open_area() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            }
        })
    }

    #[allow(dead_code)]
    pub fn open_halls() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            },
        })
    }

    #[allow(dead_code)]
    pub fn winding_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None
            },
        })
    }

    #[allow(dead_code)]
    pub fn fat_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None
            },
        })
    }

    #[allow(dead_code)]
    pub fn fearful_symmetry() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both
            },
        })
    }
    
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Set a central starting point
        let starting_position = Position{ x: build_data.map.width / 2, y: build_data.map.height / 2 };
        let start_idx = build_data.map.xy_idx(starting_position.x, starting_position.y);
        build_data.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut digger_count = 0;
        while floor_tile_count  < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = starting_position.x;
                    drunk_y = starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = starting_position.x;
                        drunk_y = starting_position.y;
                    } else {
                        drunk_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                let drunk_idx = build_data.map.xy_idx(drunk_x, drunk_y);
                if build_data.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(&mut build_data.map, self.settings.symmetry, self.settings.brush_size, drunk_x, drunk_y);
                build_data.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => { if drunk_x > 2 { drunk_x -= 1; } }
                    2 => { if drunk_x < build_data.map.width-2 { drunk_x += 1; } }
                    3 => { if drunk_y > 2 { drunk_y -=1; } }
                    _ => { if drunk_y < build_data.map.height-2 { drunk_y += 1; } }
                }

                drunk_life -= 1;
            }
            if did_something { 
                build_data.take_snapshot(); 
            }

            digger_count += 1;
            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
    }
}
```

Once again, you can test it by adjusting `random_builder` to:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(DrunkardsWalkBuilder::fearful_symmetry());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

You can `cargo run` and see it in action.

## Update Diffusion-Limited Aggregation

This is more of the same, so we'll again just provide the code:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType, Position, Symmetry, paint};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

pub struct DLABuilder {
    algorithm : DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}


impl InitialMapBuilder for DLABuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DLABuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    #[allow(dead_code)]
    pub fn walk_inwards() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    #[allow(dead_code)]
    pub fn walk_outwards() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    #[allow(dead_code)]
    pub fn central_attractor() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    #[allow(dead_code)]
    pub fn insectoid() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        })
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Carve a starting seed
        let starting_position = Position{ x: build_data.map.width/2, y : build_data.map.height/2 };
        let start_idx = build_data.map.xy_idx(starting_position.x, starting_position.y);
        build_data.take_snapshot();
        build_data.map.tiles[start_idx] = TileType::Floor;
        build_data.map.tiles[start_idx-1] = TileType::Floor;
        build_data.map.tiles[start_idx+1] = TileType::Floor;
        build_data.map.tiles[start_idx-build_data.map.width as usize] = TileType::Floor;
        build_data.map.tiles[start_idx+build_data.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        while floor_tile_count  < desired_floor_tiles {

            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }

                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = starting_position.x;
                    let mut digger_y = starting_position.y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, digger_x, digger_y);
                }

                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham, 
                        rltk::Point::new( digger_x, digger_y ), 
                        rltk::Point::new( starting_position.x, starting_position.y )
                    );

                    while build_data.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }
            }

            build_data.take_snapshot();

            floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
    }    
}
```

## Updating the Maze Builder

Once again, here's the code:

```rust
use super::{Map,  InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MazeBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Maze gen
        let mut maze = Grid::new((build_data.map.width / 2)-2, (build_data.map.height / 2)-2, rng);
        maze.generate_maze(build_data);
    }
}

/* Maze code taken under MIT from https://github.com/cyucelen/mazeGenerator/ */

const TOP : usize = 0;
const RIGHT : usize = 1;
const BOTTOM : usize = 2;
const LEFT : usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell{
            row,
            column,
            walls: [true, true, true, true],
            visited: false
        }
    }

    unsafe fn remove_walls(&mut self, next : *mut Cell) {
        let x = self.column - (*(next)).column;
        let y = self.row - (*(next)).row;

        if x == 1 {
            self.walls[LEFT] = false;
            (*(next)).walls[RIGHT] = false;
        }
        else if x == -1 {
            self.walls[RIGHT] = false;
            (*(next)).walls[LEFT] = false;
        }
        else if y == 1 {
            self.walls[TOP] = false;
            (*(next)).walls[BOTTOM] = false;
        }
        else if y == -1 {
            self.walls[BOTTOM] = false;
            (*(next)).walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng : &'a mut RandomNumberGenerator
}

impl<'a> Grid<'a> {
    fn new(width: i32, height:i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid{
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width-1 || row > self.height-1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors : Vec<usize> = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbor_indices : [i32; 4] = [
            self.calculate_index(current_row -1, current_column),
            self.calculate_index(current_row, current_column + 1),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1)
        ];

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if !neighbors.is_empty() {
            if neighbors.len() == 1 {
                return Some(neighbors[0]);
            } else {
                return Some(neighbors[(self.rng.roll_dice(1, neighbors.len() as i32)-1) as usize]);
            }
        }
        None
    }

    fn generate_maze(&mut self, build_data : &mut BuilderMap) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.insert(0, self.current);
                    unsafe {
                        let next_cell : *mut Cell = &mut self.cells[next];
                        let current_cell = &mut self.cells[self.current];
                        current_cell.remove_walls(next_cell);
                    }
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();    
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map : &mut Map) {
        // Clear the map
        for i in map.tiles.iter_mut() { *i = TileType::Wall; }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] { map.tiles[idx - map.width as usize] = TileType::Floor }
            if !cell.walls[RIGHT] { map.tiles[idx + 1] = TileType::Floor }
            if !cell.walls[BOTTOM] { map.tiles[idx + map.width as usize] = TileType::Floor }
            if !cell.walls[LEFT] { map.tiles[idx - 1] = TileType::Floor }
        }
    }
}
```

## Updating Voronoi Maps

Here's the updated code for the Voronoi builder:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DistanceAlgorithm { Pythagoras, Manhattan, Chebyshev }

pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm
}


impl InitialMapBuilder for VoronoiCellBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

    #[allow(dead_code)]
    pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

    #[allow(dead_code)]
    pub fn manhattan() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        })
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Make a Voronoi diagram. We'll do this the hard way to learn about the technique!
        let mut voronoi_seeds : Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, build_data.map.width-1);
            let vy = rng.roll_dice(1, build_data.map.height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voroni_distance = vec![(0, 0.0f32) ; self.n_seeds];
        let mut voronoi_membership : Vec<i32> = vec![0 ; build_data.map.width as usize * build_data.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % build_data.map.width;
            let y = i as i32 / build_data.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance;
                match self.distance_algorithm {           
                    DistanceAlgorithm::Pythagoras => {
                        distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Manhattan => {
                        distance = rltk::DistanceAlg::Manhattan.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Chebyshev => {
                        distance = rltk::DistanceAlg::Chebyshev.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                }
                voroni_distance[seed] = (seed, distance);
            }

            voroni_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voroni_distance[0].0 as i32;
        }

        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed { neighbors += 1; }

                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }    
}
```

## Updating Waveform Collapse

Waveform Collapse is a slightly different one to port, because it already had a concept of a "previous builder". That's gone now (chaining is automatic), so there's a bit more to update. Waveform Collapse is a meta-builder, so it implements that trait, rather than the initial map builder. Overall, these changes make it a *lot* more simple!

```rust
use super::{MetaMapBuilder, BuilderMap, Map, TileType};
use rltk::RandomNumberGenerator;
mod common;
use common::*;
mod constraints;
use constraints::*;
mod solver;
use solver::*;

/// Provides a map builder using the Waveform Collapse algorithm.
pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    /// Generic constructor for waveform collapse.
    /// # Arguments
    /// * new_depth - the new map depth
    /// * derive_from - either None, or a boxed MapBuilder, as output by `random_builder`
    #[allow(dead_code)]
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        const CHUNK_SIZE :i32 = 8;
        build_data.take_snapshot();

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constaints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);
                
        build_data.map = Map::new(build_data.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }
            build_data.take_snapshot();
            if solver.possible { break; } // If it has hit an impossible condition, try again
        }
        build_data.spawn_list.clear();
    }

    fn render_tile_gallery(&mut self, constraints: &[MapChunk], chunk_size: i32, build_data : &mut BuilderMap) {
        build_data.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut build_data.map, &constraints[counter], chunk_size, x, y);

            x += chunk_size + 1;
            if x + chunk_size > build_data.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > build_data.map.height {
                    // Move to the next page
                    build_data.take_snapshot();
                    build_data.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }
        build_data.take_snapshot();
    }
}
```

TODO: Text!

Test:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(VoronoiCellBuilder::pythagoras());
builder.with(WaveformCollapseBuilder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

## Updating the Prefab Builder

So here's a fun one. The `PrefabBuilder` is both an `InitialMapBuilder` and a `MetaMapBuilder` - with shared code between the two.

## .. eventually .. Delete the MapBuilder Trait and bits from common

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-36-layers/)
---

Copyright (C) 2019, Herbert Wolverson.

---