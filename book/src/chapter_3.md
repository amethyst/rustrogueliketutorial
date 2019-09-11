# Chapter 3 - Walking a Map

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

A Roguelike without a map to explore is a bit pointless, so in this chapter we'll put together a basic map, draw it, and let your player walk around a bit. We're starting with the code from chapter 2, but with the red smiley faces (and their leftward tendencies) removed.

## Defining the map tiles

We'll start by allowing two tile types: walls and floors. We can represent this with an `enum`:

```rust
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}
```

Notice that we've included some derived features: `Copy` and `Clone` allow this to be used as a "value" type (that is, it just passes around the value instead of pointers), and `PartialEq` allows us to use `==` to see if two tile types match.

## Building a simple map

Now we'll make a function that returns a `vec` (vector) of tiles, representing a simple map. We'll use a vector sized to the whole map, which means we need a way to figure out which array index is at a given x/y position. So first, we make a new function `xy_idx`:

```rust
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}
```

This is simple: it multiplies the `y` position by the map width (80), and adds `x`. This guarantees one tile per location, and efficiently maps it in memory for left-to-right reading.

Then we write the map function:
```rust
fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80*50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}
```

It's pretty simple: it places walls around the outer edges of the map, and then adds 400 random walls anywhere that isn't the player's starting point.

## Making the map visible to the world

Specs includes a concept of "resources" - shared data the whole ECS can use. So in our `main` function, we add a randomly generated map to the world:

```rust
gs.ecs.insert(new_map());
```

The map is now available from anywhere the ECS can see! Now inside your code, you can access the map with the rather unwieldy `let map = self.ecs.get_mut::<Vec<TileType>>();`; it's available to systems in an easier fashion.

## Draw the map

Now that we have a map available, we should put it on the screen! The complete code for the new `draw_map` function looks like this:

```rust
fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
```

This is mostly straightforward. In the declaration, we pass the map as `&[TileType]` rather than `&Vec<TileType>`; this allows us to pass in "slices" (parts of) a map if we so choose. We won't do that yet, but it may be useful later. It's also considered a more "rustic" (that is: idiomatic Rust) way to do things, and the linter (`clippy`) warns about it.

Otherwise, it takes advantage of the way we are storing our map - rows together, one after the other. So it iterates through the entire map structure, adding 1 to the `x` position for each tile. If it hits the map width, it zeroes `x` and adds one to `y`. This way we aren't repeatedly reading all over the array - which can get slow. The actual rendering is very simple: we `match` the tile type, and draw either a period or a hash for walls/floors.

We should also call the function! In our `tick` function, add:

```rust
let map = self.ecs.fetch::<Vec<TileType>>();
draw_map(&map, ctx);
```

## Making walls solid

So now if you run the program (`cargo run`), you'll have a green and grey map with a yellow `@` who can move around. Unfortunately, you'll quickly notice that the player can walk through walls! Fortunately, that's pretty easy to rectify.

To accomplish this, we modify the `try_move_player` to read the map and check that the destination is open:

```rust
fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x += delta_x;
            pos.y += delta_y;

            if pos.x < 0 { pos.x = 0; }
            if pos.x > 79 { pos.y = 79; }
            if pos.y < 0 { pos.y = 0; }
            if pos.y > 49 { pos.y = 49; }
        }
    }
}
```

The new parts are the `let map = ...` part, which uses `fetch` just the same way as the main loop (this is the advantage of storing it in the ECS - you can get to it everywhere without trying to coerce Rust into letting you use global variables!). We calculate the cell index of the player's destination with `let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);` - and if it isn't a wall, we move as normal.

Run the program (`cargo run`) now, and you have a player in a map - and can move around, properly obstructed by walls.

![Screenshot](./c3-s1.gif)

The full program now looks like this:

```rust
extern crate rltk;
use rltk::{Console, GameState, Rltk, RGB, VirtualKeyCode};
extern crate specs;
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}
 
#[derive(Component, Debug)]
struct Player {}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

struct State {
    ecs: World,
    systems: Dispatcher<'static, 'static>
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80*50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x += delta_x;
            pos.y += delta_y;

            if pos.x < 0 { pos.x = 0; }
            if pos.x > 79 { pos.y = 79; }
            if pos.y < 0 { pos.y = 0; }
            if pos.y > 49 { pos.y = 49; }
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}

fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.systems.dispatch(&self.ecs);

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "../resources");
    let mut gs = State {
        ecs: World::new(),
        systems : DispatcherBuilder::new()
            .build()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs.insert(new_map());

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    rltk::main_loop(context, gs);
}
```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-03-walkmap)**

---

Copyright (C) 2019, Herbert Wolverson.

---