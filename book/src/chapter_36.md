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

## Sharing the Random Number Generator

Currently, all of the map builders make their own random number generator. If you wanted to limit the *random seed* to make things predictable (this is planned as a future chapter topic), this wouldn't work at all: your levels would completely ignore the seed! Lets extend our `random_builder` and builder modules to use the global RNG, taken from the ECS. We'll first update our `MapBuilder` trait to indicate the new signature:

```rust
pub trait MapBuilder {
    fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs : &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}
```

That will cause an error to appear in `main.rs`, specifically in the `generate_world_map` function. The `build_map` function now *requires* that a Random Number Generator be passed to it. We'll update the function to provide one:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    ...
    let mut builder = map_builders::random_builder(new_depth);
    let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
    builder.build_map(&mut rng);
    std::mem::forget(rng);
    ...
```

We retrieve `rng` from the ECS World, like we have many times before. Then we pass a mutable reference to `build_map` (it has to be mutable because retrieving a random number causes the RNG to change internal state). Then we call a new function: `std::mem::forget`. This tells the borrow checker that we're done with a borrow, and it can safely ignore it from now on. This prevents errors with later code not liking that we are still borrowing the RNG, even though we're done with it for now.

Now we have to update *every single builder* to use the new RNG. I've made all the changes in the [source](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers); for brevity, we'll only cover a couple of them in the chapter text. In `simple_map.rs`, we change the `build_map` to look like this:

```rust
fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator)  {
    self.rooms_and_corridors(rng);
}
```

We also update the top of `rooms_and_corridors` to use the passed RNG:

```rust
fn rooms_and_corridors(&mut self, rng : &mut rltk::RandomNumberGenerator) {
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    for _i in 0..MAX_ROOMS {
```

Notice that we've *deleted* the line that creates a new RNG - and are using the one we pass in, instead.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-36-layers/)
---

Copyright (C) 2019, Herbert Wolverson.

---