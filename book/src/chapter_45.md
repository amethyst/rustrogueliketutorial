# Data-Driven Design: Raw Files

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

If you've ever played *Dwarf Fortress*, one of its defining characteristics (under the hood) is the *raw file* system. Huge amounts of the game are detailed in the `raws`, and you can completely "mod" the game into something else. Other games, such as *Tome 4* take this to the extent of defining scripting engine files for *everything* - you can customize the game to your heart's content. Once implemented, `raws` turn your game into more of an *engine* - displaying/managing interactions with content written in the raw files. That isn't to say the engine is simple: it has to support everything that one specifies in the raw files!

This is called *data-driven design*: your game is defined by the data describing it, more than the actual engine mechanics. It has a few advantages:

* It makes it very easy to make changes; you don't have to dig through `spawner.rs` every time you want to change a goblin, or make a new variant such as a `cowardly goblin`. Instead, you edit the `raws` to include your new monster, add him/her/it to spawn, loot and faction tables, and the monster is now in your game! (Unless of course being *cowardly* requires new support code - in which case you write that, too).
* Data-driven design meshes beautifully with Entity Component Systems (ECS). The `raws` serve as a *template*, from which you build your entities by composing components until it matches your `raw` description.
* Data-driven design makes it easy for people to change the game you've created. For a tutorial such as this, this is pretty essential: I'd much rather you come out of this tutorial able to go forth and make your own game, rather than just re-hashing this one!

## A downside of web assembly

Web assembly doesn't make it easy to read files from your computer. That's why we started using the *embedding* system for assets; otherwise you have to make a bunch of hooks to read game data with JavaScript calls to download resources, obtain them as arrays of data, and pass the arrays into the Web Assembly module. There are probably better ways to do it than embedding everything, but until I find a good one (that also works in native code), we'll stick to embedding.

That gets rid of one advantage of data-driven design: you still have to recompile the game. So we'll make the embedding optional; if we *can* read a file from disk, we'll do so. In practice, this will mean that when you ship your game, you have to include the executable *and* the raw files - or embed them in the final build.

## Deciding upon a format for our Raw files

In some projects, I've used the scripting language `Lua` for this sort of thing. It's a great language, and having executable configuration is surprisingly useful (the configuration can include functions and helpers to build itself). That's overkill for this project. We already support JSON in our saving/loading of the game, so we'll use it for `Raws` also.

Taking a look at `spawner.rs` in the current game should give us some clues as to what to put into these files. Thanks to our use of components, there's already a lot of shared functionality we can build upon. For example, the definition for a *health potion* looks like this:

```rust
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Health Potion".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{ heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

In JSON, we might go for a representation like this (just an example):

```json
{
    "name" : "Healing Potion",
    "renderable": {
        "glyph" : "!",
        "fg" : "#FF00FF",
        "bg" : "#000000"
    },
    "consumable" : {
        "effects" : { "provides_healing" : "8" }
    }
}
```

## Making a raw files

Your package should be laid out like this:

```
|     Root folder
\ -   src (your source files)
```

At the root level, we'll make a new directory/folder called `raws`. So your tree should look like this:

```
|     Root folder
\ -   src (your source files)
\ -   raws
```

In this directory, create a new file: `spawns.json`. We'll temporarily put all of our definitions into one file; this will change later, but we want to get support for our data-driven ambitions bootstrapped. In this file, we'll put definitions for some of the entities we currently support in `spawner.rs`. We'll start with just a couple of items:

```json
{
{
"items" : [
    {
        "name" : "Health Potion",
        "renderable": {
            "glyph" : "!",
            "fg" : "#FF00FF",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { "provides_healing" : "8" }
        }
    },

    {
        "name" : "Magic Missile Scroll",
        "renderable": {
            "glyph" : ")",
            "fg" : "#00FFFF",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { 
                "ranged" : "6",
                "damage" : "20"
            }
        }
    }
]
}
```

If you aren't familiar with the JSON format, it's basically a JavaScript dump of data: 
* We wrap the file in `{` and `}` to denote the *object* we are loading. This will be our `Raws` object, eventually.
* Then we have an *array* called `Items` - which will hold our items.
* Each `Item` has a `name` - this maps directly to the `Name` component.
* Items may have a `renderable` structure, listing glyph, foreground and background colors.
* These items are `consumable`, and we list their effects in a "key/value map" - basically a `HashMap` like we've used before, a `Dictionary` in other languages.

We'll be adding a lot more to the spawns list eventually, but lets start by making these work.

## Embedding the Raw Files

In your project `src` directory, make a new directory: `src/raws`. We can reasonably expect this module to become quite large, so we'll support breaking it into smaller pieces from the beginning. To comply with Rust's requirements for building modules, make a new file called `mod.rs` in the new folder:

```rust
rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");
}
```

And at the top of `main.rs` add it to the list of modules we use:

```rust
pub mod raws;
```

In our initialization, add a call to `load_raws` after component initialization and before you start adding to `World`:

```rust
...
gs.ecs.register::<Door>();
gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

raws::load_raws();

gs.ecs.insert(Map::new(1, 64, 64));
...
```

The `spawns.json` file will now be embedded into your executable, courtesy of RLTK's embedding system.

## Parsing the raw files

This is the hard part: we need a way to *read* the JSON file we've created, and to turn it into a format we can use within Rust. Going back to `mod.rs`, we can expand the function to load our embedded data as a string:

```rust
// Retrieve the raw data as an array of u8 (8-bit unsigned chars)
let raw_data = rltk::embedding::EMBED
    .lock()
    .unwrap()
    .get_resource("../../raws/spawns.json".to_string())
    .unwrap();
let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
```

This will panic (crash) if it isn't able to find the resource, or if it is unable to parse it as a regular string (Rust likes UTF-8 Unicode encoding, so we'll go with it. It lets us include extended glyphs, which we can parse via RLTK's `to_cp437` function - so it works out nicely!).

Now we need to actually *parse* the JSON into something usable. Just like our `saveload.rs` system, we can do this with Serde. For now, we'll just dump the results to the console so we can see that it *did* something:

```rust
let decoder : Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");
println!("{:?}", decoder);
```

(See the cryptic `{:?}`? That's a way to print *debug* information about a structure). This will fail to compile, because we haven't actually implemented `Raws` - the type it is looking for. 

For clarity, we'll put the classes that actually handle the data in their own file, `raws/item_structs.rs`. Here's the file:

```rust
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>
}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg : String,
    pub bg : String,
    pub order: i32
}

#[derive(Deserialize, Debug)]
pub struct Consumable {
    pub effects : HashMap<String, String>
}
```

At the top of the file, make sure to include `use serde::{Deserialize};` and `use std::collections::HashMap;` to include the types we need. Also notice that we have included `Debug` in the derived types list. This allows Rust to print a debug copy of the struct, so we can see what the code did. Notice also that a lot of things are an `Option`. This way, the parsing will work if an item *doesn't* have that entry. It will make reading them a little more complicated later on, but we can live with that!

If you `cargo run` the project now, ignore the game window - watch the console. You'll see the following:

```
Raws { items: [Item { name: "Healing Potion", renderable: Some(Renderable { glyph: "!", fg: "#FF00FF", bg: "#000000" }), consumable: Some(Consumable { effects: {"provides_healing": "8"} }) }, Item { name: "Magic Missile Scroll", renderable: Some(Renderable { glyph: ")", fg: "#00FFFF", bg: "#000000" 
}), consumable: Some(Consumable { effects: {"damage": "20", "ranged": "6"} }) }] }
```

That's *super* ugly and horribly formatted, but you can see that it contains the data we entered!

## Storing and indexing our raw item data

Having this (largely text) data is great, but it doesn't really help us until it can directly relate to spawning entities. We're also discarding the data as soon as we've loaded it! 

We want to create a structure to hold all of our raw data, and provide useful services such as spawning an object entirely from the data in the `raws`. We'll make a new file, `raws/rawmaster.rs`:

```rust
use std::collections::HashMap;
use specs::prelude::*;
use crate::components::*;
use super::{Raws};

pub struct RawMaster {
    raws : Raws,
    item_index : HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{ items: Vec::new() },
            item_index : HashMap::new()
        }
    }

    pub fn load(&mut self, raws : Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i,item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }    
}
```

That's very straightforward, and well within what we've learned of Rust so far: we make a structure called `RawMaster`, it gets a private copy of the `Raws` data and a `HashMap` storing item names and their index inside `Raws.items`. The `empty` constructor does just that: it makes a completely empty version of the `RawMaster` structure. `load` takes the de-serialized `Raws` structure, stores it, and indexes the items by name and location in the `items` array.

## Accessing Raw Data From Anywhere

This is one of those times that it would be nice if Rust didn't make global variables difficult to use; we want exactly one copy of the `RawMaster` data, and we'd like to be able to *read* it from anywhere. You *can* accomplish that with a bunch of `unsafe` code, but we'll be good "Rustaceans" and use a popular method: the `lazy_static`. This functionality isn't part of the language itself, so we need to add a crate to `cargo.toml`. Add the following line to your `[dependencies]` in the file:

```toml
lazy_static = "1.4.0"
```

Now we do a bit of a dance to make the global safely available from everywhere. At the end of `main.rs`'s import section, add:

```rust
#[macro_use]
extern crate lazy_static;
```

This is similar to what we've done for other macros: it tells Rust that we'd like to import the macros from the crate `lazy_static`. In `mod.rs`, declare the following:

```rust
mod rawmaster;
pub use rawmaster::*;
use std::sync::Mutex;
```

Also:

```rust
lazy_static! {
    pub static ref RAWS : Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}
```

The `lazy_static!` macro does a bunch of hard work for us to make this safe. The interesting part is that we still have to use a `Mutex`. Mutexes are a construct that ensure that no more than one thread at a time can write to a structure. You access a Mutex by calling `lock` - it is now yours until the lock goes out of scope. So in our `load_raws` function, we need to populate it:

```rust
// Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
        .lock()
        .unwrap()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder : Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
```

You'll notice that RLTK's `embedding` system is quietly using a `lazy_static` itself - that's what the `lock` and `unwrap` code is for: it manages the Mutex. So for our `RAWS` global, we `lock` it (retrieving a scoped lock), `unwrap` that lock (to allow us to access the contents), and call the `load` function we wrote earlier. Quite a mouthful, but now we can safely share the `RAWS` data without having to worry about threading problems. Once loaded, we'll probably never write to it again - and Mutex locks for reading are pretty much instantaneous when you don't have lots of threads running.

## Spawning items from the RAWS

In `rawmaster.rs`, we'll make a new function:

```rust
pub fn spawn_named_item(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        match pos {
            SpawnType::AtPosition{x,y} => {
                eb = eb.with(Position{ x, y });
            }
        }

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(crate::components::Renderable{  
                glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
                fg : rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
                bg : rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
                render_order : renderable.order
            });
        }

        eb = eb.with(Name{ name : item_template.name.clone() });

        eb = eb.with(crate::components::Item{});

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(crate::components::Consumable{});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => { 
                        eb = eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }) 
                    }
                    "ranged" => { eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }) },
                    "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
                    _ => {
                        println!("Warning: consumable effect {} not implemented.", effect_name);
                    }
                }
            }
        }

        return Some(eb.build());
    }
    None
}
```

It's a long function, but it's actually very straightforward - and uses patterns we've encountered plenty of times before. It does the following:

1. It looks to see if the `key` we've passed exists in the `item_index`. If it doesn't, it returns `None` - it didn't do anything.
2. If the `key` does exist, then it adds a `Name` component to the entity - with the name from the raw file.
3. If `Renderable` exists in the item definition, it creates a component of type `Renderable`.
4. If `Consumable` exists in the item definition, it makes a new consumable. It iterates through all of the keys/values inside the `effect` dictionary, adding effect components as needed.

Now you can open `spawner.rs` and modify `spawn_entity`:

```rust
pub fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let item_result = spawn_named_item(&RAWS.lock().unwrap(), ecs.create_entity(), &spawn.1, SpawnType::AtPosition{ x, y});
    if item_result.is_some() {
        return;
    }

    match spawn.1.as_ref() {
        "Goblin" => goblin(ecs, x, y),
        "Orc" => orc(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Dagger" => dagger(ecs, x, y),
        "Shield" => shield(ecs, x, y),
        "Longsword" => longsword(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        "Rations" => rations(ecs, x, y),
        "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
        "Bear Trap" => bear_trap(ecs, x, y),
        "Door" => door(ecs, x, y),
        _ => {}
    }
}
```

Note that we've deleted the items we've added into `spawns.json`. We can also delete the associated functions. `spawner.rs` will be really small when we're done! So the magic here is that it calls `spawn_named_item`, using a rather ugly `&RAWS.lock().unwrap()` to obtain safe access to our `RAWS` global variable. If it matched a key, it will return `Some(Entity)` - otherwise, we get `None`. So we check if `item_result.is_some()` and return if we succeeded in spawning something from the data. Otherwise, we use the new code.

You'll also want to add a `raws::*` to the list of items imported from `super`.

If you `cargo run` now, the game runs as before - including health potions and magic missile scrolls.

## Adding the rest of the consumables

We'll go ahead and get the rest of the consumables into `spawns.json`:

```json
...
    {
        "name" : "Fireball Scroll",
        "renderable": {
            "glyph" : ")",
            "fg" : "#FFA500",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { 
                "ranged" : "6",
                "damage" : "20",
                "area_of_effect" : "3"
            }
        }
    },

    {
        "name" : "Confusion Scroll",
        "renderable": {
            "glyph" : ")",
            "fg" : "#FFAAAA",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { 
                "ranged" : "6",
                "damage" : "20",
                "confusion" : "4"
            }
        }
    },

    {
        "name" : "Magic Mapping Scroll",
        "renderable": {
            "glyph" : ")",
            "fg" : "#AAAAFF",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { 
                "magic_mapping" : ""
            }
        }
    },

    {
        "name" : "Rations",
        "renderable": {
            "glyph" : "%",
            "fg" : "#00FF00",
            "bg" : "#000000",
            "order" : 2
        },
        "consumable" : {
            "effects" : { 
                "food" : ""
            }
        }
    }
]
}
```

We'll put their effects into `rawmaster.rs`'s `spawn_named_item` function:

```rust
if let Some(consumable) = &item_template.consumable {
    eb = eb.with(crate::components::Consumable{});
    for effect in consumable.effects.iter() {
        let effect_name = effect.0.as_str();
        match effect_name {
            "provides_healing" => { 
                eb = eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }) 
            }
            "ranged" => { eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }) },
            "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
            "area_of_effect" => { eb = eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }) }
            "confusion" => { eb = eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() }) }
            "magic_mapping" => { eb = eb.with(MagicMapper{}) }
            "food" => { eb = eb.with(ProvidesFood{}) }
            _ => {
                println!("Warning: consumable effect {} not implemented.", effect_name);
            }
        }
    }
}
```

You can now delete the fireball, magic mapping and confusion scrolls from `spawner.rs`! Run the game, and you have access to these items. Hopefully, this is starting to illustrate the power of linking a data file to your component creation.

## Adding the remaining items

We'll make a few more JSON entries in `spawns.json` to cover the various other items we have remaining:

```json
{
    "name" : "Dagger",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAAA",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "power_bonus" : 2
    }
},

{
    "name" : "Longsword",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "power_bonus" : 4
    }
},

{
    "name" : "Shield",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00AAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "shield" : {
        "defense_bonus" : 1
    }
},

{
    "name" : "Tower Shield",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00FFFF",
        "bg" : "#000000",
        "order" : 2
    },
    "shield" : {
        "defense_bonus" : 3
    }
}
```

There are two new fields here! `shield` and `weapon`. We need to expand our `item_structs.rs` to handle them:

```rust
#[derive(Deserialize, Debug)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub shield : Option<Shield>
}

...

#[derive(Deserialize, Debug)]
pub struct Weapon {
    pub range: String,
    pub power_bonus: i32
}

#[derive(Deserialize, Debug)]
pub struct Shield {
    pub defense_bonus: i32
}
```

We'll also have to teach our `spawn_named_item` function (in `rawmaster.rs`) to use this data:

```rust
if let Some(weapon) = &item_template.weapon {
    eb = eb.with(Equippable{ slot: EquipmentSlot::Melee });
    eb = eb.with(MeleePowerBonus{ power : weapon.power_bonus });
}

if let Some(shield) = &item_template.shield {
    eb = eb.with(Equippable{ slot: EquipmentSlot::Shield });
    eb = eb.with(DefenseBonus{ defense: shield.defense_bonus });
}
```

You can now delete these items from `spawner.rs` as well, and they still spawn in game - as before.

## Now for the monsters!

We'll add a new array to `spawns.json` to handle monsters. We're calling it "mobs" - this is slang from many games for "movable object", but it has come to mean things that move around and fight you in common parlance:

```json
"mobs" : [
    {
        "name" : "Orc",
        "renderable": {
            "glyph" : "o",
            "fg" : "#FF0000",
            "bg" : "#000000",
            "order" : 1
        },
        "blocks_tile" : true,
        "stats" : {
            "max_hp" : 16,
            "hp" : 16,
            "defense" : 1,
            "power" : 4
        },
        "vision_range" : 8
    },

    {
        "name" : "Goblin",
        "renderable": {
            "glyph" : "g",
            "fg" : "#FF0000",
            "bg" : "#000000",
            "order" : 1
        },
        "blocks_tile" : true,
        "stats" : {
            "max_hp" : 8,
            "hp" : 8,
            "defense" : 1,
            "power" : 3
        },
        "vision_range" : 8
    }
]
```

You'll notice that we're fixing a minor issue from before: orcs and goblins are no longer identical in stats! Otherwise, this should make sense: the stats we set in `spawner.rs` are instead set in the JSON file. We need to create a new file, `raws/mob_structs.rs`:

```rust
use serde::{Deserialize};
use super::{Renderable};

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub blocks_tile : bool,
    pub stats : MobStats,
    pub vision_range : i32
}

#[derive(Deserialize, Debug)]
pub struct MobStats {
    pub max_hp : i32,
    pub hp : i32,
    pub power : i32,
    pub defense : i32
}
```

We'll also modify `Raws` (currently in `item_structs.rs`). We'll move it to `mod.rs`, since it is shared with other modules and edit it:

```rust
#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>,
    pub mobs : Vec<Mob>
}
```

We also need to modify `rawmaster.rs` to add an empty `mobs` list to the constructor:

```rust
impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{ items: Vec::new(), mobs: Vec::new() },
            item_index : HashMap::new()
        }
    }
    ...
```

We'll also modify `RawMaster` to index our mobs:

```rust
pub struct RawMaster {
    raws : Raws,
    item_index : HashMap<String, usize>,
    mob_index : HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{ items: Vec::new(), mobs: Vec::new() },
            item_index : HashMap::new(),
            mob_index : HashMap::new()
        }
    }

    pub fn load(&mut self, raws : Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i,item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
        for (i,mob) in self.raws.mobs.iter().enumerate() {
            self.mob_index.insert(mob.name.clone(), i);
        }
    }    
}
```

We're going to want to build a `spawn_named_mob` function, but first lets create some helpers so we're sharing functionality with `spawn_named_item` - avoid repeating ourselves. The first is pretty straightforward:

```rust
fn spawn_position(pos : SpawnType, new_entity : EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition{x,y} => {
            eb = eb.with(Position{ x, y });
        }
    }

    eb
}
```

When we add more `SpawnType` entries, this function will necessarily expand to include them - so it's *great* that it's a function. We can replace the same code in `spawn_named_item` with a single call to this function:

```rust
// Spawn in the specified location
eb = spawn_position(pos, eb);
```

Let's also break out handling of `Renderable` data. This was more difficult; I had a *terrible* time getting Rust's lifetime checker to work with a system that actually added it to the `EntityBuilder`. I finally settled on a function that returns the component for the caller to add:

```rust
fn get_renderable_component(renderable : &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable{  
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg : rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg : rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order : renderable.order
    }
}
```

That still cleans up the call in `spawn_named_item`:

```rust
// Renderable
if let Some(renderable) = &item_template.renderable {
    eb = eb.with(get_renderable_component(renderable));
}
```

Alright - so with that in hand, we can go ahead and make `spawn_named_mob`:

```rust
pub fn spawn_named_mob(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut eb = new_entity;

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name{ name : mob_template.name.clone() });

        eb = eb.with(Monster{});
        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile{});
        }
        eb = eb.with(CombatStats{
            max_hp : mob_template.stats.max_hp,
            hp : mob_template.stats.hp,
            power : mob_template.stats.power,
            defense : mob_template.stats.defense
        });
        eb = eb.with(Viewshed{ visible_tiles : Vec::new(), range: mob_template.vision_range, dirty: true });

        return Some(eb.build());
    }
    None
}
```

There's really nothing we haven't already covered in this function: we simply apply a renderable, position, name using the same code as before - and then check `blocks_tile` to see if we should add a `BlocksTile` component, and copy the stats into a `CombatStats` component. We also setup a `Viewshed` component with `vision_range` range.

Before we update `spawner.rs` again, lets introduce a master spawning method - `spawn_named_entity`. The reasoning behind this is that the spawn system doesn't actually know (or care) if an entity is an item, mob, or anything else. Rather than push a lot of `if` checks into it, we'll provide a single interface:

```rust
pub fn spawn_named_entity(raws: &RawMaster, new_entity : EntityBuilder, key : &str, pos : SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, new_entity, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, new_entity, key, pos);
    }

    None
}
```

So over in `spawner.rs` we can use the generic spawner now:

```rust
let spawn_result = spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), &spawn.1, SpawnType::AtPosition{ x, y});
if spawn_result.is_some() {
    return;
}
```

We can also go ahead and delete the references to Orcs, Goblins and Monsters! We're nearly there - you can get your data-driven monsters now.

## Doors and Traps

There are two remaining hard-coded entities. These have been left separate because they aren't really the same as the other types: they are what I call "props" - level features. You can't pick them up, but they are an integral part of the level. So in `spawns.json`, we'll go ahead and define some props:

```json
```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-45-raws1)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-45-raws1)
---

Copyright (C) 2019, Herbert Wolverson.

---