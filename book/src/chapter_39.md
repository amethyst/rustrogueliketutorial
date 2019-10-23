# Improved corridors

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Our corridor generation so far has been quite primitive, featuring overlaps - and unless you use Voronoi spawning, nothing in them. This chapter will try to offer a few more generation strategies (in turn providing even more map variety), and allow hallways to contain entities.

## New corridor strategy: nearest neighbor

One way to make a map feel more natural is to build hallways between near neighbors. This reduces (but doesn't eliminate) overlaps, and looks more like something that someone might actually *build*. We'll make a new file, `map_builders/rooms_corridors_nearest.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, Rect, draw_corridor };
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub struct NearestCorridors {}

impl MetaMapBuilder for NearestCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl NearestCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<NearestCorridors> {
        Box::new(NearestCorridors{})
    }

    fn corridors(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let rooms : Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Nearest Corridors require a builder with room structures");
        }

        let mut connected : HashSet<usize> = HashSet::new();
        for (i,room) in rooms.iter().enumerate() {
            let mut room_distance : Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();
            let room_center_pt = rltk::Point::new(room_center.0, room_center.1);
            for (j,other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = rltk::Point::new(other_center.0, other_center.1);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                        room_center_pt,
                        other_center_pt
                    );
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap() );
                let dest_center = rooms[room_distance[0].0].center();
                draw_corridor(
                    &mut build_data.map,
                    room_center.0, room_center.1,
                    dest_center.0, dest_center.1
                );
                connected.insert(i);
                build_data.take_snapshot();
            }
        }
    }
}
```

There's some boilerplate with which you should be familiar by now, so lets walk through the `corridors` function:

1. We start by obtaining the `rooms` list, and `panic!` if there isn't one.
2. We make a new `HashSet` named `connected`. We'll add rooms to this as they gain exits, so as to avoid linking repeatedly to the same room.
3. For each room, we retrieve an "enumeration" called `i` (the index number in the vector) and the `room`:
    1. We create a new vector called `room_distance`. It stores tuples containing the room being considered's index and a floating point number that will store its distance to the current room.
    2. We calculate the center of the room, and store it in a `Point` from RLTK (for compatibility with the distance algorithms).
    3. For every room, we retrieve an enumeration called `j` (it's customary to use `i` and `j` for counters, presumably dating back to the days in which longer variable names were expensive!), and the `other_room`.
        1. If `i` and `j` are equal, we are looking at a corridor to/from the same room. We don't want to do that, so we skip it!
        2. Likewise, if the `other_room`'s index (`j`) is in our `connected` set, then we don't want to evaluate it either - so we skip that.
        3. We calculate the distance from the outer room (`room`/`i`) to the room we are evaluating (`other_room`/`j`).
        4. We push the distance and the `j` index into `room_distance`.
    4. If the list for `room_distance` is empty, we skip ahead. Otherwise:
    5. We use `sort_by` to sort the `room_distance` vector, with the shortest distance being closest.
    6. Then we use the `draw_corridor` function to draw a corridor from the center of the current `room` to the closest room (index `0` in `room_distance`)

Lastly, we'll modify `random_builder` in `map_builders/mod.rs` to use this algorithm:

```rust
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    /*
    let mut builder = BuilderChain::new(new_depth);
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder)
    }

    if rng.roll_dice(1, 3)==1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    if rng.roll_dice(1, 20)==1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(PrefabBuilder::vaults());

    builder*/

    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomDrawer::new());
    builder.with(RoomSorter::new(RoomSort::LEFTMOST));
    builder.with(NearestCorridors::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStairs::new());
    builder.with(RoomBasedStartingPosition::new());
    builder
}
```

This gives nicely connected maps, with sensibly short corridor distances. If you `cargo run` the project, you should see something like this:

![Screenshot](./c39-s1.gif).

Overlapping corridors *can* still happen, but it is now quite unlikely.

## Corridors with Bresenham Lines

Instead of dog-legging around a corner, we can draw corridors as a straight line. This is a little more irritating for the player to navigate (more corners to navigate), but can give a pleasing effect. We'll create a new file, `map_builders/rooms_corridors_lines.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, Rect, TileType };
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub struct StraightLineCorridors {}

impl MetaMapBuilder for StraightLineCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl StraightLineCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<StraightLineCorridors> {
        Box::new(StraightLineCorridors{})
    }

    fn corridors(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let rooms : Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Straight Line Corridors require a builder with room structures");
        }

        let mut connected : HashSet<usize> = HashSet::new();
        for (i,room) in rooms.iter().enumerate() {
            let mut room_distance : Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();
            let room_center_pt = rltk::Point::new(room_center.0, room_center.1);
            for (j,other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = rltk::Point::new(other_center.0, other_center.1);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                        room_center_pt,
                        other_center_pt
                    );
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap() );
                let dest_center = rooms[room_distance[0].0].center();
                let line = rltk::line2d(
                    rltk::LineAlg::Bresenham, 
                    room_center_pt, 
                    rltk::Point::new(dest_center.0, dest_center.1)
                );
                for cell in line.iter() {
                    let idx = build_data.map.xy_idx(cell.x, cell.y);
                    build_data.map.tiles[idx] = TileType::Floor;
                }
                connected.insert(i);
                build_data.take_snapshot();
            }
        }
    }
}
```

This is almost the same as the previous one, but instead of calling `draw_corridor` we use RLTK's line function to plot a line from the center of the source and destination rooms. We then mark each tile along the line as a floor. If you modify your `random_builder` to use this:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(SimpleMapBuilder::new());
builder.with(RoomDrawer::new());
builder.with(RoomSorter::new(RoomSort::LEFTMOST));
builder.with(StraightLineCorridors::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStairs::new());
builder.with(RoomBasedStartingPosition::new());
builder
```

Then `cargo run` your project, you will see something like this:

![Screenshot](./c39-s2.gif).

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-39-halls)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-39-halls/)
---

Copyright (C) 2019, Herbert Wolverson.

---