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


---

Copyright (C) 2019, Herbert Wolverson.

---