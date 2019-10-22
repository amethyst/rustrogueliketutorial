# Fun With Layers

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Now that we have a nice, clean layering system we'll take the opportunity to play with it a bit. This chapter is a collection of fun things you can do with layers, and will introduce a few new layer types. It's meant to whet your appetite to write more: the sky really is the limit!

## Existing Algorithms as Meta-Builders

Let's start by adjusting some of our existing algorithms to be useful as filters.

### Applying Cellular Automata as a meta-builder

When we wrote the Cellular Automata system, we aimed for a generic cavern builder. The algorithm is capable of quite a bit more than that - each iteration is basically a "meta builder" running on the previous iteration. A simple tweak allows it to *also* be a meta-builder that only runs a single iteration. 

We'll start by moving the code for a single iteration into its own function:

```rust
fn apply_iteration(&mut self, build_data : &mut BuilderMap) {
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
```

The `build` function is easily modified to call this on each iteration:

```rust
// Now we iteratively apply cellular automota rules
for _i in 0..15 {
    self.apply_iteration(build_data);
}
```

Finally, we'll add an implementation of `MetaMapBuilder` to the mix:

```rust
impl MetaMapBuilder for CellularAutomotaBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.apply_iteration(build_data);
    }
}
```

See how we're calling a single iteration, instead of replacing the whole map? This shows how we can apply the cellular automata *rules* to the map - and change the resultant character quite a bit.

Now lets modify `map_builders/mod.rs`'s `random_builder` to force it to use this as an example:

```rust
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(VoronoiCellBuilder::pythagoras());
    builder.with(CellularAutomotaBuilder::new());
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
    builder
}
```

If you `cargo run` the project now, you'll see something like this:

![Screenshot](./c37-s1.gif).

### Eroding a boxy map with drunken dwarves

The Drunken Walk algorithm can also make a nice post-processing effect, with very minimal modification. In `drunkard.rs`, simply add the following:

```rust
impl MetaMapBuilder for DrunkardsWalkBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}
```

You can test it by once again modifying `random_builder`:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(SimpleMapBuilder::new());
builder.with(DrunkardsWalkBuilder::winding_passages());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

If you `cargo run` the project, you'll see something like this:

![Screenshot](./c37-s2.gif).

Notice how the initial boxy design now looks a bit more natural, because drunken dwarves have carved out sections of the map!

### Attacking your boxy map with Diffusion-Limited Aggregation

DLA can also be modified to erode an existing, boxy map. Simply add the `MetaBuilder` trait to `dla.rs`:

```rust
impl MetaMapBuilder for DLABuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}
```

We'll also add a new mode, `heavy_erosion` - it's the same as "walk inwards", but wants a greater percentage of floor space:

```rust
#[allow(dead_code)]
pub fn heavy_erosion() -> Box<DLABuilder> {
    Box::new(DLABuilder{
        algorithm: DLAAlgorithm::WalkInwards,
        brush_size: 2,
        symmetry: Symmetry::None,
        floor_percent: 0.35,
    })
}
```

And modify your `random_builder` test harness:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(SimpleMapBuilder::new());
builder.with(DLABuilder::heavy_erosion());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

If you `cargo run` the project, you'll see something like this:

![Screenshot](./c37-s3.gif).

## Some New Meta-Builders

There's also plenty of scope to write new map filters. We'll explore a few of the more interesting ones in this section. Pretty much anything you might use as an image filter in a program like Photoshop (or the GIMP!) could be adapted for this purpose. How useful a given filter is remains an open/interesting question!

### Eroding rooms

Nethack-style boxy rooms make for very early-D&D type play, but people often remark that they aren't all that visually pleasing or interesting. One way to keep the basic room style, but get a more organic look, is to run drunkard's walk *inside* each room. I like to call this "exploding the room" - because it looks a bit like you set off dynamite in each room. In `map_builders/`, make a new file `room_exploder.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType, paint, Symmetry, Rect};
use rltk::RandomNumberGenerator;

pub struct RoomExploder {}

impl MetaMapBuilder for RoomExploder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomExploder {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomExploder> {
        Box::new(RoomExploder{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let rooms : Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Explosions require a builder with room structures");
        }

        for room in rooms.iter() {
            let start = room.center();
            let n_diggers = rng.roll_dice(1, 20)-5;
            if n_diggers > 0 {
                for _i in 0..n_diggers {
                    let mut drunk_x = start.0;
                    let mut drunk_y = start.1;

                    let mut drunk_life = 20;
                    let mut did_something = false;

                    while drunk_life > 0 {
                        let drunk_idx = build_data.map.xy_idx(drunk_x, drunk_y);
                        if build_data.map.tiles[drunk_idx] == TileType::Wall {
                            did_something = true;
                        }
                        paint(&mut build_data.map, Symmetry::None, 1, drunk_x, drunk_y);
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

                        for t in build_data.map.tiles.iter_mut() {
                            if *t == TileType::DownStairs {
                                *t = TileType::Floor;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

There's nothing too surprising in this code: it takes the *rooms* list from the parent build data, and then iterates each room. A random number (which can be zero) of *drunkards* is then run from the center of each room, with a short lifespan, carving out the edges of each room. You can test this with the following `random_builder` code:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspDungeonBuilder::new());
builder.with(RoomExploder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

![Screenshot](./c37-s4.gif).

### Rounding Room Corners

Another quick and easy way to make a boxy map look less rectangular is to smooth the corners a bit. Add `room_corner_rounding.rs` to `map_builders/`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType, Rect};
use rltk::RandomNumberGenerator;

pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomCornerRounder {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomCornerRounder> {
        Box::new(RoomCornerRounder{})
    }

    fn fill_if_corner(&mut self, x: i32, y: i32, build_data : &mut BuilderMap) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let idx = build_data.map.xy_idx(x, y);
        let mut neighbor_walls = 0;
        if x > 0 && build_data.map.tiles[idx-1] == TileType::Wall { neighbor_walls += 1; }
        if y > 0 && build_data.map.tiles[idx-w as usize] == TileType::Wall { neighbor_walls += 1; }
        if x < w-2 && build_data.map.tiles[idx+1] == TileType::Wall { neighbor_walls += 1; }
        if y < h-2 && build_data.map.tiles[idx+w as usize] == TileType::Wall { neighbor_walls += 1; }

        if neighbor_walls == 2 {
            build_data.map.tiles[idx] = TileType::Wall;
        }
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let rooms : Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Rounding require a builder with room structures");
        }

        for room in rooms.iter() {
            self.fill_if_corner(room.x1+1, room.y1+1, build_data);
            self.fill_if_corner(room.x2, room.y1+1, build_data);
            self.fill_if_corner(room.x1+1, room.y2, build_data);
            self.fill_if_corner(room.x2, room.y2, build_data);

            build_data.take_snapshot();
        }
    }
}
```

The boilerplate (repeated code) should look familiar by now, so we'll focus on the algorithm in `build`:

1. We obtain a list of rooms, and `panic!` if there aren't any.
2. For each of the 4 corners of the room, we call a new function `fill_if_corner`.
3. `fill_if_corner` counts each of the neighboring tiles to see if it is a wall. If there are exactly 2 walls, then this tile is eligible to become a corner - so we fill in a wall.

You can try it out with the following `random_builder` code:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspDungeonBuilder::new());
builder.with(RoomCornerRounder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

The result (if you `cargo run`) should be something like this:

![Screenshot](./c37-s5.gif).

## Decoupling Rooms and Corridors

## Allowing Corridor Content

## Playing with placement

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-37-layers2)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-37-layers2/)
---

Copyright (C) 2019, Herbert Wolverson.

---