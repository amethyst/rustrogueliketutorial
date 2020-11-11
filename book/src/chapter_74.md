# One Night in the City

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

[![Hands-On Rust](./beta-webBanner.jpg)](https://pragprog.com/titles/hwrust/hands-on-rust/)

---

The next level of the game is a dark elven city. The design document is a bit sparse on details, but here's what we know:

* It eventually leads to a portal to the Abyss.
* Dark elves are infighty, back-stabbing maniacs and should behave as such.
* Dark elven cities are surprisingly city-like, just deep underground.
* Lighting will be important.

## Generating a basic city

The `level_builder` function in `map_builder/mod.rs` controls which map algorithm is called for a given level. Add a placeholder entry for a new map type:

```rust
pub fn level_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, width, height),
        2 => forest_builder(new_depth, width, height),
        3 => limestone_cavern_builder(new_depth, width, height),
        4 => limestone_deep_cavern_builder(new_depth, width, height),
        5 => limestone_transition_builder(new_depth, width, height),
        6 => dwarf_fort_builder(new_depth, width, height),
        7 => mushroom_entrance(new_depth, width, height),
        8 => mushroom_builder(new_depth, width, height),
        9 => mushroom_exit(new_depth, width, height),
        10 => dark_elf_city(new_depth, width, height),
        _ => random_builder(new_depth, width, height)
    }
}
```

At the top of the same file, add imports for a new builder module:

```rust
mod dark_elves;
use dark_elves::*;
```

And create the new `map_builders/dark_elves.rs` file with a placeholder builder in it:

```rust
use super::{BuilderChain, XStart, YStart, AreaStartingPosition, 
    CullUnreachable, VoronoiSpawning,
    AreaEndingPosition, XEnd, YEnd, BspInteriorBuilder };

pub fn dark_elf_city(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    println!("Dark elf builder");
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elven City");
    chain.start_with(BspInteriorBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::RIGHT, YStart::CENTER));
    chain.with(AreaEndingPosition::new(XEnd::LEFT, YEnd::CENTER));
    chain.with(VoronoiSpawning::new());
    chain
}
```

That makes a not-at-all city like map (just a bsp interiors map) - but it's a good start. I chose this as the base builder because it doesn't waste any space. I like to imagine that the city is a big warren of interconnected rooms, with the poorer-elf housing in the dangerous spot (at the top). So we'll populate this level with relatively "normal" dark elves, and their slaves.

## Adding some dark elves

If we just wanted to put dark elves everywhere, it would be as simple as adding one line to `spawns.json` in the `spawn_table` section:

```json
{ "name" : "Dark Elf", "weight": 10, "min_depth": 10, "max_depth": 11 }
```

That's boring, so let's not do that. Our dark elves are split between *Clan Arbat*, *Clan Barbo*, and *Clan Cirro* (A, B, C, get it?). Thanks to the Abyssal influence of the Amulet of YALA, they are wrought with terrible infighting and war! We'll worry about differentiating the clans in a moment, for now lets make some entries to provide three groups of dark elves who hate one another.

In the `factions` section of `spawns.json`, create three new factions:

```json
{ "name" : "DarkElfA", "responses" : { "Default" : "attack", "DarkElfA" : "ignore", "DarkElfB" : "attack", "DarkElfC" : "attack" } },
{ "name" : "DarkElfB", "responses" : { "Default" : "attack", "DarkElfB" : "ignore", "DarkElfA" : "attack", "DarkElfC" : "attack" } },
{ "name" : "DarkElfC", "responses" : { "Default" : "attack", "DarkElfC" : "ignore", "DarkElfA" : "attack", "DarkElfB" : "attack" } }
```

Notice how they ignore their own clan, and attack the others. That's the key to making a warzone! Our factions system already supports warring groups - we've just not used it extensively. Now find the `mobs` section, and duplicate the "Dark Elf" three times - once for each faction:

```json
{
    "name" : "Arbat Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Hand Crossbow", "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfA",
    "gold" : "3d6",
    "level" : 6
},

{
    "name" : "Barbo Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Hand Crossbow", "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfB",
    "gold" : "3d6",
    "level" : 6
},

{
    "name" : "Cirro Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Hand Crossbow", "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfC",
    "gold" : "3d6",
    "level" : 6
},
```

In the spawn table, we want them to appear on level 10:

```json
{ "name" : "Arbat Dark Elf", "weight": 10, "min_depth": 10, "max_depth": 11 },
{ "name" : "Barbo Dark Elf", "weight": 10, "min_depth": 10, "max_depth": 11 },
{ "name" : "Cirro Dark Elf", "weight": 10, "min_depth": 10, "max_depth": 11 }
```

If you `cargo run` now, and cheat your way down to depth 10 (I recommend god mode, and teleport) - you find yourself in the midst of a warzone between three clans. There's combat everywhere, and they only pause killing one another long enough to murder the player. There's a lovely amount of mayhem - the gods of Chaos would be proud.

## Clan Differentiation

It's kinda boring having all of the clans be identical. The basic "Dark Elf" can stay the same, but lets add a bit of flavor to make the clans *feel* differentiated.

### Clan Arbat

We'll start by making Arbat a different color - a lighter red. Replace the "fg" attribute of their Dark Elves with `#FFAAAA` - a pinkish color. We'll take away their crossbows, also. They are a melee-oriented clan. Replace `Scimitar` with `Scimitar +1`. The modified `Arbat Dark Elf` looks like this:

```json
{
    "name" : "Arbat Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FFAAAA",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Scimitar +1", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfA",
    "gold" : "3d6",
    "level" : 6
},
```

Let's also give them leaders - tougher fighters:

```json
{
    "name" : "Arbat Dark Elf Leader",
    "renderable": {
        "glyph" : "E",
        "fg" : "#FFAAAA",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Scimitar +2", "Buckler +1", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfA",
    "gold" : "3d6",
    "level" : 7
},
```

They also deserve some orc slaves:

```json
{
    "name" : "Arbat Orc Slave",
    "renderable": {
        "glyph" : "o",
        "fg" : "#FFAAAA",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "static",
    "attributes" : {},
    "faction" : "DarkElfA",
    "gold" : "1d8"
},
```

Finally, put these into the spawn table:

```json
{ "name" : "Arbat Dark Elf", "weight": 10, "min_depth": 10, "max_depth": 11 },
{ "name" : "Arbat Dark Elf Leader", "weight": 7, "min_depth": 10, "max_depth": 11 },
{ "name" : "Arbat Orc Slave", "weight": 14, "min_depth": 10, "max_depth": 11 },
```

They are probably going to regret their melee focus, but we aren't too concerned for their health!

### Clan Barbo

Conversely, we'll make Barbo quite missile oriented - and a little more scarce, because that's super-dangerous. We'll also give them a dagger instead of a scimitar, and change their color to orange:

```json
{
    "name" : "Barbo Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF9900",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Hand Crossbow +1", "Dagger", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfB",
    "gold" : "3d6",
    "level" : 6
},
```

They also get some slaves - this time goblins, with a missile weapon:

```json
{
    "name" : "Barbo Goblin Archer",
    "renderable": {
        "glyph" : "g",
        "fg" : "#FF9900",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "static",
    "attributes" : {},
    "faction" : "Cave Goblins",
    "gold" : "1d6",
    "equipped" : [ "Shortbow", "Leather Armor", "Leather Boots" ]
},
```

Finally, update the spawns table to include them:

```json
{ "name" : "Barbo Dark Elf", "weight": 9, "min_depth": 10, "max_depth": 11 },
{ "name" : "Barbo Goblin Archer", "weight": 13, "min_depth": 10, "max_depth": 11 },
```

### Clan Cirro

We're going to make Cirro powerful and rare. The basic Cirro Dark Elf looks like this:

```json
{
    "name" : "Cirro Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF00FF",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Hand Crossbow", "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
    "faction" : "DarkElfC",
    "gold" : "3d6",
    "level" : 7
},
```

We'll also give them leaders - priestesses who can web you:

```json
{
        "name" : "Cirro Dark Priestess",
        "renderable": {
            "glyph" : "E",
            "fg" : "#FF00FF",
            "bg" : "#000000",
            "order" : 1
        },
        "blocks_tile" : true,
        "vision_range" : 8,
        "movement" : "random_waypoint",
        "attributes" : {},
        "equipped" : [ "Hand Crossbow", "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
        "faction" : "DarkElfC",
        "gold" : "3d6",
        "level" : 8,
        "abilities" : [
            { "spell" : "Web", "chance" : 0.2, "range" : 6.0, "min_range" : 3.0 }
        ]
    },
```

Instead of slaves, we'll give them spiders:

```json
{
    "name" : "Cirro Spider",
    "level" : 3,
    "attributes" : {},
    "renderable": {
        "glyph" : "s",
        "fg" : "#FF00FF",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 6,
    "movement" : "static",
    "natural" : {
        "armor_class" : 12,
        "attacks" : [
            { "name" : "bite", "hit_bonus" : 1, "damage" : "1d12" }
        ]
    },
    "abilities" : [
        { "spell" : "Web", "chance" : 0.2, "range" : 6.0, "min_range" : 3.0 }
    ],
    "faction" : "DarkElfC"
},
```

This also requires a spawn table update:

```json
{ "name" : "Cirro Dark Elf", "weight": 7, "min_depth": 10, "max_depth": 11 },
{ "name" : "Cirro Dark Priestess", "weight": 6, "min_depth": 10, "max_depth": 11 },
{ "name" : "Cirro Spider", "weight": 10, "min_depth": 10, "max_depth": 11 }
```

If you `cargo run` the project now, you'll find that the dark elves are murdering one another - and there's a good level of variety present.

## Wrap-Up

This has been a short chapter: because most of the pre-requisites were already written. That's a good sign for the engine as a whole: we can now build a very different style of level without much in the way of new code. In the next chapter, we'll advance further into the dark elven city - trying to make more of an open city level. The mayhem will continue!

---

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-74-darkcity)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-74-darkcity)
---

Copyright (C) 2019, Herbert Wolverson.

---