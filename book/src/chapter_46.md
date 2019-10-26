# Data-Driven Spawn Tables

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the previous chapter, we moved spawning to be data-driven: you define your monsters, items and props in a JSON data file - and the spawn function becomes a parser that builds components based on your definitions. That gets you half-way to a data-driven world.

If you look at the ever-shrinking `spawner.rs` file, we have a hard-coded table for handling spawning:

```rust
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 5)
}
```

It's served us well for all these chapters, but sadly it's time to put it out to pasture. We'd like to be able to specify the spawn table in our JSON data - that way, we can add new entities to the data file and spawn list, and they appear in the game with no additional Rust coding (unless they need new features, in which case it's time to extend the engine).

## A JSON-based spawn table



**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-46-raws2)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-46-raws2)
---

Copyright (C) 2019, Herbert Wolverson.

---