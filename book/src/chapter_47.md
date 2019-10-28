# Making the starting town

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

## What is the town for?

Back in the [Design Document](./chapter_44.md) we decided: *The game starts in town. In town, there are only minimal enemies (pickpockets, thugs). You start in the to-be-named pub (tavern), armed only with a meager purse, minimal starting equipment, a stein of beer, a dried sausage, a backpack and a hangover. Town lets you visit various vendors.*

From a development point of view, this tells us a few things:

* The town has a *story* aspect, in that you start there and it ground the story - giving a starting point, a destiny (in this case a drunken promise to save the world). So the town implies a certain *cozy* starting point, implies some communication to help you understand *why* you are embarking on the life of an adventurer, and so on.
* The town has vendors. That won't make sense at this point, because we don't have a value/currency system - but we know that we need somewhere to put them.
* The town has a tavern/inn/pub - it's a starting location, but it's obviously important enough that it needs to *do* something!
* Elsewhere in the design document, we mention that you can *town portal* back to the settlement. This again implies a certain coziness/safety, and also implies that doing so is *useful* - so the services offered by the town need to retain their utility throughout the game.
* Finally, the town is the winning condition: once you've grabbed the Amulet of Yala - getting back to town lets you save the world. That implies that the town should have some sort of holy structure to which you have to return the amulet.
* The town is the first thing that new players will encounter - so it has to look alive and somewhat slick, or players will just close the window and try something else. It may also serve as a location for some tutorials.

This sort of discussion is essential to game design; you don't want to implement something just because you can (in most cases; big open world games relax that a bit). The town has a *purpose*, and that purpose guides its *design*.

## So what do we have to include in the town?

So that discussion lets us determine that the town must include:

* One or more merchants. We're not implementing the sale of goods yet, but they need a place to operate.
* Some friendly/neutral NPCs for color.
* A temple.
* A tavern.
* A place that town portals arrive.
* A way out to begin your adventure.

We can also think a little bit about what makes a town:

* There's generally a communication route (land or sea), otherwise the town won't prosper.
* Frequently, there's a market (surrounding villages use towns for commerce).
* There's almost certainly either a river or a deep natural water source.
* Towns typically have authority figures, visible at least as Guards or Watch.
* Towns also generally have a shady side.

## How do we want to generate our town?

We could go for a prefabricated town. This has the upside that the town can be tweaked until it's *just right*, and plays smoothly. It has the downside that getting out of the town becomes a purely mechanical step after the first couple of play-throughs ("runs"); look at Joppa in Caves of Qud - it became little more than a "grab the chest content, talk to these guys, and off you go" speed-bump start to an amazing game.

So - we want a procedurally generated town, but we want to keep it functional - and make it pretty. Not much to ask!

## Making some new tile types

From the above, it sounds like we are going to need some new tiles. The ones that spring to mind for a town are roads, grass, water (both deep and shallow), bridge, wooden floors, and building walls. One thing we can count on: we're going to add *lots* of new tile types as we progress, so we better take the time to make it a seamless experience up-front!

The `map.rs` could get quite complicated if we're not careful, so lets make it into its own module with a directory. We'll start by making a directory, `map/`. Then we'll move `map.rs` into it, and rename it `mod.rs`. Now, we'll take `TileType` out of `mod.rs` and put it into a new file - `tiletype.rs`:

```rust
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, Floor, DownStairs
}
```

And in `mod.rs` we'll accept the module and share the public types it exposes:

```rust
mod tiletype;
pub use tiletype::TileType;
```

This hasn't gained us much yet... but now we can start supporting the various tile types. As we add functionality, you'll hopefully see why using a separate file makes it easier to find the relevant code:

```rust
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, 
    Floor, 
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge
}
```

This is only part of the picture, because now we need to handle a bunch of grunt-work: can you enter tiles of that type, do they block visibility, do they have a different cost for path-finding, and so on. We've also done a lot of "spawn if its a floor" code in our map builders; maybe that wasn't such a good idea if you can have multiple floor types? Anyway, the current `map.rs` provides some of what we need in order to satisfy the `BaseMap` trait for RLTK.

We'll make a few functions to help satisfy this requirement, while keeping our tile functionality in one place:

```rust
pub fn tile_walkable(tt : TileType) -> bool {
    match tt {
        TileType::Floor | TileType::DownStairs | TileType::Road | TileType::Grass |
        TileType::ShallowWater | TileType::WoodFloor | TileType::Bridge 
            => true,
        _ => false        
    }
}

pub fn tile_opaque(tt : TileType) -> bool {
    match tt {
        TileType::Wall => true,
        _ => false
    }
}
```

Now we'll go back into `mod.rs`, and import these - and make them public to anyone who wants them:

```rust
mod tiletype;
pub use tiletype::{TileType, tile_walkable, tile_opaque};
```

We also need to update some of our functions to use this functionality. We determine a lot of path-finding with the `blocked` system, so we need to update `populate_blocked` to handle the various types using the functions we just made:

```rust
pub fn populate_blocked(&mut self) {        
    for (i,tile) in self.tiles.iter_mut().enumerate() {
        self.blocked[i] = !tile_walkable(*tile);
    }
}
```

We also need to update our visibility determination code:

```rust
impl BaseMap for Map {
    fn is_opaque(&self, idx:i32) -> bool {
        let idx_u = idx as usize;
        if idx_u > 0 && idx_u < self.tiles.len() {
            tile_opaque(self.tiles[idx_u]) || self.view_blocked.contains(&idx_u)
        } else {
            true
        }
    }
    ...
```

Lastly, lets look at `get_available_exits`. This uses the blocked system to determine if an exit is *possible*, but so far we've hard-coded all of our costs. When there is just a floor and a wall to choose from, it is a pretty easy choice after all! Once we start offering choices, we might want to encourage certain behaviors. It would certainly look more realistic if people preferred to travel on the road than the grass, and *definitely* more realistic if they avoid standing in shallow water unless they need to. So we'll build a *cost* function (in `tiletype.rs`):

```rust
pub fn tile_cost(tt : TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0
    }
}
```

Then we update our `get_available_exits` to use it:

```rust
fn get_available_exits(&self, idx:i32) -> Vec<(i32, f32)> {
    let mut exits : Vec<(i32, f32)> = Vec::new();
    let x = idx % self.width;
    let y = idx / self.width;
    let tt = self.tiles[idx as usize];

    // Cardinal directions
    if self.is_exit_valid(x-1, y) { exits.push((idx-1, tile_cost(tt))) };
    if self.is_exit_valid(x+1, y) { exits.push((idx+1, tile_cost(tt))) };
    if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, tile_cost(tt))) };
    if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, tile_cost(tt))) };

    // Diagonals
    if self.is_exit_valid(x-1, y-1) { exits.push(((idx-self.width)-1, tile_cost(tt) * 1.45)); }
    if self.is_exit_valid(x+1, y-1) { exits.push(((idx-self.width)+1, tile_cost(tt) * 1.45)); }
    if self.is_exit_valid(x-1, y+1) { exits.push(((idx+self.width)-1, tile_cost(tt) * 1.45)); }
    if self.is_exit_valid(x+1, y+1) { exits.push(((idx+self.width)+1, tile_cost(tt) * 1.45)); }

    exits
}
```
We've replaced all the costs of `1.0` with a call to our `tile_cost` function, and multiplied diagonals by 1.45 to encourage more natural looking movement.

## Fixing our camera

We also need to be able to render these tile types, so we open up `camera.rs` and add them to the `match` statement in `get_tile_glyph`:

```rust
fn get_tile_glyph(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Floor => { glyph = rltk::to_cp437('.'); fg = RGB::from_f32(0.0, 0.5, 0.5); }
        TileType::WoodFloor => { glyph = rltk::to_cp437('.'); fg = RGB::named(rltk::CHOCOLATE); }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_f32(0., 1.0, 0.);
        }
        TileType::DownStairs => { glyph = rltk::to_cp437('>'); fg = RGB::from_f32(0., 1.0, 1.0); }
        TileType::Bridge => { glyph = rltk::to_cp437('.'); fg = RGB::named(rltk::CHOCOLATE); }
        TileType::Road => { glyph = rltk::to_cp437('~'); fg = RGB::named(rltk::GRAY); }
        TileType::Grass => { glyph = rltk::to_cp437('"'); fg = RGB::named(rltk::GREEN); }
        TileType::ShallowWater => { glyph = rltk::to_cp437('≈'); fg = RGB::named(rltk::CYAN); }
        TileType::DeepWater => { glyph = rltk::to_cp437('≈'); fg = RGB::named(rltk::NAVY_BLUE); }
    }
    if map.bloodstains.contains(&idx) { bg = RGB::from_f32(0.75, 0., 0.); }
    if !map.visible_tiles[idx] { 
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // Don't show stains out of visual range
    }

    (glyph, fg, bg)
}
```

## Starting to build our town

We want to stop making maps randomly, and instead start being a bit predictable in what we make. So when you start depth 1, you *always* get a town. In `map_builders/mod.rs`, we'll make a new function. For now, it'll just fall back to being random:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    random_builder(new_depth, rng, width, height)
}
```

Pop over to `main.rs` and change the builder function call to use our new function:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = map_builders::level_builder(new_depth, &mut rng, 80, 50);
    ...
```

Now, we'll start fleshing out our `level_builder`; we want depth 1 to generate a town map - otherwise, we'll stick with random for now. We *also* want it to be obvious via a `match` statement how we're routing each level's procedural generation:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    println!("Depth: {}", new_depth);
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}
```

At the top of the `mod.rs` file, add:

```rust
mod town;
use town::town_builder;
```

And in a new file, `map_builders/town.rs` we'll begin our function:

```rust
use super::BuilderChain;

pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
}
```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-47-town1)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-47-town1)
---

Copyright (C) 2019, Herbert Wolverson.

---