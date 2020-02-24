# Difficulty

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Currently, you can advance through multiple dungeon levels - but they all have the same spawns. There's no ramp-up of difficulty as you advance, and no easy-mode to get you through the beginning. This chapter aims to change that.

# Adding a wait key

An important tactical element of most roguelikes is the ability to skip a turn - let the monsters come to you (and not get the first hit!). As part of turning the game into a more tactical challenge, lets quickly implement turn skipping. In `player.rs` (along with the rest of the input), we'll add numeric keypad 5 and space to be skip:

```rust
// Skip Turn
VirtualKeyCode::Numpad5 => return RunState::PlayerTurn,
VirtualKeyCode::Space => return RunState::PlayerTurn,
```

This adds a nice tactical dimension to the game: you can lure enemies towards you, and benefit from tactical placement. Another frequently found feature of roguelikes is waiting providing some healing if there are no enemies nearby. We'll only implement that for the player, since mobs suddenly healing up is disconcerting! So we'll change that to:

```rust
// Skip Turn
VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
```

Now we implement `skip_turn`:

```rust
fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
```

This looks up various entities, and then iterates the player's viewshed using the `tile_content` system. It checks what the player can see for monsters; if no monster is present, it heals the player by 1 hp. This encourages cerebral play - and can be balanced with the inclusion of a hunger clock at a later date. It also makes the game *really easy* - but we're getting to that!

# Increased difficulty as you delve: spawn tables

Thus far, we've been using a simple spawn system: it randomly picks a number of monsters and items, and then picks each with an equal weight. That's not much like "normal" games, which tend to make some things rare - and some things common. We'll create a generic `random_table` system, for use in the spawn system. Create a new file, `random_table.rs` and put the following in it:

```rust
use rltk::RandomNumberGenerator;

pub struct RandomEntry {
    name : String,
    weight : i32
}

impl RandomEntry {
    pub fn new<S:ToString>(name: S, weight: i32) -> RandomEntry {
        RandomEntry{ name: name.to_string(), weight }
    }
}

#[derive(Default)]
pub struct RandomTable {
    entries : Vec<RandomEntry>,
    total_weight : i32
}

impl RandomTable {
    pub fn new() -> RandomTable {
        RandomTable{ entries: Vec::new(), total_weight: 0 }
    }

    pub fn add<S:ToString>(mut self, name : S, weight: i32) -> RandomTable {
        self.total_weight += weight;
        self.entries.push(RandomEntry::new(name.to_string(), weight));
        self
    }

    pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        if self.total_weight == 0 { return "None".to_string(); }
        let mut roll = rng.roll_dice(1, self.total_weight)-1;
        let mut index : usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone();
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        "None".to_string()
    }
}
```

So this creates a new type, `random_table`. It adds a `new` method to it, to facilitate making a new one. It also creates a `vector` or entries, each of which has a weight and a name (passing strings around isn't very efficient, but makes for clear example code!). It also implements an `add` function that lets you pass in a new name and weight, and updates the structure's `total_weight`. Finally, `roll` makes a dice roll from `0 .. total_weight - 1`, and iterates through entries. If the roll is below the weight, it returns it - otherwise, it reduces the roll by the weight and tests the next entry. This gives a chance equal to the relative weight of the entry for any given item in the table. There's a bit of extra work in there to help chain methods together, for the Rust-like look of chained function calls. We'll use it in `spawner.rs` to create a new function, `room_table`:

```rust
fn room_table() -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2)
        .add("Confusion Scroll", 2)
        .add("Magic Missile Scroll", 4)
}
```

This contains all of the items and monsters we've added so far, with a weight attached. I wasn't very careful with these weights; we'll play with them later! It does mean that a call to `room_table().roll(rng)` will return a random room entry.

Now we simplify a bit. Delete the `NUM_MONSTERS`, `random_monster` and `random_item` functions in `spawner.rs`. Then we replace the room spawning code with:

```rust
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let spawn_table = room_table();
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) - 3;

        for _i in 0 .. num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            _ => {}
        }
    }
}
```

Lets work through this:

1. The first line tells the Rust linter that we really do like to check a `HashMap` for membership and then insert into it - we also set a flag, which doesn't work well with its suggestion.
2. We obtain the global random number generator, and set the number of spawns to be 1d7-3 (for a -2 to 4 range).
3. For each spawn above 0, we pick a random point in the room. We keep picking random points until we find an empty one (or we exceed 20 tries, in which case we give up). Once we find a point, we add it to the `spawn` list with a location and a roll from our random table.
4. Then we iterate the spawn list, match on the roll result and spawn monsters and items.

This is definitely cleaner than the previous approach, and now you are less likely to run into orcs - and more likely to run into goblins and health potions.

A quick `cargo run` shows you the improved spawn variety.

# Increasing the spawn rate as you delve

That gave a nicer distribution, but didn't solve the problem of later levels being of the same difficulty as earlier ones. A quick and dirty approach is to spawn more entities as you descend. That still doesn't *solve* the problem, but it's a start! We'll start by modifying the function signature of `spawn_room` to accept the map depth:

```rust
pub fn spawn_room(ecs: &mut World, room : &Rect, map_depth: i32) {
```

Then we'll change the number of entities that spawn to use this:
```rust
let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;
```

We'll have to change a couple of calls in `main.rs` to pass in the depth:
```rust
for room in map.rooms.iter().skip(1) {
    spawner::spawn_room(&mut gs.ecs, room, 1);
}
```

```rust
// Build a new map and place the player
let worldmap;
let current_depth;
{
    let mut worldmap_resource = self.ecs.write_resource::<Map>();
    current_depth = worldmap_resource.depth;
    *worldmap_resource = Map::new_map_rooms_and_corridors(current_depth + 1);
    worldmap = worldmap_resource.clone();
}

// Spawn bad guys
for room in worldmap.rooms.iter().skip(1) {
    spawner::spawn_room(&mut self.ecs, room, current_depth+1);
}
```

If you `cargo run` now, the first level is quite quiet. Difficulty ramps up a bit as you descend, until you have veritable hordes of monsters!

# Increasing the weights by depth

Let's modify the `room_table` function to include map depth:
```rust
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
}
```
We also change the call to it in `spawn_room` to use it:

```rust
let spawn_table = room_table(map_depth);
```

A `cargo build` later, and voila - you have an increasing probability of finding orcs, fireball and confusion scrolls as you descend. The total weight of goblins, health potions and magic missile scrolls remains the same - but because the others change, their total likelihood diminishes.

# Wrapping Up

You now have a dungeon that increases in difficulty as you descend! In the next chapter, we'll look at giving your character some progression as well (through equipment), to balance things out.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-13-difficulty)**

[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-13-difficulty/)

---

Copyright (C) 2019, Herbert Wolverson.

---
