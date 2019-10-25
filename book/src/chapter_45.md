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
"items" : [
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
    },

    {
        "name" : "Magic Missile Scroll",
        "renderable": {
            "glyph" : ")",
            "fg" : "#00FFFF",
            "bg" : "#000000"
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

Having this (largely text) data is great, but it doesn't really help us until it can directly relate to spawning entities. We're also discarding the data as soon as we've loaded it! We want to create a structure to hold all of our raw data, and provide useful services such as spawning an object entirely from the data in the `raws`. We'll make a new file, `raws/rawmaster.rs`:

Add lazy static to cargo...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-45-raws1)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-45-raws1)
---

Copyright (C) 2019, Herbert Wolverson.

---