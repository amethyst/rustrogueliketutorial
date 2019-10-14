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


**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-33-wfc)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-33-wfc/)
---

Copyright (C) 2019, Herbert Wolverson.

---