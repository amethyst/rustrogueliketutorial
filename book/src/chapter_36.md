# Layering/Builder Chaining

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The last few chapters have introduced an important concept in procedural generation: chained builders. We're happily building a map, calling Waveform Collapse to mutate the map, calling our `PrefabBuilder` to change it again, and so on. This chapter will formalize this process a bit, expand upon it, and leave you with a framework that lets you *clearly* build new maps by chaining concepts together.

## Cleaning up the boxes

I'm currently moving house, so boxes are a bit of a sore subject! It's becoming a sore subject in the `map_builders` module, too. If you look at `map_builders/mod.rs`, the `random_builder` function is *full* of convoluted `Box::new(BspDungeonBuilder::new(new_depth));` statements. All our builders live in a box, but we're building the box at the *module* level - leading to a lot of repetitive typing.

Cleaning this up will lead to code that is a *lot* easier to read. Fortunately, it's quite easy to do. Open up `map_builders/bsp_dungeon.rs` and take a look at the `new` function:

```rust
pub fn new(new_depth : i32) -> BspDungeonBuilder {
    BspDungeonBuilder{
        map : Map::new(new_depth),
        starting_position : Position{ x: 0, y : 0 },
        depth : new_depth,
        rooms: Vec::new(),
        history: Vec::new(),
        rects: Vec::new(),
        spawn_list: Vec::new()
    }
}
```

We can change it to return a ready-boxed version of the builder:

```rust
pub fn new(new_depth : i32) -> Box<BspDungeonBuilder> {
    Box::new(BspDungeonBuilder{
        map : Map::new(new_depth),
        starting_position : Position{ x: 0, y : 0 },
        depth : new_depth,
        rooms: Vec::new(),
        history: Vec::new(),
        rects: Vec::new(),
        spawn_list: Vec::new()
    })
}
```

Now we change `random_builder` in `map_builders/mod.rs` to no longer need to add a box to this type:

```rust
...
match builder {
    1 => { result = BspDungeonBuilder::new(new_depth); }
    ...
```

Isn't that easier to read? We can apply the same change to *all* the builders. I won't bore you by listing them out one at a time - they really are quite simple to change. You can always check the [source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers) to see what I've done if you need help.

Like all refactors, it's a good idea to `cargo run` your project to make sure that it still works. *Nothing* should have changed in your program's output.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-36-layers/)
---

Copyright (C) 2019, Herbert Wolverson.

---