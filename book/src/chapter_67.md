# Enter The Dragon

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Now that we have spells and more advanced item abilities, the player might just be able to survive the first layer of the dwarven fortress we build back in section 4.17! Also, now that NPCs can use special abilities - we should be able to model the evil dragon who (according our design document) occupies the dwarven fortress! This chapter will be all about building the dragon's lair, populating it, and making the dragon a scary - but beatable - foe.

## Building the Lair

According to the design document, level six used to be a mighty Dwarven Fortress - but has been occupied by a nasty black dragon who just loves eating adventurers (and presumably dealt with the dwarves of yore). That implies that what we want is a corridor-based dungeon - Dwarves tend to love that, but eroded and blasted into a dragon's lair.

To assist with this, we'll re-enable watching map generation. In `main.rs`, change the toggle to true:

```rust
const SHOW_MAPGEN_VISUALIZER : bool = true;
```

Next, we'll put together a skeleton to build the level. In `map_builders/mod.rs`, add the following:

```rust
mod dwarf_fort_builder;
use dwarf_fort_builder::*;
```

And update the function at the end:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    println!("Depth: {}", new_depth);
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_cavern_builder(new_depth, rng, width, height),
        4 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        5 => limestone_transition_builder(new_depth, rng, width, height),
        6 => dwarf_fort_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}
```

Now, we'll make a new file - `map_builders/dwarf_fort.rs` and put a minimal BSP-based room dungeon in it:

```rust
use super::{BuilderChain, XStart, YStart, AreaStartingPosition, RoomSorter, RoomSort,
    CullUnreachable, VoronoiSpawning, BspDungeonBuilder, DistantExit, BspCorridors,
    CorridorSpawner, RoomDrawer};

pub fn dwarf_fort_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarven Fortress");
    chain.start_with(BspDungeonBuilder::new());
    chain.with(RoomSorter::new(RoomSort::CENTRAL));
    chain.with(RoomDrawer::new());
    chain.with(BspCorridors::new());
    chain.with(CorridorSpawner::new());

    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::TOP));
    chain.with(CullUnreachable::new());
    chain.with(AreaEndingPosition::new(XEnd::RIGHT, YEnd::BOTTOM));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
```

This gets you a very basic room-based dungeon:

![Screenshot](./c67-s1.gif)

That's a good start, but not really what we're looking for. It's obviously man (dwarf!) made, but it doesn't have the "scary dragon lives here" vibe going on. So we'll *also* make a scary looking map with a larger central area, and merge the two together. We want a somewhat sinister feel, so we'll make a custom builder layer to generate a DLA Insectoid map and marge it in:

```rust
pub struct DragonsLair {}

impl MetaMapBuilder for DragonsLair {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DragonsLair {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonsLair> {
        Box::new(DragonsLair{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        build_data.map.depth = 7;
        build_data.take_snapshot();

        let mut builder = BuilderChain::new(6, build_data.width, build_data.height, "New Map");
        builder.start_with(DLABuilder::insectoid());
        builder.build_map(rng);

        // Add the history to our history
        for h in builder.build_data.history.iter() {
            build_data.history.push(h.clone());
        }
        build_data.take_snapshot();

        // Merge the maps
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Wall && builder.build_data.map.tiles[idx] == TileType::Floor {
                *tt = TileType::Floor;
            }
        }
        build_data.take_snapshot();
    }
}
```

And we'll add it into our builder function:

```rust
pub fn dwarf_fort_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarven Fortress");
    chain.start_with(BspDungeonBuilder::new());
    chain.with(RoomSorter::new(RoomSort::CENTRAL));
    chain.with(RoomDrawer::new());
    chain.with(BspCorridors::new());
    chain.with(CorridorSpawner::new());
    chain.with(DragonsLair::new());

    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::TOP));
    chain.with(CullUnreachable::new());
    chain.with(AreaEndingPosition::new(XEnd::RIGHT, YEnd::BOTTOM));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
```

This gives a much more appropriate map:

![Screenshot](./c67-s2.gif)

You'll notice that we're starting at one diagonal corner, and ending at the other - to make it hard for the player to completely avoid the middle!

## General Spawns

If you `cargo run` right now, you'll notice that the level is mostly full of items - free loot! - and not so many monsters. There's actually two things going on here: we're weighting items and mobs in the same table, and we've added a *lot* of items recently (relative to mobs and props) - and there aren't that many mobs permitted on this level to begin with. As we add more and more items, the issue is only going to get worse! So let's fix it.

Let's start by opening up `raws/rawmaster.rs` and add a new function to determine what type of spawn an entry is:

```rust
pub enum SpawnTableType { Item, Mob, Prop }

pub fn spawn_type_by_name(raws: &RawMaster, key : &str) -> SpawnTableType {
    if raws.item_index.contains_key(key) {
        SpawnTableType::Item
    } else if raws.mob_index.contains_key(key) {
        SpawnTableType::Mob
    } else {
        SpawnTableType::Prop
    }
}
```

We'll add a new `MasterTable` structure to the `random_table.rs` file. This acts as a holder for tables, sorted by item type. We're also going to fix some of the odd layout of the previous version:

```rust
use rltk::RandomNumberGenerator;
use crate::raws::{SpawnTableType, spawn_type_by_name, RawMaster};

pub struct RandomEntry {
    name : String,
    weight : i32
}

impl RandomEntry {
    pub fn new<S:ToString>(name: S, weight: i32) -> RandomEntry {
        RandomEntry{ name: name.to_string(), weight }
    }
}

#[derive(Default)]
pub struct MasterTable {
    items : RandomTable,
    mobs : RandomTable,
    props : RandomTable
}

impl MasterTable {
    pub fn new() -> MasterTable {
        MasterTable{
            items : RandomTable::new(),
            mobs : RandomTable::new(),
            props : RandomTable::new()
        }
    }

    pub fn add<S:ToString>(&mut self, name : S, weight: i32, raws: &RawMaster) {
        match spawn_type_by_name(raws, &name.to_string()) {
            SpawnTableType::Item => self.items.add(name, weight),
            SpawnTableType::Mob => self.mobs.add(name, weight),
            SpawnTableType::Prop => self.props.add(name, weight),
        }
    }

    pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        let roll = rng.roll_dice(1, 4);
        match roll {
            1 => self.items.roll(rng),
            2 => self.props.roll(rng),
            3 => self.mobs.roll(rng),
            _ => "None".to_string()
        }
    }
}

#[derive(Default)]
pub struct RandomTable {
    entries : Vec<RandomEntry>,
    total_weight : i32
}

impl RandomTable {
    pub fn new() -> RandomTable {
        RandomTable{ entries: Vec::new(), total_weight: 0 }
    }

    pub fn add<S:ToString>(&mut self, name : S, weight: i32) {
        if weight > 0 {
            self.total_weight += weight;
            self.entries.push(RandomEntry::new(name.to_string(), weight));
        }
    }

    pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        if self.total_weight == 0 { return "None".to_string(); }
        let mut roll = rng.roll_dice(1, self.total_weight)-1;
        let mut index : usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone();
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        "None".to_string()
    }
}
```

As you can see, this divides the available spawns by type - and then rolls for which table to use, before it rolls on the table itself. Now in `rawmaster.rs`, we'll modify the `get_spawn_table_for_depth` function to use the master table:

```rust
pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> MasterTable {
    use super::SpawnTableEntry;

    let available_options : Vec<&SpawnTableEntry> = raws.raws.spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = MasterTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt.add(e.name.clone(), weight, raws);
    }

    rt
}
```

Since we've implemented basically the same interface for the `MasterTable`, we can mostly keep the existing code - and just use the new type instead. In `spawner.rs`, we also need to change one function signature:

```rust
fn room_table(map_depth: i32) -> MasterTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
```

If you `cargo run` now, there's a much better balance of monsters and items (on ALL levels!).

## Enter The Dragon

This level is dominated by a black dragon. Checking our D&D rules, these are fearsome *enormous* lizards of incredible physical might, razor-sharp teeth and claws, and a horrible acid breath that burns the flesh from your bones. With an introduction like that, the dragon had better be pretty fearsome! He also needs to be possible to beat, by a player with 5 levels under their belt. Slaying the dragon really should give some pretty amazing rewards, too. It should also be possible to sneak around the dragon if you are careful!

In `spawns.json`, we'll make a start at sketching out what the dragon looks like:

```json
{
    "name" : "Black Dragon",
    "renderable": {
        "glyph" : "D",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 12,
    "movement" : "static",
    "attributes" : {
        "might" : 5,
        "fitness" : 5
    },
    "skills" : {
        "Melee" : 18,
        "Defense" : 16
    },
    "natural" : {
        "armor_class" : 17,
        "attacks" : [
            { "name" : "bite", "hit_bonus" : 4, "damage" : "1d10+2" },
            { "name" : "left_claw", "hit_bonus" : 2, "damage" : "1d10" },
            { "name" : "right_claw", "hit_bonus" : 2, "damage" : "1d10" }
        ]
    },
    "loot_table" : "Wyrms",
    "faction" : "Wyrm",
    "level" : 6,
    "gold" : "10d6",
    "abilities" : [
        { "spell" : "Acid Breath", "chance" : 0.2, "range" : 8.0, "min_range" : 2.0 }
    ]
},
```

We'll also need to define an *Acid Breath* effect:

```json
{
    "name" : "Acid Breath",
    "mana_cost" : 2,
    "effects" : {
        "ranged" : "6",
        "damage" : "10",
        "area_of_effect" : "3",
        "particle_line" : "☼;#00FF00;400.0"
    }
}
```

Now we need to actually spawn the dragon. We *don't* want to put the dragon into our spawn table - that would make him appear randomly, and potentially in the wrong level. That ruins his reputation as a boss! We also don't want him to be surrounded by friends - that would be too difficult for the player (and draw focus away from the boss fight).

In `dwarf_fort_builder.rs`, we'll add another layer to the spawn generation:

```rust
pub struct DragonSpawner {}

impl MetaMapBuilder for DragonSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DragonSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonSpawner> {
        Box::new(DragonSpawner{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Find a central location that isn't occupied
        let seed_x = build_data.map.width / 2;
        let seed_y = build_data.map.height / 2;
        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if crate::map::tile_walkable(*tiletype) {
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

        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;
        let dragon_pt = rltk::Point::new(start_x, start_y);

        // Remove all spawns within 25 tiles of the drake
        let w = build_data.map.width as i32;
        build_data.spawn_list.retain(|spawn| {
            let spawn_pt = rltk::Point::new(
                spawn.0 as i32 % w,
                spawn.0 as i32 / w
            );
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(dragon_pt, spawn_pt);
            distance > 25.0
        });

        // Add the dragon
        let dragon_idx = build_data.map.xy_idx(start_x, start_y);
        build_data.spawn_list.push((dragon_idx, "Black Dragon".to_string()));
    }
}
```

This function is quite straight-forward, and very similar to ones we've written before. We find an open space close to the map's center, and then remove all mob spawns that are less than 25 tiles from the center point (keeping mobs away from the middle). We then spawn the black dragon in the middle.

Let's pop over to `main.rs` and temporarily change one line in the `main` function:

```rust
gs.generate_world_map(6, 0);
```

That starts you on the dragon lair level (remember to change this back when we're done!), so you don't have to navigate the other levels to complete it. Now `cargo run` the project, use the cheat (`backslash` followed by `g`) to enable *God Mode* - and explore the level. It looks pretty good, but the dragon is so powerful that it takes *ages* to slay them - and if you watch the damage log, the player is pretty certain to die! With a bit of practice, though - you can take down the dragon with a combination of spells and item use. So we'll let it stand for now.

Go ahead and change the `main` function back:

```rust
gs.generate_world_map(1, 0);
```

## Worrying about balance / Playtesting

Now for the *hard* part. It's a good time to play through levels 1 through 6 a few times, see how far you get, and see what problems you encounter. Here's a few things I noticed:

* I ran into *deer* being far more annoying than intended, so for now I've removed them from the spawn table.
* *Rats* can't actually damage you! Changing their might to 7 gives a minimal penalty, but makes them occasionally do some damage.
* Healing potions need to spawn a lot more frequently! I changed their spawn weight to 15. I encountered several occasions in which I really needed an emergency heal.
* The game is a little too hard; you are really at the mercy of the Random Number Generator. You can be doing really well, and a bandit with good dice rolls slaughters you mercilessly - even after you've managed to get some armor on and leveled up! I decided to do two things to rectify this:

I gave the player 20 hit points per level instead of 10, by changing `player_hp_per_level` and `player_hp_at_level` in `gamessytem.rs`:

```rust
pub fn player_hp_per_level(fitness: i32) -> i32 {
    15 + attr_bonus(fitness)
}

pub fn player_hp_at_level(fitness:i32, level:i32) -> i32 {
    15 + (player_hp_per_level(fitness) * level)
}
```

In `effects/damage.rs` where we handle leveling up, I gave the player some more reason to level up! A random attribute and all skills improve. So leveling up makes you faster, stronger and more damaging. It also makes you harder to hit. Here's the code:

```rust
if xp_gain != 0 || gold_gain != 0.0 {
    let mut log = ecs.fetch_mut::<GameLog>();
    let mut player_stats = pools.get_mut(source).unwrap();
    let mut player_attributes = attributes.get_mut(source).unwrap();
    player_stats.xp += xp_gain;
    player_stats.gold += gold_gain;
    if player_stats.xp >= player_stats.level * 1000 {
        // We've gone up a level!
        player_stats.level += 1;
        log.entries.insert(0, format!("Congratulations, you are now level {}", player_stats.level));

        // Improve a random attribute
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        let attr_to_boost = rng.roll_dice(1, 4);
        match attr_to_boost {
            1 => {
                player_attributes.might.base += 1;
                log.entries.insert(0, "You feel stronger!".to_string());
            }
            2 => {
                player_attributes.fitness.base += 1;
                log.entries.insert(0, "You feel healthier!".to_string());
            }
            3 => {
                player_attributes.quickness.base += 1;
                log.entries.insert(0, "You feel quicker!".to_string());
            }
            _ => {
                player_attributes.intelligence.base += 1;
                log.entries.insert(0, "You feel smarter!".to_string());
            }
        }

        // Improve all skills
        let mut skills = ecs.write_storage::<Skills>();
        let player_skills = skills.get_mut(*ecs.fetch::<Entity>()).unwrap();
        for sk in player_skills.skills.iter_mut() {
            *sk.1 += 1;
        }

        ecs.write_storage::<EquipmentChanged>()
            .insert(
                *ecs.fetch::<Entity>(),
                EquipmentChanged{})
            .expect("Insert Failed");

        player_stats.hit_points.max = player_hp_at_level(
            player_attributes.fitness.base + player_attributes.fitness.modifiers,
            player_stats.level
        );
        player_stats.hit_points.current = player_stats.hit_points.max;
        player_stats.mana.max = mana_at_level(
            player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
            player_stats.level
        );
        player_stats.mana.current = player_stats.mana.max;

        let player_pos = ecs.fetch::<rltk::Point>();
        let map = ecs.fetch::<Map>();
        for i in 0..10 {
            if player_pos.y - i > 1 {
                add_effect(None, 
                    EffectType::Particle{ 
                        glyph: rltk::to_cp437('░'),
                        fg : rltk::RGB::named(rltk::GOLD),
                        bg : rltk::RGB::named(rltk::BLACK),
                        lifespan: 400.0
                    }, 
                    Targets::Tile{ tile_idx : map.xy_idx(player_pos.x, player_pos.y - i) as i32 }
                );
            }
        }
    }
}
```

Playing again after these changes made for a much easier game - and one in which I felt like I could progress (but still faced a real risk of death). The dragon still beat me, but it was a *very* close fight - and I nearly won! So I played a few more times, and was victorious once I found a strategy of spell and item use that rewarded me. That's great - that's the type of play a roguelike should have!

I also found that in the earlier levels, I would die if I didn't pay attention - but would generally be victorious if I put my mind to it.

## Wrap Up

So in this chapter, we've built a dragon's lair - and populated it with a nasty dragon. He's just about beatable, but you'll really have to put your mind to it.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-67-dragon)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-67-dragon)
---

Copyright (C) 2019, Herbert Wolverson.

---