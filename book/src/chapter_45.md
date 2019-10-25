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

In some projects, I've used the scripting language `Lua` for this sort of thing. It's a great language, and having executable configuration is surprisingly useful (the configuration can include functions and helpers to build itself). That's overkill for this project. Since Rust uses [Tom's Obvious, Minimal Language - TOML](https://github.com/toml-lang/toml) natively, we'll also use it to define our various game elements.

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

In TOML, we might go for a representation like this (just an example):

```toml
[health potion]
name = "Potion of Healing"
glyph = "¡"
foreground = "#FF00FF"
background = "#000000"
render_order = 2
type = "item-consumable"
consume_effects = [  
    { effect = "self-healing", amount: 8 }
]
```

## Supporting TOML in your program

Despite using TOML as part of the build process, Rust doesn't actually natively support it! It relies upon a `crate` - just like the other crates you've been using in your package to date. Open up your `cargo.toml` file, and we'll add `TOML` support:

```toml
[dependencies]
rltk = { git = "https://github.com/thebracket/rltk_rs", features = ["serialization"] }
specs = { version = "0.15.0", features = ["serde"] }
specs-derive = "0.4.0"
serde= { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.5.3"
```

Since the `toml` crate already includes support for Serde (our serialization system), that's all you need to be able to use it.

## Making some raw files

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

In this directory, create a new file: `spawns.toml`. We'll temporarily put all of our definitions into one file; this will change later, but we want to get support for our data-driven ambitions bootstrapped. In this file, we'll put definitions for some of the entities we currently support in `spawner.rs`:

```toml

```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-45-raws1)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-45-raws1)
---

Copyright (C) 2019, Herbert Wolverson.

---