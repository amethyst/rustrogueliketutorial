# One Night in the City

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

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



---

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-74-darkcity)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-74-darkcity)
---

Copyright (C) 2019, Herbert Wolverson.

---