# Waveform Collapse

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

A few years ago, *Waveform Collapse* (WFC) exploded onto the procedural generation scene. Apparently magical, it took images in - and made a similar image. Demos showed it spitting out great looking game levels, and the amazing Caves of Qud started using it for generating fun levels. The canonical demonstrations - along with the original algorithm in C# and various explanatory links/ports - may be [found here](https://github.com/mxgmn/WaveFunctionCollapse).

In this chapter, we're going to implement Waveform Collapse from scratch - and apply it to making fun Roguelike levels. Note that there is a crate with the original algorithm available (`wfc`, accompanied by `wfc-image`); it seemed pretty good in testing, but I had problems making it work with Web Assembly. I also didn't feel that I was really *teaching* the algorithm by saying "just import this". It's a longer chapter, but by the end you should feel comfortable with the algorithm.

## So what does WFC really do?

Waveform Collapse is unlike the map generation algorithms we've used so far in that it doesn't actually *make* maps. It takes source data in (we'll use other maps!), scans them, and builds a new map featuring elements made exclusively from the source data. It operates in a few phases:

1. It reads the incoming data. In the original implementation, this was a PNG file. In our implementation, this is a `Map` structure like others we've worked with; we'll also implement a REX Paint reader to load maps.
2. It divides the source image into "tiles", and optionally makes more tiles by mirroring the tiles it reads along one or two axes.
3. It either loads or builds a "constraints" graph. This is a set of rules specifying which tiles can go next to each other. In an image, this may be derived from tile adjacency. In a Roguelike map, connectivity of exits is a good metric. For a tile-based game, you might carefully build a layout of what can go where.
4. It then divides the output image into tile-sized chunks, and sets them all to "empty". The first tile placed will be pretty random, and then it selects areas and examines tile data that is already known - placing down tiles that are compatible with what is already there. Eventually, it's placed all of the tiles - and you have a map/image!

The name "Waveform Collapse" refers to the Quantum Physics idea that a particle may have not actually *have* a state until you look at it. In the algorithm, tiles don't really *coalesce* into being until you pick one to examine. So there is a slight similarity to Quantum Physics. In reality, though - the name is a triumph of marketing. The algorithm is what is known as a *solver* - given a set of constraints, it iterates through possible solutions until the constraints are *solved*. This isn't a new concept - [Prolog](https://en.wikipedia.org/wiki/Prolog) is an entire programming language based around this idea, and it first hit the scene in 1972. So in a way, it's older than me!

## Getting started: Rust support for complex modules

All our previous algorithms were small enough to fit into one source code file, without too much paging around to find the relevant bit of code. Waveform Collapse is complicated enough that it deserves to be broken into multiple files - in much the same was as the `map_builders` module was broken into a `module` - WFC will be divided into its own `module`. The module will still live inside `map_builders` - so in a way it's really a *sub-module*.

Rust makes it pretty easy to break any module into multiple files: you create a directory inside the *parent* module, and put a file in it called `mod.rs`. You can then put more files in the folder, and so long as you enable them (with `mod myfile`) and use the contents (with `use myfile::MyElement`) it works just like a single file.

So to get started, inside your `map_builders` directory - make a new directory called `waveform_collapse`. Add a file, `mod.rs` into it. You should have a source tree like this:

```
\ src
   \ map_builders
      \ waveform_collapse
         + mod.rs
      bsp_dungeon.rs
      (etc)
   main.rs
   (etc)
```

We'll populate `mod.rs` with a skeletal implementation similar to previous chapters:

```rust
use super::{MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct WaveformCollapseBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>
}

impl MapBuilder for WaveformCollapseBuilder {
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
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
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

impl WaveformCollapseBuilder {
    pub fn new(new_depth : i32) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new()
        }
    }    

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // TODO: Builder goes here

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        /*let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }*/
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

}
```

We'll also modify `map_builders/mod.rs`'s `random_builder` function to always return the algorithm we're currently working with:

```rust
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    /*
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 16);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomotaBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        9 => Box::new(MazeBuilder::new(new_depth)),
        10 => Box::new(DLABuilder::walk_inwards(new_depth)),
        11 => Box::new(DLABuilder::walk_outwards(new_depth)),
        12 => Box::new(DLABuilder::central_attractor(new_depth)),
        13 => Box::new(DLABuilder::insectoid(new_depth)),
        14 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
        15 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth))
    }*/
    Box::new(WaveformCollapseBuilder::new(new_depth))
}
```

This will give you an empty map (all walls) if you `cargo run` it - but it's a good starting point.

## Loading the source image - REX Paint

You may remember back in [section 2](chapter_21.html) we loaded a REX Paint file to use as the main menu screen. We're going to do similar here, but we're going to turn it into a playable map. It's a deliberately odd map to help illustrate what you can do with this algorithm. Here's the original in REX Paint:

![Screenshot](./c33-s1.jpg).

I've tried to include some interesting shapes, a silly face, and plenty of corridors and different sized rooms. Here's a second REX Paint file, designed to be more like the old board game [The Sorcerer's Cave](https://en.wikipedia.org/wiki/The_Sorcerer%27s_Cave), of which the algorithm reminds me - tiles with 1 exit, 2 exits, 3 exits and 4. It would be easy to make these prettier, but we'll keep it simple for demonstration purposes.

![Screenshot](./c33-s2.jpg).

These files are found in the `resources` directory, as `wfc-demo1.xp` and `wfc-demo2.xp`. One thing I love about REX Paint: the files are *tiny* (102k and 112k respectively). To make accessing them easier - and avoid having to ship them with the executable when you publish your finished game, we'll *embed* them into our game. We did this previously for the main menu. Modify `rex_assets.xp` to include the new files:

```rust
use rltk::{rex::XpFile};

rltk::embedded_resource!(SMALL_DUNGEON, "../../resources/SmallDungeon_80x50.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE2, "../../resources/wfc-demo2.xp");

pub struct RexAssets {
    pub menu : XpFile
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../../resources/SmallDungeon_80x50.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE2, "../../resources/wfc-demo2.xp");

        RexAssets{
            menu : XpFile::from_resource("../../resources/SmallDungeon_80x50.xp").unwrap()
        }
    }
}
```

Finally, we should *load* the map itself! Inside the `waveform_collapse` directory, make a new file: `image_loader.rs`:

```rust
use rltk::rex::XpFile;
use super::{Map, TileType};

/// Loads a RexPaint file, and converts it into our map format
pub fn load_rex_map(new_depth: i32, xp_file : &XpFile) -> Map {
    let mut map : Map = Map::new(new_depth);

    for layer in &xp_file.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if x < map.width as usize && y < map.height as usize {
                    let idx = map.xy_idx(x as i32, y as i32);
                    match cell.ch {
                        32 => map.tiles[idx] = TileType::Floor, // #
                        35 => map.tiles[idx] = TileType::Wall, // #
                        _ => {}
                    }
                }
            }
        }
    }

    map
}
```

This is really simple, and if you remember the main menu graphic tutorial it should be quite self-explanatory. This function:

1. Accepts arguments for `new_depth` (because maps want it) and a *reference* to an `XpFile` - a REX Paint map. It will be made completely solid, walls everywhere by the constructor.
2. It creates a new map, using the `new_depth` parameter.
3. For each *layer* in the REX Paint file (there should be only one at this point):
    1. For each `y` and `x` on that layer:
        1. Load the tile information for that coordinate.
        2. Ensure that we're within the map boundaries (in case we have a mismatch in sizes).
        3. Calculate the `tiles` index for the cell.
        4. Match on the cell glyph; if its a `#` (35) we place a wall, if its a space (32) we place a floor.

Now we can modify our `build` function (in `mod.rs`) to load the map:

```rust
fn build(&mut self) {
    let mut rng = RandomNumberGenerator::new();

    self.map = load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../../resources/wfc-demo1.xp").unwrap());
    self.take_snapshot();

    // Find a starting point; start at the middle and walk left until we find an open tile
    self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
    ...
```

At the top, we have to tell it to *use* the new `image_loader` file:

```rust
mod image_loader;
use image_loader::*;
```

Note that we're *not* putting `pub` in front of these: we're using them, but not exposing them outside of the module. This helps us keep our code clean, and our compile times short!

In and of itself, this is cool - we can now load any REX Paint designed level and play it! If you `cargo run` now, you'll find that you can play the new map:

![Screenshot](./c33-s3.jpg).

We'll make use of this in later chapters for *vaults*, *prefabs* and *pre-designed levels* - but for now, we'll just use it as source data for later in the Waveform Collapse implementation.

## Carving up our map into tiles

We discussed earlier that WFC works by carving the original image into chunks/tiles, and optionally flipping them in different directions. It does this as the first part of building *constraints* - how the map can be laid out. So now we need to start carving up our image.

We'll start by picking a tile size (we're going to call it `chunk_size`). We'll make it a constant for now (it'll become tweakable later), and start with a size of `7` - because that was the size of the tiles in our second REX demo file. We'll also call a function we'll write in a moment:

```rust
fn build(&mut self) {
    let mut rng = RandomNumberGenerator::new();

    const CHUNK_SIZE :i32 = 7;

    self.map = load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../../resources/wfc-demo2.xp").unwrap());
    self.take_snapshot();

    let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
    ...
```

Since we're dealing with *constraints*, we'll make a new file in our `map_builders/waveform_collapse` directory - `constraints.rs`. We're going to make a function called `build_patterns`:

```rust
use super::{TileType, Map};
use std::collections::HashSet;

pub fn build_patterns(map : &Map, chunk_size: i32, include_flipping: bool, dedupe: bool) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            // Normal orientation
            let mut pattern : Vec<TileType> = Vec::new();
            let start_x = cx * chunk_size;
            let end_x = (cx+1) * chunk_size;
            let start_y = cy * chunk_size;
            let end_y = (cy+1) * chunk_size;

            for y in start_y .. end_y {
                for x in start_x .. end_x {
                    let idx = map.xy_idx(x, y);
                    pattern.push(map.tiles[idx]);
                }
            }
            patterns.push(pattern);

            if include_flipping {
                // Flip horizontal
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(end_x - (x+1), y);
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip vertical
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(x, end_y - (y+1));
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip both
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(end_x - (x+1), end_y - (y+1));
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);
            }
        }
    }

    // Dedupe
    if dedupe {
        println!("Pre de-duplication, there are {} patterns", patterns.len());
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect(); // dedup
        patterns.extend(set.into_iter());
        println!("There are {} patterns", patterns.len());
    }

    patterns
}
```

That's quite the mouthful of a function, so let's walk through it:

1. At the top, we're importing some items from elsewhere in the project: `Map`, `TileType`, and the built-in collection `HashMap`.
2. We declare our `build_patterns` function, with parameters for a *reference* to the source map, the `chunk_size` to use (tile size), and *flags* (`bool` variables) for `include_flipping` and `dedupe`. These indicate which features we'd like to use when reading the source map. We're returning a `vector`, containing a series of `vector`s of different `TileType`s. The outer container holds each *pattern*. The inner vector holds the `TileType`s that make up the pattern itself.
3. We determine how many chunks there are in each direction and store it in `chunks_x` and `chunks_y`.
4. We create a new `vector` called `patterns`. This will hold the result of the function; we don't declare it's type, because Rust is smart enough to see that we're returning it at the end of the function - and can figure out what type it is for us.
5. We iterate every vertical chunk in the variable `cy`:
    1. We iterate every horizontal chunk in the variable `cx`:
        1. We make a new `vector` to hold this pattern.
        2. We calculate `start_x`, `end_x`, `start_y` and `end_y` to hold the four corner coordinates of this chunk - on the original map.
        3. We iterate the pattern in `y`/`x` order (to match our map format), read in the `TileType` of each map tile within the chunk, and add it to the pattern.
        4. We push the pattern to the `patterns` result vector.
        5. If `include_flipping` is set to `true` (because we'd like to flip our tiles, making more tiles!):
            1. Repeat iterating `y`/`x` in different orders, giving 3 more tiles. Each is added to the `patterns` result vector.
6. If `dedupe` is set, then we are "de-duplicating" the pattern buffer. Basically, removing any pattern that occurs more than once. This is good for a map with lots of wasted space, if you don't want to make an equally sparse result map. We de-duplicate by adding the patterns into a `HashMap` (which can only store one of each entry) and then reading it back out again.

For this to compile, we have to make `TileType` know how to convert itself into a *hash*. `HashMap` uses "hashes" (basically a checksum of the contained values) to determine if an entry is unique, and to help find it. In `map.rs`, we can simply add one more derived attribute to the `TileType` enumeration:

```rust
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, Floor, DownStairs
}
```

This code should get you every 7x7 tile within your source file - but it'd be *great* to be able to prove that it works! As Reagan's speech-writer once wrote, *Trust - But Verify*. In `constraints.rs`, we'll add another function: `render_pattern_to_map`:

```rust
fn render_pattern_to_map(map : &mut Map, pattern: &Vec<TileType>, chunk_size: i32, start_x : i32, start_y: i32) {
    let mut i = 0usize;
    for tile_y in 0..chunk_size {
        for tile_x in 0..chunk_size {
            let map_idx = map.xy_idx(start_x + tile_x, start_y + tile_y);
            map.tiles[map_idx] = pattern[i];
            map.visible_tiles[map_idx] = true;
            i += 1;
        }
    }
}
```

This is pretty simple: iterate the pattern, and copy to a location on the map - offset by the `start_x` and `start_y` coordinates. Note that we're also marking the tile as `visible` - this will make the renderer display our tiles in color.

Now we just need to display our tiles as part of the `snapshot` system. In `waveform_collapse/mod.rs` add a new function as part of the *implementation* of `WaveformCollapseBuilder` (underneath `build`). It's a *member* function because it needs access to the `take_snapshot` command:

```rust
fn render_tile_gallery(&mut self, patterns: &Vec<Vec<TileType>>, chunk_size: i32) {
    self.map = Map::new(0);
    let mut counter = 0;
    let mut x = 1;
    let mut y = 1;
    while counter < patterns.len() {
        render_pattern_to_map(&mut self.map, &patterns[counter], chunk_size, x, y);

        x += chunk_size + 1;
        if x + chunk_size > self.map.width {
            // Move to the next row
            x = 1;
            y += chunk_size + 1;

            if y + chunk_size > self.map.height {
                // Move to the next page
                self.take_snapshot();
                self.map = Map::new(0);

                x = 1;
                y = 1;
            }
        }

        counter += 1;
    }
    self.take_snapshot();
}
```

Now, we need to call it. In `build`:

```rust
let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
self.render_tile_gallery(&patterns, CHUNK_SIZE);
```

Also, comment out some code so that it doesn't crash from not being able to find a starting point:

```rust
let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
/*while self.map.tiles[start_idx] != TileType::Floor {
    self.starting_position.x -= 1;
    start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
}*/
```

If you `cargo run` now, it'll show you the tile patterns from map sample 2:

![Screenshot](./c33-s4.jpg).

Notice how *flipping* has given us multiple variants of each tile. If we change the image loading code to load `wfc-demo1` (by changing the loader to `self.map = load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../../resources/wfc-demo1.xp").unwrap());`), we get chunks of our hand-drawn map:

![Screenshot](./c33-s5.jpg).


**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-33-wfc)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-33-wfc/)
---

Copyright (C) 2019, Herbert Wolverson.

---