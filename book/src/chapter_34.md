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

If you `cargo run` your project now, you can run around the (otherwise deserted) demo map:

![Screenshot](./c34-s1.jpg).

## Populating the test map with prefabbed entities

Let's pretend that our test map is some sort of super-duper end-game map. We'll take a copy and call it `wfc-populated.xp`. Then we'll splat a bunch of monster and item glyphs around it:

![Screenshot](./c34-s2.jpg).

The color coding is completely optional, but I put it in for clarity. You'll see we have an `@` to indicate the player start, a `>` to indicate the exit, and a bunch of `g` goblins, `o` orcs, `!` potions, `%` rations and `^` traps. Not too bad a map, really.

We'll add `wfc-populated.xp` to our `resources` folder, and extend `rex_assets.rs` to load it:

```rust
use rltk::{rex::XpFile};

rltk::embedded_resource!(SMALL_DUNGEON, "../../resources/SmallDungeon_80x50.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_POPULATED, "../../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu : XpFile
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../../resources/SmallDungeon_80x50.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_POPULATED, "../../resources/wfc-populated.xp");

        RexAssets{
            menu : XpFile::from_resource("../../resources/SmallDungeon_80x50.xp").unwrap()
        }
    }
}
```

We also want to be able to list out spawns that are required by the map. Looking in `spawner.rs`, we have an established `tuple` format for how we pass spawns - so we'll use it in the struct:

```rust
#[allow(dead_code)]
pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>
}
```

Now we'll modify our constructor to *use* the new map, and initialize `spawns`:

```rust
impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth : i32) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode : PrefabMode::RexLevel{ template : "../../resources/wfc-populated.xp" },
            spawns: Vec::new()
        }
    }
    ...
```

To make use of the function in `spawner.rs` that accepts this type of data, we need to make it *public*. So we open up the file, and add the word `pub` to the function signature:

```rust
/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    ...
```

We'll then modify our `PrefabBuilder`'s `spawn_entities` function to make use of this data:

```rust
fn spawn_entities(&mut self, ecs : &mut World) {
    for entity in self.spawns.iter() {
        spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
    }
}
```

We do a bit of a dance with references just to work with the previous function signature (and not have to change it, which would change lots of other code). So far, so good - it reads the `spawn` list, and requests that everything in the list be placed onto the map. Now would be a good time to add something to the list! We'll want to modify our `load_rex_map` to handle the new data:

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
                    // We're doing some nasty casting to make it easier to type things like '#' in the match
                    match (cell.ch as u8) as char {
                        ' ' => self.map.tiles[idx] = TileType::Floor,
                        '#' => self.map.tiles[idx] = TileType::Wall,
                        '@' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.starting_position = Position{ x:x as i32, y:y as i32 };
                        }
                        '>' => self.map.tiles[idx] = TileType::DownStairs,
                        'g' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.spawns.push((idx, "Goblin".to_string()));
                        }
                        'o' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.spawns.push((idx, "Orc".to_string()));
                        }
                        '^' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.spawns.push((idx, "Bear Trap".to_string()));
                        }
                        '%' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.spawns.push((idx, "Rations".to_string()));
                        }
                        '!' => {
                            self.map.tiles[idx] = TileType::Floor;
                            self.spawns.push((idx, "Health Potion".to_string()));
                        }
                        _ => {
                            println!("Unknown glyph loading map: {}", (cell.ch as u8) as char);
                        }
                    }
                }
            }
        }
    }
}
```

This recognizes the extra glyphs, and prints a warning to the console if we've loaded one we forgot to handle. Note that for entities, we're setting the tile to `Floor` and *then* adding the entity type. That's because we can't overlay two glyphs on the same tile - but it stands to reason that the entity is *standing* on a floor.

Lastly, we need to modify our `build` function to not move the exit and the player. We simply wrap the fallback code in an `if` statement to detect if we've set a `starting_position` (we're going to require that if you set a start, you also set an exit):

```rust
fn build(&mut self) {
    match self.mode {
        PrefabMode::RexLevel{template} => self.load_rex_map(&template)
    }
    self.take_snapshot();

    // Find a starting point; start at the middle and walk left until we find an open tile
    if self.starting_position.x == 0 {
        self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();
    }
}
```

If you `cargo run` the project now, you start in the specified location - and entities spawn around you.

![Screenshot](./c34-s3.gif).

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-34-vaults)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-34-vaults/)
---

Copyright (C) 2019, Herbert Wolverson.

---