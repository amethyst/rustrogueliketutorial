# BSP Room Dungeons (alpha quality)

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Yadda

## Implementing a new map - subdivided BSP

Now that we have a framework to *allow* us to make a new map, lets do so! Nethack started out using a relatively simple map generation system that still produces satisfying maps. You subdivide your map rectangle into ever-smaller rectangles, sub-dividing each rectangle in turn - until you hit a minimum size. Then you randomly join them together to give a more interesting map.

We'll start by making a new file in `map_builders` - `bsp_dungeon.rs`. A skeleton implementation of a new builder goes here:

```rust
use super::{MapBuilder, Map, Rect, apply_room_to_map, 
    apply_horizontal_tunnel, apply_vertical_tunnel, TileType,
    Position, spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct BspDungeonBuilder {}

impl MapBuilder for BspDungeonBuilder {
    fn build(new_depth: i32) -> (Map, Position) {
        let mut map = Map::new(new_depth);
        (map, Position{x:0, y:0})
    }

    fn spawn(map : &Map, ecs : &mut World, new_depth: i32) {
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, new_depth);
        }
    }
}
```

This makes an unusable map - but gives us a starting point. We'll modify `mod.rs` to always use *this* builder, while we work on it:

```rust
pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    BspDungeonBuilder::build(new_depth)
}

pub fn spawn(map : &Map, ecs : &mut World, new_depth: i32) {
    BspDungeonBuilder::spawn(map, ecs, new_depth);
}
```

We'll worry about swapping out map types later. Onto making the map! Note that this implementation is ported from my C++ game, *One Knight in the Dungeon*. We'll start with room generation:

```rust
fn build(new_depth: i32) -> (Map, Position) {
    let mut map = Map::new(new_depth);
    let mut rng = RandomNumberGenerator::new();

    let mut rects : Vec<Rect> = Vec::new(); // Vector to hold our rectangles as we divide
    rects.push( Rect::new(2, 2, map.width-5, map.height-5) ); // Start with a single map-sized rectangle
    let first_room = rects[0];
    add_subrects(&mut rects, first_room); // Divide the first room

    // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
    // room in there, we place it and add it to the rooms list.
    let mut n_rooms = 0;
    while n_rooms < 240 {
        let rect = get_random_rect(&mut rects, &mut rng);
        let candidate = get_random_sub_rect(rect, &mut rng);

        if is_possible(&mut map, candidate) {
            apply_room_to_map(&mut map, &candidate);
            map.rooms.push(candidate);
            add_subrects(&mut rects, rect);
        }

        n_rooms += 1;
    }
    let player_start = map.rooms[0].center();
    (map, Position{ x : player_start.0, y : player_start.1 })
}
```

So what on Earth does this do?

1. We start by initializing a new map and random number generator to use.
2. We create a new vector of `Rect` structures.
3. We create the "first room" - which is really the whole map. We've trimmed a bit to add some padding to the sides of the map.
4. We call `add_subrects`, passing it the rectangle list - and the first room. We'll implement that in a minute, but what it does is: it divides the rectangle into four quadrants, and adds each of the quadrants to the rectangle list.
5. Now we setup a room counter, so we don't infinitely loop.
6. While that counter is less than 240 (a relatively arbitrary limit that gives fun results):
    1. We call `get_random_rect` to retrieve a random rectangle from the rectangles list.
    2. We call `get_random_sub_rect` using this rectangle as an outer boundary. It creates a random room from 3 to 10 tiles in size (on each axis), somewhere within the parent rectangle.
    3. We ask `is_possible` if the candidate can be drawn to the map; every tile must be within the map boundaries, and not already a room. If it IS possible:
        1. We mark it on the map.
        2. We add it to the rooms list.
        3. We call `add_subrects` to sub-divide the rectangle we just used (not the candidate!).

If you `cargo run` now, you will be in a room with no exits. That's a great start.

Now, we sort the rooms by left coordinate. You don't *have* to do this, but it helps make connected rooms line up.

```rust
map.rooms.sort_by(|a,b| a.x1.cmp(&b.x1) );
```

`sort_by` takes a *closure* - that is, an inline function (known as a "lambda" in other languages) as a parameter. You could specify a whole other function if you wanted to, or implement traits on `Rect` to make it sortable - but this is easy enough. It sorts by comparing the `x1` value of each rectangle.

Now we'll add some corridors:

```rust
for i in 0..map.rooms.len()-1 {
    let room = map.rooms[i];
    let next_room = map.rooms[i+1];
    let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
    let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
    let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
    let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
    draw_corridor(&mut map, start_x, start_y, end_x, end_y);
}
```

This iterates the rooms list, ignoring the last one. It fetches the current room, and the next one in the list and calculates a random location (`start_x`/`start_y` and `end_x`/`end_y`) within each room. It then calls the mysterious `draw_corridor` function with these coordinates. Draw corridor adds a line from the start to the end, using only north/south or east/west (it can give 90-degree bends). It won't give you a staggered, hard to navigate perfect line like Bresenham would.

Finally, we need to wrap up and create the exit:

```rust
let stairs = map.rooms[map.rooms.len()-1].center();
let stairs_idx = map.xy_idx(stairs.0, stairs.1);
map.tiles[stairs_idx] = TileType::DownStairs;
```

We place the exit in the last room, guaranteeing that the poor player has a ways to walk.

If you `cargo run` now, you'll see something like this:

![Screenshot](./c24-s1.jpg).

We have a different dungeon, this one quite suited to sewers or similar.

## Randomizing the dungeon per level

Rather than *always* using the BSP sewer algorithm, we would like to sometimes use one or the other. In `map_builders/mod.rs`, replace the `build` function:

```rust
pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 2);
    match builder {
        1 => SimpleMapBuilder::build(new_depth),
        _ => BspDungeonBuilder::build(new_depth)
    }    
}
```

Now when you play, it's a coin toss what type of map you encounter. The `spawn` functions for the types are the same - so we're not going to worry about map builder state until the next chapter.

## Wrap-Up

You've refactored your map building into a new module, and built a simple BSP (Binary Space Partitioning) based map. The game randomly picks a map type, and you have more variety. The next chapter will further refactor map generation, and introduce another technique.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-23-bsproom-dungeons)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-23-bsproom-dungeons/)
---

Copyright (C) 2019, Herbert Wolverson.

---