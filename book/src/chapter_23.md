# Refactor: Generic Map Interface

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

[![Hands-On Rust](./beta-webBanner.jpg)](https://pragprog.com/titles/hwrust/hands-on-rust/)

---

So far, we've really just had one map design. It's different every time (unless you hit a repeat random seed), which is a great start - but the world of procedural generation leaves so many more possibilities. Over the next few chapters, we'll start building a few different map types.

## Refactoring the builder - Defining an Interface

Up until now, all of our map generation code has sat in the `map.rs` file. That's fine for a single style, but what if we want to have lots of styles? This is the perfect time to create a proper builder system! If you look at the map generation code in `main.rs`, we have the beginnings of an interface defined:

* We call `Map::new_map_rooms_and_corridors`, which builds a set of rooms.
* We pass that to `spawner::spawn_room` to populate each room.
* We then place the player in the first room.

To better organize our code, we'll make a *module*. Rust lets you make a directory, with a file in it called `mod.rs` - and that directory is now a module. Modules are exposed through `mod` and `pub mod`, and provide a way to keep parts of your code together. The `mod.rs` file provides an *interface* - that is, a list of what is provided by the module, and how to interact with it. Other files in the module can do whatever they want, safely isolated from the rest of the code.

So, we'll create a directory (off of `src`) called `map_builders`. In that directory, we'll create an empty file called `mod.rs`. We're trying to define an interface, so we'll start with a skeleton. In `mod.rs`:

```rust
use super::Map;

trait MapBuilder {
    fn build(new_depth: i32) -> Map;
}
```

The use of `trait` is new! A trait is like an *interface* in other languages: you are saying that any other type can *implement* the trait, and can then be treated as a variable of that type. [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/trait.html) has a great section on traits, as does [The Rust Book](https://doc.rust-lang.org/beta/book/ch10-02-traits.html). What we're stating is that anything can declare itself to be a `MapBuilder` - and that includes a promise that they will provide a `build` function that takes in an ECS `World` object, and returns a map.

Open up `map.rs`, and add a new function - called, appropriately enough, `new`:

```rust
/// Generates an empty map, consisting entirely of solid walls
pub fn new(new_depth : i32) -> Map {
    Map{
        tiles : vec![TileType::Wall; MAPCOUNT],
        rooms : Vec::new(),
        width : MAPWIDTH as i32,
        height: MAPHEIGHT as i32,
        revealed_tiles : vec![false; MAPCOUNT],
        visible_tiles : vec![false; MAPCOUNT],
        blocked : vec![false; MAPCOUNT],
        tile_content : vec![Vec::new(); MAPCOUNT],
        depth: new_depth,
        bloodstains: HashSet::new()
    }
}
```

We'll need this for other map generators, and it makes sense for a `Map` to know how to return a new one as a constructor - without having to encapsulate all the logic for map layout. The idea is that any `Map` will work basically the same way, irrespective of how we've decided to populate it.

Now we'll create a new file, also inside the `map_builders` directory. We'll call it `simple_map.rs` - and it'll be where we put the existing map generation system. We'll also put a skeleton in place here:

```rust
use super::MapBuilder;
use super::Map;
use specs::prelude::*;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        Map::new(new_depth)
    }
}
```

This simply returns an unusable, solid map. We'll flesh out the details in a bit - lets get the interface working, first.

Now, back in `map_builders/mod.rs` we add a public function. For now, it just calls the builder in `SimpleMapBuilder`:

```rust
pub fn build_random_map(new_depth: i32) -> Map {
    SimpleMapBuilder::build(new_depth)
}
```

Finally, we'll tell `main.rs` to actually include the module:

```rust
pub mod map_builders;
```

Ok, so that was a fair amount of work to not actually *do* anything - but we've gained a clean interface offering map creation (via a single function), and setup a *trait* to require that our map builders work in a similar fashion. That's a good start.

## Fleshing out the Simple Map Builder

Now we start moving functionality out of `map.rs` into our `SimpleMapBuilder`. We'll start by adding *another* file to `map_builders` - `common.rs`. This will hold functions that used to be part of the map, and are now commonly used when building.

The file looks like this:

```rust
use super::{Map, Rect, TileType};
use std::cmp::{max, min};

pub fn apply_room_to_map(map : &mut Map, room : &Rect) {
    for y in room.y1 +1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map : &mut Map, x1:i32, x2:i32, y:i32) {
    for x in min(x1,x2) ..= max(x1,x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map : &mut Map, y1:i32, y2:i32, x:i32) {
    for y in min(y1,y2) ..= max(y1,y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}
```

These are exactly the same as the functions from `map.rs`, but with `map` passed as a mutable reference (so you are working on the original, rather than a new one) and all vestiges of `self` gone. These are *free functions* - that is, they are functions available from anywhere, not tied to a type. The `pub fn` means they are public *within the module* - unless we add `pub use` to the module itself, they aren't passed out of the module to the main program. This helps keeps code organized.

Now that we have these helpers, we can start porting the map builder itself. In `simple_map.rs`, we start by fleshing out the `build` function a bit:

```rust
impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        let mut map = Map::new(new_depth);
        SimpleMapBuilder::rooms_and_corridors(&mut map);
        map
    }
}
```

We're calling a new function, `rooms_and_corridors`. Lets build it:

```rust
impl SimpleMapBuilder {
    fn rooms_and_corridors(map : &mut Map) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let stairs_position = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
    }
}
```

You'll notice that this is built as a *method* attached to the `SimpleMapBuilder` structure. It isn't part of the trait, so we can't define it there - but we want to keep it separated from other builders, which might have their own functions. The code itself should look eerily familiar: it's the same as the generator in `map.rs`, but with `map` as a variable rather than being generated inside the function.

This is only the first half of generation, but it's a good start! Now go to `map.rs`, and *delete* the entire `new_map_rooms_and_corridors` function. Also delete the ones we replicated in `common.rs`. The `map.rs` file looks much cleaner now, without any references to map building strategy! Of course, your compiler/IDE is probably telling you that we've broken a bunch of stuff. That's ok - and a normal part of "refactoring" - the process of changing code to be easier to work with.

There are three lines in `main.rs` that are now flagged by the compiler.

* We can replace `*worldmap_resource = Map::new_map_rooms_and_corridors(current_depth + 1);` with `*worldmap_resource = map_builders::build_random_map(current_depth + 1);`.
* `*worldmap_resource = Map::new_map_rooms_and_corridors(1);` can become `*worldmap_resource = map_builders::build_random_map(1);`.
* `let map : Map = Map::new_map_rooms_and_corridors(1);` transforms to `let map : Map = map_builders::build_random_map(1);`.

If you `cargo run` now, you'll notice: the game is exactly the same! That's good: we've successfully *refactored* functionality out of `Map` and into `map_builders`.

## Placing the Player

If you look in `main.rs`, pretty much every time we build a map - we then look for the first room, and use it to place the player. It's quite possible that we won't want to use the same strategy in future maps, so we should indicate where the player goes when we build the map. Lets expand our interface in `map_builders/mod.rs` to also return a position:

```rust
trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(new_depth)
}
```

Notice that we're using a *tuple* to return two values at once. We've talked about those earlier, but this is a great example of why they are useful! We now need to go into `simple_map` to make the `build` function actually return the correct data. The definition of `build` in `simple_map.rs` now looks like this:

```rust
fn build(new_depth: i32) -> (Map, Position) {
    let mut map = Map::new(new_depth);
    let playerpos = SimpleMapBuilder::rooms_and_corridors(&mut map);
    (map, playerpos)
}
```

We'll update the signature of `rooms_and_corridors`:

```rust
fn rooms_and_corridors(map : &mut Map) -> Position {
```

And we'll add a last line to return the center of room 0:

```rust
let start_pos = map.rooms[0].center();
Position{ x: start_pos.0, y: start_pos.1 }
```

This has, *of course*, broken the code we updated in `main.rs`. We can quickly take care of that! The first error can be taken care of with the following code:

```rust
// Build a new map and place the player
let worldmap;
let current_depth;
let player_start;
{
    let mut worldmap_resource = self.ecs.write_resource::<Map>();
    current_depth = worldmap_resource.depth;
    let (newmap, start) = map_builders::build_random_map(current_depth + 1);
    *worldmap_resource = newmap;
    player_start = start;
    worldmap = worldmap_resource.clone();
}

// Spawn bad guys
for room in worldmap.rooms.iter().skip(1) {
    spawner::spawn_room(&mut self.ecs, room, current_depth+1);
}

// Place the player and update resources
let (player_x, player_y) = (player_start.x, player_start.y);
```

Notice how we use [destructuring](https://aminb.gitbooks.io/rust-for-c/content/destructuring/index.html) to retrieve both the map and the start position from the builder. We then put these in the appropriate places. Since assignment in Rust is a *move* operation, this is pretty efficient - and the compiler can get rid of temporary assignments for us.

We do the same again on the second error (around line 369). It's almost exactly the same code, so feel free to check the [source code for this chapter](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-23-bsproom-dungeons) if you are stuck.

Lastly, the final error can be simply replaced like this:

```rust
let (map, player_start) = map_builders::build_random_map(1);
let (player_x, player_y) = (player_start.x, player_start.y);
```

Alright, lets `cargo run` that puppy! If all went well, then... nothing has changed. We've made a significant gain, however: our map building strategy now determines the player's starting point on a level, not the map itself.

## Cleaning up room spawning

It's quite possible that we won't *have* the concept of rooms in some map designs, so we also want to move spawning to be a function of the map builder. We'll add a generic spawner to the interface in `map_builders/mod.rs`:

```rust
trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
    fn spawn(map : &Map, ecs : &mut World, new_depth: i32);
}
```

Simple enough: it requires the ECS (since we're adding entities) and the map. We'll also add a public function, `spawn` to provide an external interface to layout out the monsters:

```rust
pub fn spawn(map : &mut Map, ecs : &mut World, new_depth: i32) {
    SimpleMapBuilder::spawn(map, ecs, new_depth);
}
```

Now we open `simple_map.rs` and actually *implement* `spawn`. Fortunately, it's very simple:

```rust
fn spawn(map : &mut Map, ecs : &mut World) {
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, room, 1);
    }
}
```

Now, we can go into `main.rs` and find every time we loop through calling `spawn_room` and replace it with a call to `map_builders::spawn`.

Once again, `cargo run` should give you the same game we've been looking at for 22 chapters!

## Maintaining builder state

If you look closely at what we have so far, there's one problem: the builder has no way of knowing what should be used for the *second* call to the builder (spawning things). That's because our functions are *stateless* - we don't actually create a builder and give it a way to remember anything. Since we want to support a wide variety of builders, we should correct that.

This introduces a new Rust concept: *dynamic dispatch*. [The Rust Book](https://doc.rust-lang.org/1.8.0/book/trait-objects.html) has a good section on this if you are familiar with the concept. If you've previously used an Object Oriented Programming language, then you will have encountered this also. The basic idea is that you have a "base object" that specifies an *interface* - and multiple objects *implement* the functions from the interface. You can then, at run-time (when the program runs, rather than when it compiles) put *any* object that implements the interface into a variable typed by the *interface* - and when you call the methods from the interface, the *implementation* runs from the actual type. This is nice because your underlying program doesn't have to know about the actual implementations - just how to talk to the interface. That helps keep your program clean.

Dynamic dispatch does come with a cost, which is why Entity Component Systems (and Rust in general) prefer not to use it for performance-critical code. There's actually *two* costs:

1. Since you don't know what type the object is up-front, you have to allocate it via a *pointer*. Rust makes this easy by providing the `Box` system (more on that in a moment), but there is a cost: rather than just jumping to a readily defined piece of memory (which your CPU/memory can generally figure out easily in advance and make sure the cache is ready) the code has to follow the pointer - and then run what it finds at the end of the pointer. That's why some C++ programmers call `->` (dereference operator) the "cache miss operator". Simply by being boxed, your code is slowed down by a tiny amount.
2. Since multiple types can implement methods, the computer needs to know which one to run. It does this with a `vtable` - that is, a "virtual table" of method implementations. So each call has to check the table, find out which method to run, and then run from there. That's *another* cache miss, and more time for your CPU to figure out what to do.

In this case, we're just generating the map - and making very few calls into the builder. That makes the slowdown acceptable, since it's *really small* and not being run frequently. You wouldn't want to do this in your main loop, if you can avoid it!

So - implementation. We'll start by changing our trait to be *public*, and have the methods accept an `&mut self` - which means "this method is a *member* of the trait, and should receive access to `self` - the attached object when we call it. The code looks like this:

```rust
pub trait MapBuilder {
    fn build_map(&mut self, new_depth: i32) -> (Map, Position);
    fn spawn_entities(&mut self, map : &Map, ecs : &mut World, new_depth: i32);
}
```

Notice that I've also taken the time to make the names a bit more descriptive! Now we replace our free function calls with a *factory* function: it creates a `MapBuilder` and returns it. The name is a bit of a lie until we have more map implementations - it claims to be random, but when there's only one choice it's not hard to guess which one it will pick (just ask Soviet election systems!):

```rust
pub fn random_builder() -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slighlty random
    Box::new(SimpleMapBuilder{})
}
```

Notice that it doesn't return a `MapBuilder` - rather it returns a `Box<dyn MapBuilder>`! That's rather convoluted (and in earlier versions of Rust, the `dyn` is optional). A `Box` is a type wrapped in a pointer, whose size may not be known at compile time. It's the same as a C++ `MapBuilder *` - it *points* to a `MapBuilder` rather than actually *being* one. The `dyn` is a flag to say "this should use dynamic dispatch"; the code will work without it (it will be inferred), but it's good practice to flag that you are doing something complicated/expensive here.

The function simply returns `Box::new(SimpleMapBuilder{})`. This is actually two calls, now: we make a box with `Box::new(...)`, and we place an empty `SimpleMapBuilder` into the box.

Over in `main.rs`, we once again have to change all three calls to the map builder. We now need to use the following pattern:

1. Obtain a boxed `MapBuilder` object, from the factory.
2. Call `build_map` as a *method* - that is, a function attached to the *object*.
3. Call `spawn_entities` also as a *method*.

The implementation from `goto_next_level` now reads as follows:

```rust
// Build a new map and place the player
let mut builder = map_builders::random_builder(current_depth + 1);
let worldmap;
let current_depth;
let player_start;
{
    let mut worldmap_resource = self.ecs.write_resource::<Map>();
    current_depth = worldmap_resource.depth;
    let (newmap, start) = builder.build_map(current_depth + 1);
    *worldmap_resource = newmap;
    player_start = start;
    worldmap = worldmap_resource.clone();
}

// Spawn bad guys
builder.spawn_entities(&worldmap, &mut self.ecs, current_depth+1);
```

It's not very different, but now we're *keeping* the builder object around - so subsequent calls to the builder will apply to the same *implementation* (sometimes called "concrete object" - the object that actually physically exists).

If we were to add 5 more map builders, the code in `main.rs` wouldn't care! We can add them to the *factory*, and the rest of the program is blissfully unaware of the workings of the map builder. This is a very good example of how dynamic dispatch can be useful: you have a clearly defined interface, and the rest of the program doesn't *need* to understand the inner workings.

## Adding a constructor to SimpleMapBuilder

We're currently making a SimpleMapBuilder as an empty object. What if it *needs* to keep track of some data? In case we need it, lets add a simple constructor to it and use that instead of a blank object. In `simple_map.rs`, modify the `struct` implementation as follows:

```rust
impl SimpleMapBuilder {
    pub fn new(new_depth : i32) -> SimpleMapBuilder {
        SimpleMapBuilder{}
    }
    ...
```

That simply returns an empty object for now. In `mod.rs`, change the `random_map_builder` function to use it:

```rust
pub fn random_builder(new_depth : i32) -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slighlty random
    Box::new(SimpleMapBuilder::new(new_depth))
}
```

This hasn't gained us anything, but is a bit cleaner - when you write more maps, they may do something in their constructors!

## Cleaning up the trait - simple, obvious steps and single return types

Now that we've come this far, lets extend the trait a bit to obtain the player's position in one function, the map in another, and build/spawn separately. Using small functions tends to make the code easier to read, which is a worthwhile goal in and of itself. In `mod.rs`, we change the interface as follows:

```rust
pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs : &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}
```

There's a few things to note here:

1. `build_map` no longer returns anything at all. We're using it as a function to build map state.
2. `spawn_entities` no longer asks for a Map parameter. Since all map builders *have* to implement a map in order to make sense, we're going to assume that the map builder has one.
3. `get_map` returns a map. Again, we're assuming that the builder implementation keeps one.
4. `get_starting_position` also assumes that the builder will keep one around.

Obviously, our `SimpleMapBuilder` now needs to be modified to work this way. We'll start by modifying the `struct` to include the required variables. This is the map builder's *state* - and since we're doing dynamic object-oriented code, the state remains attached to the object. Here's the code from `simple_map.rs`:

```rust
pub struct SimpleMapBuilder {
    map : Map,
    starting_position : Position,
    depth: i32
}
```

Next, we'll implement the *getter* functions. These are very simple: they simply return the variables from the structure's state:

```rust
impl MapBuilder for SimpleMapBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }
    ...
```

We'll also update the constructor to create the state:

```rust
pub fn new(new_depth : i32) -> SimpleMapBuilder {
    SimpleMapBuilder{
        map : Map::new(new_depth),
        starting_position : Position{ x: 0, y : 0 },
        depth : new_depth
    }
}
```

This also simplifies `build_map` and `spawn_entities`:

```rust
fn build_map(&mut self) {
    SimpleMapBuilder::rooms_and_corridors();
}

fn spawn_entities(&mut self, ecs : &mut World) {
    for room in self.map.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, room, self.depth);
    }
}
```

Lastly, we need to modify `rooms_and_corridors` to work with this interface:

```rust
fn rooms_and_corridors(&mut self) {
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for i in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
        let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in self.map.rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&mut self.map, &new_room);

            if !self.map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = self.map.rooms[self.map.rooms.len()-1].center();
                if rng.range(0,2) == 1 {
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                }
            }

            self.map.rooms.push(new_room);
        }
    }

    let stairs_position = self.map.rooms[self.map.rooms.len()-1].center();
    let stairs_idx = self.map.xy_idx(stairs_position.0, stairs_position.1);
    self.map.tiles[stairs_idx] = TileType::DownStairs;

    let start_pos = self.map.rooms[0].center();
    self.starting_position = Position{ x: start_pos.0, y: start_pos.1 };
}
```

This is very similar to what we had before, but now uses `self.map` to refer to its own copy of the map, and stores the player position in `self.starting_position`.

The calls into the new code in `main.rs` once again change. The call from `goto_next_level` now looks like this:

```rust
let mut builder;
let worldmap;
let current_depth;
let player_start;
{
    let mut worldmap_resource = self.ecs.write_resource::<Map>();
    current_depth = worldmap_resource.depth;
    builder = map_builders::random_builder(current_depth + 1);
    builder.build_map();
    *worldmap_resource = builder.get_map();
    player_start = builder.get_starting_position();
    worldmap = worldmap_resource.clone();
}

// Spawn bad guys
builder.spawn_entities(&mut self.ecs);
```

We basically repeat those changes for the others (see the source). We now have a pretty comfortable interface into the map builder: it exposes enough to be easy to use, without exposing the details of the magic it uses to actually *build* the map!

If you `cargo run` the project now: once again, nothing visible has changed - it still works the way it did before. When you are refactoring, that's a good thing!

## So why do maps still have rooms?

Rooms don't actually do much in the game itself: they are an artifact of how we build the map. It's quite possible that later map builders won't actually *care* about rooms, at least not in the "here's a rectangle, we're calling a room" sense. Lets try and move that abstraction out of the map, and also out of the spawner.

As a first step, in `map.rs` we remove the `rooms` structure completely:

```rust
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}
```

We also remove it from the `new` function. Take a look at your IDE, and you'll notice that you've only broken code in `simple_map.rs`! We weren't *using* the `rooms` anywhere else - which is a pretty big clue that they don't belong in the map we're passing around throughout the main program.

We can fix `simple_map` by putting `rooms` into the builder rather than the map. We'll put it into the structure:

```rust
pub struct SimpleMapBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    rooms: Vec<Rect>
}
```

This requires that we fixup the constructor:

```rust
pub fn new(new_depth : i32) -> SimpleMapBuilder {
    SimpleMapBuilder{
        map : Map::new(new_depth),
        starting_position : Position{ x: 0, y : 0 },
        depth : new_depth,
        rooms: Vec::new()
    }
}
```

The spawn function becomes:

```rust
fn spawn_entities(&mut self, ecs : &mut World) {
    for room in self.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, room, self.depth);
    }
}
```

And we replace *every* instance of `map.rooms` with `self.rooms` in `rooms_and_corridors`:

```rust
fn rooms_and_corridors(&mut self) {
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for i in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
        let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in self.rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&mut self.map, &new_room);

            if !self.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = self.rooms[self.rooms.len()-1].center();
                if rng.range(0,2) == 1 {
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                }
            }

            self.rooms.push(new_room);
        }
    }

    let stairs_position = self.rooms[self.rooms.len()-1].center();
    let stairs_idx = self.map.xy_idx(stairs_position.0, stairs_position.1);
    self.map.tiles[stairs_idx] = TileType::DownStairs;

    let start_pos = self.rooms[0].center();
    self.starting_position = Position{ x: start_pos.0, y: start_pos.1 };
}
```

Once again, `cargo run` the project: and nothing should have changed.

## Wrap-up

This was an interesting chapter to write, because the objective is to finish with code that operates *exactly* as it did before - but with the map builder cleaned into its own module, completely isolated from the rest of the code. That gives us a great starting point to start building new map builders, without having to change the game itself.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-23-generic-map)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-23-generic-map/). There isn't a lot of point, since refactoring aims to *not* change the visible result!
---

Copyright (C) 2019, Herbert Wolverson.

---