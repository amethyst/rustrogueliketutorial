# Items and Inventory

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

So far, we have maps, monsters, and bashing things! No roguelike "murder hobo" experience would be complete without items to pick up along the way. This chapter will add some basic items to the game, along with User Interface elements required to pick them up, use them and drop them.

# Thinking about composing items

A major difference between object-oriented and entity-component systems is that rather than thinking about something as being located on an inheritance tree, you think about how it *composes* from components. Ideally, you already have some of the components ready to use!

So... what makes up an item? Thinking about it, an item can be said to have the following properties:

* It has a `Renderable` - a way to draw it.
* If its on the ground, awaiting pickup - it has a `Position`.
* If its NOT on the ground - say in a backpack, it needs a way to indicate that it it is stored. We'll start with `InPack`
* It's an `item`, which implies that it can be picked up. So it'll need an `Item` component of some sort.
* If it can be used, it will need some way to indicate that it *can* be used - and what to do with it.

# Consistently random

Computers are actually really bad at random numbers. Computers are inherently deterministic - so (without getting into cryptographic stuff) when you ask for a "random" number, you are actually getting a "really hard to predict next number in a sequence". The sequence is controlled by a *seed* - with the same seed, you always get the same dice rolls!

Since we have an ever-increasing number of things that use randomness, lets go ahead and make the RNG (Random Number Generator) a resource.

In `main.rs`, we add:

```rust
gs.ecs.insert(rltk::RandomNumberGenerator::new());
```

We can now access the RNG whenever we need it, without having to pass one around. Since we're not creating a new one, we can start it with a seed (we'd use `seeded` instead of `new`, and provide a seed). We'll worry about that later; for now, it's just going to make our code cleaner!

# Improved Spawning

One monster per room, always in the middle, makes for rather boring play. We also need to support spawning items as well as monsters!

To that end, we're going to make a new file `spawner.rs`:

```rust
extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };
extern crate specs;
use specs::prelude::*;
use super::{CombatStats, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : u8, name : S) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
}
```

As you can see, we've taken the existing code in `main.rs` - and wrapped it up in functions in a different module. We don't *have* to do this - but it helps keep things tidy. Since we're going to be expanding our spawning, it's nice to keep things separated out. Now we modify `main.rs` to use it:

```rust
let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

gs.ecs.insert(rltk::RandomNumberGenerator::new());
for room in map.rooms.iter().skip(1) {
    let (x,y) = room.center();
    spawner::random_monster(&mut gs.ecs, x, y);
}
```

That's definitely tidier! `cargo run` will give you exactly what we had at the end of the previous chapter.

# Spawn All The Things

We're going to extend the function to spawn multiple monsters per room, with 0 being an option. In `spawner.rs`, we create a new function - `spawn_room`:

```rust
/// Fills a room with stuff!
pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;

        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }
}
```

This obtains the RNG and the map, and rolls a dice for how many monsters it should spawn. It then keeps trying to add random positions that aren't already occupied, until sufficient monsters have been created. Each monster is then spawned at the determined location. The borrow checker isn't at all happy with the idea that we mutably access `rng`, and then pass the ECS itself along: so we introduce a scope to keep it happy (automatically dropping access to the RNG when we are done with it).

In `main.rs`, we then replace our monster spawner with:

```rust
for room in map.rooms.iter().skip(1) {
    spawner::spawn_room(&mut gs.ecs, room);
}
```

If you `cargo run` the project now, it will have between 0 and 4 monsters per room. It can get a little hairy!

![Screenshot](./c9-s1.png)

# Health Potion Entities

We'll improve the chances of surviving for a bit by adding health potions to the game! We'll start off by adding some components to help define a potion. In `components.rs`:

```rust
#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount : i32
}
```

We of course need to register these in `main.rs`:
```rust
gs.ecs.register::<Item>();
gs.ecs.register::<Potion>();
```

In `spawner.rs`, we'll add a new function: `health_potion`:

```rust
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(Potion{ heal_amount: 8 })
        .build();
}
```

This is pretty straight-forward: we create an entity with a position, a renderable (we picked `¡` because it looks a bit like a potion, and my favorite game Dwarf Fortress uses it), a name, an `Item` component and a `Potion` component that specifies it heals 8 points of damage.

Now we can modify the spawner code to also have a chance to spawn between 0 and 2 items:

```rust
pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();
    let mut item_spawn_points : Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0 .. num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        health_potion(ecs, x as i32, y as i32);
    }
}
```

If you `cargo run` the project now, rooms now sometimes contain health potions. Tooltips and rendering "just work" - because they have the components required to use them.

![Screenshot](./c9-s2.png)

# Picking Up Items

Having potions exist is a great start, but it would be helpful to be able to pick them up! We'll create a new component in `components.rs` (and register it in `main.rs`!), to represent an item being in someone's backpack:

```rust
#[derive(Component, Debug)]
pub struct InBackpack {
    pub owner : Entity
}
```

Logically, picking up an item means removing its `Position` (it's no longer on the ground), and adding an `InBackpack` component with the owner listed (we're making it generic so that eventually monsters can have loot).

The next step is to add an input command to pick up an item. `g` is a popular key for this, so we'll go with that (we can always change it!). In `player.rs`, in the ever-growing `match` statement of inputs, we add:

```rust
VirtualKeyCode::G => get_item(&mut gs.ecs),
```

As you probably guessed, the next step is to implement `get_item`:
```rust
fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let names = ecs.read_storage::<Name>();
    let mut positions = ecs.write_storage::<Position>();
    let mut backpack = ecs.write_storage::<InBackpack>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();    

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.insert(0, "There is nothing here to pick up.".to_string()),
        Some(item) => {
            positions.remove(item);
            backpack.insert(item, InBackpack{ owner: *player_entity }).expect("Unable to insert component");
            gamelog.entries.insert(0, format!("You pick up the {}.", names.get(item).unwrap().name));
        }
    }
}
```

This obtains a bunch of references/accessors from the ECS, and iterates all items with a position. If it matches the player's position, `target_item` is set. Then, if `target_item` is none - we tell the player that there is nothing to pick up. If it isn't `None`, we remove its `Position` component, and add an `InBackpack` component - with the owner set to the player's entity.

If you `cargo run` the project now, you can press `g` anywhere to be told that there's nothing to get. If you are standing on a potion, it will vanish when you press `g`! It's in our backpack - but we haven't any way to *know* that other than the log entry.

# Listing your inventory

It's a good idea to be able to see your inventory list! This will be a game *mode* - that is, another state in which the game loop can find itself. So to start, we'll extend `RunMode` in `main.rs` to include it:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn, ShowInventory }
```

The `i` key is a popular choice for inventory (`b` is also popular!), so in `player.rs` we'll add the following to the player input code:
```rust
VirtualKeyCode::I => return RunState::ShowInventory,
```

In our `tick` function in `main.rs`, we'll add another matching:
```rust
RunState::ShowInventory => {
    if gui::show_inventory(self, ctx) == gui::ItemMenuResult::Cancel {
        newrunstate = RunState::AwaitingInput;
    }
}
```

That naturally leads to implementing `show_inventory`! In `gui.rs`, we add:
```rust
pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut j = 0;
    for (_pack, name) in (&backpack, &names).join().filter(|item| item.0.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as u8);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { ItemMenuResult::Cancel }
                _ => ItemMenuResult::NoResponse
            }
        }
    }
}
```

This starts out by using the `filter` feature of Rust iterators to count all items in your backpack. It then draws an appropriately sized box, and decorates it with a title and instructions. Next, it iterates all matching items and renders them in a menu format. Finally, it waits for keyboard input - and if you pressed `ESCAPE`, indicates that it is time to close the menu.

If you `cargo run` your project now, you can see items that you have collected:

![Screenshot](./c9-s3.png)

# Using Items

Now that we can display our inventory, lets make selecting an item actually *use* it. We'll extend the menu to return both an item entity and a result:
```rust
pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity );
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as u8);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => { 
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }  
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}
```

Our call to `show_inventory` in `main.rs` is now invalid, so we'll fix it up:
```rust
RunState::ShowInventory => {
    let result = gui::show_inventory(self, ctx);
    match result.0 {
        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
        gui::ItemMenuResult::NoResponse => {}
        gui::ItemMenuResult::Selected => {
            let item_entity = result.1.unwrap();
            let names = self.ecs.read_storage::<Name>();
            let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
            gamelog.entries.insert(0, format!("You try to use {}, but it isn't written yet", names.get(item_entity)         .unwrap().name));
            newrunstate = RunState::AwaitingInput;
        }
    }
}
```

If you try to use an item in your inventory now, you'll get a log entry that you try to use it, but we haven't written that bit of code yet. That's a start!

# Dropping Items

# Render order

# Wrap Up


**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-09-items)**

---

Copyright (C) 2019, Herbert Wolverson.

---