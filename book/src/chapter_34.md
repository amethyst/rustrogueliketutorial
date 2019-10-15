# Prefabs and Vaults

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Despite being essentially pseudorandom (that is, random - but constrained in a way that makes for a fun, cohesive game), *many* roguelikes feature some hand-crafted content. Typically, these can be divided into a few categories:

* Hand-crafted *levels* - the whole level is premade, the content static. These are typically used very sparingly, for big set-piece battles essential to the story.
* Hand-crafted *level sections* - some of the level is randomly created, but a large part is pre-made. For example, a fortress might be a "set piece", but the dungeon leading up to it is random. Dungeon Crawl Stone Soup uses these extensively - you sometimes run into areas that you recognize because they are prefabricated - but the dungeon around them is clearly random. Cogmind uses these for parts of the caves (I'll avoid spoilers). Caves of Qud has a few set-piece levels that appear to be built around a number of prefabricated parts. Some systems call this mechanism "vaults" - but the name can also apply to the third category.
* Hand-crafted *rooms* (also called Vaults in some cases). The level is largely random, but when sometimes a room fits a *vault* - so you put one there.

The first category is special and should be used sparingly (otherwise, your players will just learn an optimal strategy and power on through it - and may become bored from lack of variety). The other categories benefit from either providing *lots* of vaults (so there's a ton of content to sprinkle around, meaning the game doesn't feel too similar each time you play) or being *rare* - so you only occasionally see them (for the same reason).

## Some Clean Up

In the [Waveform Collapse chapter](./chapter_33.md), we loaded a pre-made level - without any entities (those are added later). It's not really very nice to hide a map loader inside WFC - since that isn't it's primary purpose - so we'll start by removing it:

We'll start by deleting the file `map_builders/waveform_collapse/image_loader.rs`. We'll be building a better one in a moment.

Now we edit the start of `mod.rs` in ``map_builders/waveform_collapse`:

```rust
use super::{MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;
mod common;
use common::*;
mod constraints;
use constraints::*;
mod solver;
use solver::*;

/// Provides a map builder using the Waveform Collapse algorithm.
pub struct WaveformCollapseBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>,
    derive_from : Option<Box<dyn MapBuilder>>
}
...

impl WaveformCollapseBuilder {
    /// Generic constructor for waveform collapse.
    /// # Arguments
    /// * new_depth - the new map depth
    /// * derive_from - either None, or a boxed MapBuilder, as output by `random_builder`
    pub fn new(new_depth : i32, derive_from : Option<Box<dyn MapBuilder>>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
            derive_from
        }
    }
    
    /// Derives a map from a pre-existing map builder.
    /// # Arguments
    /// * new_depth - the new map depth
    /// * derive_from - either None, or a boxed MapBuilder, as output by `random_builder`
    pub fn derived_map(new_depth: i32, builder: Box<dyn MapBuilder>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder::new(new_depth, Some(builder))
    }
    ...
```

We've removed all references to `image_loader`, removed the test map constructor, and removed the ugly mode enumeration. WFC is now exactly what it says on the tin, and nothing else. Lastly, we'll modify `random_builder` to not use the test map anymore:

```rust
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 16);
    let mut result : Box<dyn MapBuilder>;
    match builder {
        1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(CellularAutomotaBuilder::new(new_depth)); }
        4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
        8 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        9 => { result = Box::new(MazeBuilder::new(new_depth)); }
        10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
        11 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
        12 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
        13 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
        14 => { result = Box::new(VoronoiCellBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(VoronoiCellBuilder::manhattan(new_depth)); }
        _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 3)==1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    result
}
```

## Skeletal Builder

We'll start with a very basic skeleton, similar to those used before. We'll make a new file, `prefab_builder.rs` in `map_builders`:

```rust
use super::{MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    draw_corridor};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self)  {
        self.build();
    }

    fn spawn_entities(&mut self, ecs : &mut World) {
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl PrefabBuilder {
    pub fn new(new_depth : i32) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new()
        }
    }

    fn build(&mut self) {
    }
}
```

## Prefab builder mode 1 - hand-crafted levels

We're going to support multiple modes for the prefab-builder, so lets bake that in at the beginning. In `prefab_builder.rs`:

```rust
#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode { 
    RexLevel{ template : &'static str }
}

pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode
}
```

This is new - an `enum` with variables? This works because under the hood, Rust enumerations are actually *unions*. They can hold whatever you want to put in there, and the type is sized to hold the largest of the options. It's best used sparingly in tight code, but for things like configuration it is a very clean way to pass in data. We should also update the constructor to create the new types:

```rust
impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth : i32) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode : PrefabMode::RexLevel{ template : "../../resources/wfc-demo1.xp" }
        }
    }
    ...
```

Including the map template path in the mode makes for easier reading, even if it is slightly more complicated. We're not filling the `PrefabBuilder` with variables for all of the options we *might* use - we're keeping them separated. That's generally good practice - it makes it much more obvious to someone who reads your code what's going on.

Now we'll re-implement the map reader we previously deleted from `image_loader.rs` - only we'll add it as a member function for `PrefabBuilder`, and use the enclosing class features rather than passing `Map` and `new_depth` in and out:

```rust
#[allow(dead_code)]
fn load_rex_map(&mut self, path: &str) {
    let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

    for layer in &xp_file.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if x < self.map.width as usize && y < self.map.height as usize {
                    let idx = self.map.xy_idx(x as i32, y as i32);
                    match (cell.ch as u8) as char {
                        ' ' => self.map.tiles[idx] = TileType::Floor, // space
                        '#' => self.map.tiles[idx] = TileType::Wall, // #
                        _ => {}
                    }
                }
            }
        }
    }
}
```

That's pretty straightforward, more or less a direct port of the one form the Waveform Collapse chapter. Now lets start making our `build` function:

```rust
fn build(&mut self) {
    match self.mode {
        PrefabMode::RexLevel{template} => self.load_rex_map(&template)
    }

    // Find a starting point; start at the middle and walk left until we find an open tile
    self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
    let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
    while self.map.tiles[start_idx] != TileType::Floor {
        self.starting_position.x -= 1;
        start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
    }
    self.take_snapshot();
}
```

Notice that we've copied over the find starting point code; we'll improve that at some point, but for now it ensures you can play your level. We *haven't* spawned anything - so you will be alone in the level. There's also a slightly different usage of `match` here - we're using the variable in the enum. The code `PrefabMode::RexLevel{template}` says "match `RexLevel`, but with *any* value of `template` - and make that value available via the name `template` in the match scope". You could use `_` to match any value if you didn't want to *access* it. [Rust's pattern matching system](https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html) is really impressive - you can do a *lot* with it!

Lets modify our `random_builder` function to always call this type of map (so we don't have to test over and over in the hopes of getting the one we want!). In `map_builders/mod.rs`:

```rust
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    /*
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 16);
    let mut result : Box<dyn MapBuilder>;
    match builder {
        1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(CellularAutomotaBuilder::new(new_depth)); }
        4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
        8 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        9 => { result = Box::new(MazeBuilder::new(new_depth)); }
        10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
        11 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
        12 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
        13 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
        14 => { result = Box::new(VoronoiCellBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(VoronoiCellBuilder::manhattan(new_depth)); }
        _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 3)==1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    result*/

    Box::new(PrefabBuilder::new(new_depth))
}
```

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-34-vaults)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-34-vaults/)
---

Copyright (C) 2019, Herbert Wolverson.

---