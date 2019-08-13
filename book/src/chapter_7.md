# Dealing Damage (and taking some!)

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Now that we have monsters, we want them to be more interesting than just yelling at you on the console! This chapter will make them chase you, and introduce some basic game stats to let you fight your way through the hordes.

# Chasing the Player

The first thing we need to do is finish implementing `BaseMap` for our `Map` class. In particular, we need to support `get_available_exits` - which is used by the pathfinding.

In our `Map` implementation, we'll need a helper function:

```rust
fn is_exit_valid(&self, x:i32, y:i32) -> bool {
    if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
    let idx = (y * self.width) + x;
    self.tiles[idx as usize] != TileType::Wall
}
```
This takes an index, and calculates if it can be entered.

We then implement the trait, using this helper:

```rust
fn get_available_exits(&self, idx:i32) -> Vec<(i32, f32)> {
    let mut exits : Vec<(i32, f32)> = Vec::new();
    let x = idx % self.width;
    let y = idx / self.width;

    // Cardinal directions
    if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
    if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
    if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, 1.0)) };
    if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, 1.0)) };

    exits
}
```

Pretty straight-forward: we evaluate each possible exit, and add it to the `exits` vector if it can be taken. Next, we modify the main loop in `monster_ai_system`:

```rust
extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Monster, Name, Map, Position};
extern crate rltk;
use rltk::{Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed,_monster,name,mut pos) in (&mut viewshed, &monster, &name, &mut position).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                println!("{} shouts insults", name.name);
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32, 
                    map.xy_idx(player_pos.x, player_pos.y) as i32, 
                    &mut *map
                );
                if path.success && path.steps.len()>1 {
                    pos.x = path.steps[1] % map.width;
                    pos.y = path.steps[1] / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
```

We've changed a few things to allow write access, requested access to the map. We've also added an `#[allow...]` to tell the linter that we really did mean to use quite so much in one type! The meat is the `a_star_search` call; RLTK includes a high-performance A* implementation, so we're asking it for a path from the monster's position to the player. Then we check that the path succeeded, and has more than 2 steps (step 0 is always the current location). If it does, then we move the monster to that point - and set their viewshed to be dirty.

If you `cargo run` the project, monsters will now chase the player - and stop if they lose line-of-sight. We're not preventing monsters from standing on each other - or you - and we're not having them *do* anything other than yell at your console - but it's a good start. It wasn't too hard to get chase mechanics in!

# Blocking access

We don't want monsters to walk on top of each other, nor do we want them to get stuck in a traffic jam hoping to find the player; we'd rather they are willing to try and flank the player! We'll accompany this by keeping track of what parts of the map are blocked.

First, we'll add another vector of bools to our `Map`:

```rust
#[derive(Default)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>
}
```

We'll also initialize it, just like the other vectors:

```rust
let mut map = Map{
    tiles : vec![TileType::Wall; 80*50],
    rooms : Vec::new(),
    width : 80,
    height: 50,
    revealed_tiles : vec![false; 80*50],
    visible_tiles : vec![false; 80*50],
    blocked : vec![false; 80*50]
};
```

Lets introduce a new function to populate whether or not a tile is blocked. In the `Map` implementation:

```rust
pub fn populate_blocked(&mut self) {
    for (i,tile) in self.tiles.iter_mut().enumerate() {
        self.blocked[i] = *tile == TileType::Wall;
    }
}
```

This function is very simple: it sets `blocked` for a tile to true if its a wall, false otherwise (we'll expand it when we add more tile types). While we're working with `Map`, lets adjust `is_exit_valid` to use this data:

```rust
fn is_exit_valid(&self, x:i32, y:i32) -> bool {
    if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
    let idx = (y * self.width) + x;
    !self.blocked[idx as usize]
}
```

Now we'll make a new component, `BlocksTile`. You should know the drill by now; in `Components.rs`:

```rust
#[derive(Component, Debug)]
pub struct BlocksTile {}
```rust

Then register it in `main.rs`: `gs.ecs.register::<BlocksTile>();`

We should apply `BlocksTile` to NPCs - so our NPC creation code becomes:

```rust
gs.ecs.create_entity()
    .with(Position{ x, y })
    .with(Renderable{
        glyph,
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
    })
    .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
    .with(Monster{})
    .with(Name{ name: format!("{} #{}", &name, i) })
    .with(BlocksTile{})
    .build();
```

Lastly, we need to populate the blocked list. We'll probably extend this system later, so we'll go with a nice generic name `map_indexing_system.rs`:

```rust
extern crate specs;
use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers) = data;

        map.populate_blocked();
        for (position, _blocks) in (&position, &blockers).join() {
            let idx = map.xy_idx(position.x, position.y);
            map.blocked[idx] = true;
        }
    }
}
```

This tells the map to setup blocking from the terrain, and then iterates all entities with a `BlocksTile` component, and applies them to the blocked list. We need to register it with the dispatcher; in `main.rs`:

```rust
let mut gs = State {
    ecs: World::new(),
    systems : DispatcherBuilder::new()
        .with(MapIndexingSystem{}, "map_indexing_system", &[])
        .with(VisibilitySystem{}, "visibility_system", &[])
        .with(MonsterAI{}, "monster_ai", &["visibility_system", "map_indexing_system"])
        .build(),
    runstate : RunState::Running
};
```

We didn't specify any dependencies, we're relying upon Specs to figure out if it can run concurrently with anything. We do however add it to the dependency list for `MonsterAI` - the AI relies on its results, so it has to be done.

If you `cargo run` now, monsters no longer end up on top of each other - but they do end up on top of the player. We should fix that. We can make the monster only yell when it is adjacent to the player. In `monster_ai_system.rs`, add this above the visibility test:

```rust
let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
if distance < 1.5 {
    // Attack goes here
    println!("{} shouts insults", name.name);
    return;
}
```

Lastly, we want to stop the player from walking over monsters. In `player.rs`, we replace the `if` statement that looks for walls with:

```rust
if !map.blocked[destination_idx] {
```

Since we already put walls into the blocked list, this should take care of the issue for now. `cargo run` shows that monsters now block the player. They block them *perfectly* - so a monster that wants to be in your way is an unpassable obstacle!

# Allowing Diagonal Movement

It would be nice to be able to bypass the monsters - and diagonal movement is a mainstay of roguelikes. So lets go ahead and support it. In `map.rs`'s `get_available_exits` function, we add them:

```rust
fn get_available_exits(&self, idx:i32) -> Vec<(i32, f32)> {
    let mut exits : Vec<(i32, f32)> = Vec::new();
    let x = idx % self.width;
    let y = idx / self.width;

    // Cardinal directions
    if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
    if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
    if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, 1.0)) };
    if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, 1.0)) };

    // Diagonals
    if self.is_exit_valid(x-1, y-1) { exits.push(((idx-self.width)-1, 1.45)); }
    if self.is_exit_valid(x+1, y-1) { exits.push(((idx-self.width)+1, 1.45)); }
    if self.is_exit_valid(x-1, y+1) { exits.push(((idx+self.width)-1, 1.45)); }
    if self.is_exit_valid(x+1, y+1) { exits.push(((idx+self.width)+1, 1.45)); }

    exits
}
```

We also modify the `player.rs` input code:

```rust
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => { return RunState::Paused } // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad4 => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad6 => try_move_player(1, 0, &mut gs.ecs),            
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad8 => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad2 => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            _ => { return RunState::Paused }
        },
    }
    RunState::Running
}
```

You can now diagonally dodge around monsters - and they can move/attack diagonally.

---

Copyright (C) 2019, Herbert Wolverson.

---