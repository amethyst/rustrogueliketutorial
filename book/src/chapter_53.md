# Into the Woods!

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

We've spend a few chapters improving the basic game, its interface, and the starting town. That's fun, and we could honestly keep improving it for *many* chapters - but it's a good idea when developing to see some real progress. Otherwise, you tend to get demotivated! So for this chapter, we're going to add the next level to the game, populate it, and tackle the concept of *themes* to differentiate levels.

## Into the woods!

Our design document says that we go from the town to a limestone cavern. That's a good start, but it's quite unlikely that you would transition from one to the other with nothing in-between; otherwise, *everyone* would go there! So we're going to add a forest next to the town of Bracketon, with a cave entrance to the main adventure. A road runs through the woods, which is where *everyone else* typically goes (those who aren't set on trying to save the world, which is most people!).

Let's start by moving the exit from Bracketon to cover the whole east side. In `town.rs`, find the line placing the exit (it's around line 36), and replace with:

```rust
for y in wall_gap_y-3 .. wall_gap_y + 4 {
    let exit_idx = build_data.map.xy_idx(build_data.width-2, y);
    build_data.map.tiles[exit_idx] = TileType::DownStairs;        
}
```

This fills the whole road out of town with exit tiles:

![Screenshot](./c53-s1.jpg)

This has one primary advantage: it's *really* hard to miss!

## Building the woods

Now we want to start on the second level. In `map_builders/mod.rs` we have the function `level_builder`; lets add a new call in there for the second level:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    println!("Depth: {}", new_depth);
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}
```

To implement this, we'll make a new file - `map_builders/forest.rs` and give it some placeholder content (just like we did for the town):

```rust
use super::{BuilderChain, CellularAutomotaBuilder, XStart, YStart, AreaStartingPosition, 
    CullUnreachable, VoronoiSpawning, DistantExit};

pub fn forest_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomotaBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));

    // Setup an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
```

Also, don't forget to add `mod forest; use forest::forest_builder` to your `map_builders/mod.rs` file! If you run this, you'll see that we have a basic cellular automata dungeon:

![Screenshot](./c53-s2.jpg)

That's isn't really what we want... or is it? It does look a bit forest like in *shape* - but rendering it with wall graphics everywhere doesn't give the impression that you are in a forest.

## Themes

You *could* make all new tiles, and have the forest generator spit them out - but that's duplicating a lot of code just to change appearance. A better approach would be to support *themes*. So the town uses one look, the forest uses another - but they share basic functionality such as walls blocking movement. Now we reveal why we made `map` into a multi-file module: we're going to build a theming engine! Create a new file, `map/themes.rs` and we'll put in a default function and our existing tile selection code (from `camera.rs`):

```rust
use super::{Map, TileType};
use rltk::RGB;

pub fn tile_glyph(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.depth {
        2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map)
    };

    if map.bloodstains.contains(&idx) { bg = RGB::from_f32(0.75, 0., 0.); }
    if !map.visible_tiles[idx] { 
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // Don't show stains out of visual range
    }

    (glyph, fg, bg)
}

fn get_tile_glyph_default(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Floor => { glyph = rltk::to_cp437('.'); fg = RGB::from_f32(0.0, 0.5, 0.5); }
        TileType::WoodFloor => { glyph = rltk::to_cp437('░'); fg = RGB::named(rltk::CHOCOLATE); }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_f32(0., 1.0, 0.);
        }
        TileType::DownStairs => { glyph = rltk::to_cp437('>'); fg = RGB::from_f32(0., 1.0, 1.0); }
        TileType::Bridge => { glyph = rltk::to_cp437('.'); fg = RGB::named(rltk::CHOCOLATE); }
        TileType::Road => { glyph = rltk::to_cp437('≡'); fg = RGB::named(rltk::GRAY); }
        TileType::Grass => { glyph = rltk::to_cp437('"'); fg = RGB::named(rltk::GREEN); }
        TileType::ShallowWater => { glyph = rltk::to_cp437('~'); fg = RGB::named(rltk::CYAN); }
        TileType::DeepWater => { glyph = rltk::to_cp437('~'); fg = RGB::named(rltk::BLUE); }
        TileType::Gravel => { glyph = rltk::to_cp437(';'); fg = RGB::from_f32(0.5, 0.5, 0.5); }
    }

    (glyph, fg, bg)
}

fn wall_glyph(map : &Map, x: i32, y:i32) -> u8 {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
    if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
    if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
    if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 9 } // Pillar because we can't see neighbors
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        _ => { 35 } // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
```

In `map/mod.rs` add `mod themes; pub use themes::*` to add it to your project.

Now we'll modify `camera.rs` by deleting these functions, and importing the map themes instead. Delete `get_tile_glyph`, `wall_glyph` and `is_revealed_and_wall`. At the top, add `use crate::map::tile_glyph` and change the two render functions to use it:

```rust
let (glyph, fg, bg) = tile_glyph(idx, &*map);
```

This has two nice effects: your camera is now *just* a camera, and you have the ability to change your theme per level!

## Building a forest theme

In `themes.rs`, lets extend the `tile_glyph` function to branch to a separate forest theme for level 2:

```rust
pub fn tile_glyph(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    match map.depth {
        2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map)
    }
}
```

Now, of course, we have to *write* `get_forest_glyph`:

```rust
fn get_forest_glyph(idx:usize, map: &Map) -> (u8, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Wall => { glyph = rltk::to_cp437('♣'); fg = RGB::from_f32(0.0, 0.6, 0.0); }
        TileType::Bridge => { glyph = rltk::to_cp437('.'); fg = RGB::named(rltk::CHOCOLATE); }
        TileType::Road => { glyph = rltk::to_cp437('≡'); fg = RGB::named(rltk::YELLOW); }
        TileType::Grass => { glyph = rltk::to_cp437('"'); fg = RGB::named(rltk::GREEN); }
        TileType::ShallowWater => { glyph = rltk::to_cp437('~'); fg = RGB::named(rltk::CYAN); }
        TileType::DeepWater => { glyph = rltk::to_cp437('~'); fg = RGB::named(rltk::BLUE); }
        TileType::Gravel => { glyph = rltk::to_cp437(';'); fg = RGB::from_f32(0.5, 0.5, 0.5); }
        TileType::DownStairs => { glyph = rltk::to_cp437('>'); fg = RGB::from_f32(0., 1.0, 1.0); }
        _ => { glyph = rltk::to_cp437('"'); fg = RGB::from_f32(0.0, 0.6, 0.0); }
    }

    (glyph, fg, bg)
}
```

`cargo run` now, and you'll see that the visual change made a *huge* difference - it now looks like a forest!

![Screenshot](./c53-s3.jpg)

## Follow the yellow-brick road

We specified that a roads runs through the level, but we don't have a builder for that! Let's make one and add it to the builder chain. First, we'll modify the builder chain - get rid of the `DistantExit` part and add a new `YellowBrickRoad` stage:

```rust
pub fn forest_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomotaBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(YellowBrickRoad::new());
    chain
}
```

Then we'll implement `YellowBrickRoad`:

```rust
pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    #[allow(dead_code)]
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad{})
    }

    fn find_exit(&self, build_data : &mut BuilderMap, seed_x : i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if map::tile_walkable(*tiletype) {
                available_floors.push(
                    (
                        idx,
                        rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                            rltk::Point::new(seed_x, seed_y)
                        )
                    )
                );
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        let end_x = available_floors[0].0 as i32 % build_data.map.width;
        let end_y = available_floors[0].0 as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data : &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width-2 || y < 1 || y > build_data.map.height-2 {
            return;
        }
        let idx = build_data.map.xy_idx(x, y);
        if build_data.map.tiles[idx] != TileType::DownStairs {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y) as i32;

        let (end_x, end_y) = self.find_exit(build_data, build_data.map.width - 2, build_data.map.height / 2);
        let end_idx = build_data.map.xy_idx(end_x, end_y);
        build_data.map.tiles[end_idx] = TileType::DownStairs;

        build_data.map.populate_blocked();
        let path = rltk::a_star_search(start_idx, end_idx as i32, &mut build_data.map);
        //if !path.success {
        //    panic!("No valid path for the road");
        //}
        for idx in path.steps.iter() {
            let x = idx % build_data.map.width;
            let y = idx / build_data.map.width;
            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x-1, y);
            self.paint_road(build_data, x+1, y);
            self.paint_road(build_data, x, y-1);
            self.paint_road(build_data, x, y+1);
        }
        build_data.take_snapshot();
    }
}
```

This builder combines a few concepts we've already implemented:

* `find_exit` is just like the `AreaStartingPoint` builder, but finds an area close to the provided "seed" location and returns it. We'll give it a central-east seed point, and use the result as a destination for the road, since we've started in the west.
* `paint_road` checks to see if a tile is within the map bounds, and if it isn't a down staircase - paints it as a road.
* `build` calls `a_star_search` to find an efficient path from west to east. It then paints a 3x3 road all along the path.

The result is a forest with a yellow road going to the East. Of course, there isn't *actually* an exit yet (and you are highly likely to be murdered by kobolds, goblins and orcs)!

![Screenshot](./c53-s4.jpg)

## Adding an exit - and some breadcrumbs

Now we'll hide the exit somewhere in the north-east of the map - or the south-east, we'll pick randomly! Hiding it provides an element of exploration, but not giving the user a clue (especially when the road is essentially a red herring) as to the location is a good way to frustrate your players! We know that the destination is a limestone cave, and limestone caves generally happen because of water - so it stands to reason that there should be a water source in/around the cave. We'll add a stream to the map! Add the following to your `build` function:

```rust
// Place exit
let exit_dir = rng.roll_dice(1, 2);
let (seed_x, seed_y, stream_startx, stream_starty) = if exit_dir == 1 {
    (build_data.map.width-1, 1, 0, build_data.height-1)
} else {
    (build_data.map.width-1, build_data.height-1, 1, build_data.height-1)
};

let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);
build_data.take_snapshot();

let (stream_x, stream_y) = self.find_exit(build_data, stream_startx, stream_starty);
let stream_idx = build_data.map.xy_idx(stream_x, stream_y) as usize;
let stream = rltk::a_star_search(stairs_idx as i32, stream_idx as i32, &mut build_data.map);
for tile in stream.steps.iter() {
    if build_data.map.tiles[*tile as usize] == TileType::Floor {
        build_data.map.tiles[*tile as usize] = TileType::ShallowWater;
    }
}
build_data.map.tiles[stairs_idx] = TileType::DownStairs;
build_data.take_snapshot();
```

This randomly picks an exit location (from NE and SE), and then adds a stream in the opposite direction. Once again, we use path-finding to place the stream - so we don't disturb the overall layout too much. Then we place the exit stairs.

## But - I keep getting murdered by orcs!

We've left the default spawning to happen, with no concern for updating the monsters for our level! Our player is probably *very* low level, especially given that we won't implement levelling up until the next chapter. *ahem*. Anyway, we should introduce some beginner-friendly spawns and adjust the spawn locations of our other enemies. Take a look at `spawns.json` once again, and we'll go straight to the spawn tables at the top. We'll start by adjusting the `min_depth` entries for things we don't want to see yet:

```json
"spawn_table" : [
    { "name" : "Goblin", "weight" : 10, "min_depth" : 3, "max_depth" : 100 },
    { "name" : "Orc", "weight" : 1, "min_depth" : 3, "max_depth" : 100, "add_map_depth_to_weight" : true },
    { "name" : "Health Potion", "weight" : 7, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Fireball Scroll", "weight" : 2, "min_depth" : 0, "max_depth" : 100, "add_map_depth_to_weight" : true },
    { "name" : "Confusion Scroll", "weight" : 2, "min_depth" : 0, "max_depth" : 100, "add_map_depth_to_weight" : true },
    { "name" : "Magic Missile Scroll", "weight" : 4, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Dagger", "weight" : 3, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Shield", "weight" : 3, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Longsword", "weight" : 1, "min_depth" : 3, "max_depth" : 100 },
    { "name" : "Tower Shield", "weight" : 1, "min_depth" : 3, "max_depth" : 100 },
    { "name" : "Rations", "weight" : 10, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Magic Mapping Scroll", "weight" : 2, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Bear Trap", "weight" : 5, "min_depth" : 0, "max_depth" : 100 },
    { "name" : "Battleaxe", "weight" : 1, "min_depth" : 2, "max_depth" : 100 },
    { "name" : "Kobold", "weight" : 15, "min_depth" : 3, "max_depth" : 3 }
],
```

See how none of the monsters appear before depth 3? If you `cargo run` now, you'll have a "Monty Haul" (this was an old TV show about getting free stuff; it became a D&D term for "too easy, too much treasure") of a forest - free stuff everywhere and nary a bit of risk to be seen. We *want* the player to find some useful items, but we also want *some* risk! It's not much of a game if you just win every time!

![Screenshot](./c53-s5.jpg)

## Adding some woodland beasties

What would you expect to find in a beginner-friendly wood? Probably rats, wolves, foxes, various edible-but-harmless wildlife (such as deer) and maybe some travelers. You might even encounter a bear, but it would be very scary at this level! We already have rats, so lets just add them to the spawn table:

```json
{ "name" : "Rat", "weight" : 15, "min_depth" : 2, "max_depth" : 3 }
```

We can add wolves by copy/pasting the rat and editing a bit:

```json
{
    "name" : "Mangy Wolf",
    "renderable": {
        "glyph" : "w",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "ai" : "melee",
    "attributes" : {
        "Might" : 3,
        "Fitness" : 3
    },
    "skills" : {
        "Melee" : -1,
        "Defense" : -1
    },
    "natural" : {
        "armor_class" : 12,
        "attacks" : [
            { "name" : "bite", "hit_bonus" : 0, "damage" : "1d6" }
        ]   
    }
},
```

We'd like them to be less frequent than rats, so lets put them into the spawn table, too - but with a lower weight:

```json
{ "name" : "Mangy Wolf", "weight" : 13, "min_depth" : 2, "max_depth" : 3 }
```

We could make a nasty fox, too. Again, it's secretly quite rat-like!

```json
{
    "name" : "Fox",
    "renderable": {
        "glyph" : "f",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "ai" : "melee",
    "attributes" : {
        "Might" : 3,
        "Fitness" : 3
    },
    "skills" : {
        "Melee" : -1,
        "Defense" : -1
    },
    "natural" : {
        "armor_class" : 11,
        "attacks" : [
            { "name" : "bite", "hit_bonus" : 0, "damage" : "1d4" }
        ]   
    }
},
```

And add the fox to the spawn table, too:

```json
{ "name" : "Fox", "weight" : 15, "min_depth" : 2, "max_depth" : 3 }
```

## It's still too hard - lets give the player more health!

Ok, so we're still getting murdered far too often. Let's give the poor player some more hit points! Open `gamesystem.rs` and edit `player_hp_at_level` to add 10 more HP:

```rust
pub fn player_hp_at_level(fitness:i32, level:i32) -> i32 {
    10 + (player_hp_per_level(fitness) * level)
}
```

In a real game, you'll find yourself tweaking this stuff a *lot* until you get the right feeling of balance!

## Adding in some harmless beasties

Not *everything* in a typical forest is trying to kill you (unless you live in Australia, I'm told). Let's start by making a deer and giving it `bystander` AI so it won't hurt anyone:

```json
{
    "name" : "Deer",
    "renderable": {
        "glyph" : "d",
        "fg" : "#FFFF00",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "ai" : "bystander",
    "attributes" : {
        "Might" : 3,
        "Fitness" : 3
    },
    "skills" : {
        "Melee" : -1,
        "Defense" : -1
    },
    "natural" : {
        "armor_class" : 11,
        "attacks" : [
            { "name" : "bite", "hit_bonus" : 0, "damage" : "1d4" }
        ]   
    }
},
```

And adding it to the spawn table:

```json
{ "name" : "Deer", "weight" : 14, "min_depth" : 2, "max_depth" : 3 }
```

If you `cargo run` now, you'll encounter a plethora of life in the forest - and deer will roam randomly, not doing much.

![Screenshot](./c53-s6.jpg)

## But Venison is Tasty!

The problem with making deer use the `bystander` system is that they roam stupidly, and neither you - nor the wolves - can eat them. On a larger level, you can't eat the wolves either (not that they would taste good). Nor can you sell their pelts, or otherwise profit from their slaughter!

It seems like there are really *three* issues here:

* When we kill things, they should (sometimes) drop loot for us to use.
* Deer need their own AI.
* Wolves need to want to eat deer, which probably requires that they have their own AI too.

### Loot Dropping

### Scared deer

### Hungry Wolves

## Some Brigands - and they drop stuff!

## A big nasty (by 1st level standards) guy at the end

## Wrap-Up

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-53-woods)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-53-woods)
---

Copyright (C) 2019, Herbert Wolverson.

---