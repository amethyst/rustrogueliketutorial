# Deep Mushroom Forest

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

[![Hands-On Rust](./beta-webBanner.jpg)](https://pragprog.com/titles/hwrust/hands-on-rust/)

---

This chapter will add another level of mushroom grove to the game, this time without a dwarven fortress. It'll also add the final mushroom level, which according to the design document gives way to a dark elven city. Finally, we'll further improve our item story by automating some of the drudge-work going with adding magical and cursed items.

## Building the mushroom forest

We'll start by opening up `map_builders/mod.rs` and adding another line to the map builder calls:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_cavern_builder(new_depth, rng, width, height),
        4 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        5 => limestone_transition_builder(new_depth, rng, width, height),
        6 => dwarf_fort_builder(new_depth, rng, width, height),
        7 => mushroom_entrance(new_depth, rng, width, height),
        8 => mushroom_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}
```

Then we'll open up `map_builders/mushroom_forest.rs` and stub in a basic map builder for the level:

```rust
pub fn mushroom_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into The Mushroom Grove");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(WaveformCollapseBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::RIGHT, YStart::CENTER));
    chain.with(AreaEndingPosition::new(XEnd::LEFT, YEnd::CENTER));
    chain.with(VoronoiSpawning::new());
    chain
}
```

This is basically the same as the other mushroom builder, but without the prefab overlay. If you go into `main.rs` and change the starting level:

```rust
gs.generate_world_map(8, 0);

rltk::main_loop(context, gs)
```

And `cargo run`, you get a pretty passable level. It's retained the mob spawns from our previous level, because we carefully included them in our spawn level ranges.

## End of the Fungal Forest

Once again, we'll add another level into `map_builders/mod.rs`:

```rust
pub fn level_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_cavern_builder(new_depth, rng, width, height),
        4 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        5 => limestone_transition_builder(new_depth, rng, width, height),
        6 => dwarf_fort_builder(new_depth, rng, width, height),
        7 => mushroom_entrance(new_depth, rng, width, height),
        8 => mushroom_builder(new_depth, rng, width, height),
        9 => mushroom_exit(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}
```

And give it the same code to start with as the `mushroom_builder`:

```rust
pub fn mushroom_exit(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into The Mushroom Grove");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(WaveformCollapseBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::RIGHT, YStart::CENTER));
    chain.with(AreaEndingPosition::new(XEnd::LEFT, YEnd::CENTER));
    chain.with(VoronoiSpawning::new());
    chain
}
```

We'll also hit up `main.rs` to make us start on this level:

```rust
gs.generate_world_map(9, 0);
```

Two identical (design-wise; the content will vary due to procedural generation) levels in a row is pretty dull, and we need to convey the idea that there is an entrance to a dark elven city here. We'll start by adding a new prefab sectional to the map:

```rust
#[allow(dead_code)]
pub const DROW_ENTRY : PrefabSection = PrefabSection{
    template : DROW_ENTRY_TXT,
    width: 12,
    height: 10,
    placement: ( HorizontalPlacement::Center, VerticalPlacement::Center )
};

#[allow(dead_code)]
const DROW_ENTRY_TXT : &str = "
            
 ########## 
 #        # 
 #   >    # 
 #        # 
 #e       # 
    e     # 
 #e       # 
 ########## 
            
";
```

Be careful with spaces: there are spaces all around the prefab that are *meant to be there* - to ensure that it has a "gutter" around it. Now we modify our `mushroom_exit` function to spawn it:

```rust
pub fn mushroom_exit(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into The Mushroom Grove");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(WaveformCollapseBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::RIGHT, YStart::CENTER));
    chain.with(AreaEndingPosition::new(XEnd::LEFT, YEnd::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(PrefabBuilder::sectional(DROW_ENTRY));
    chain
}
```

## Unknown glyph loading map: e

You can `cargo run` and find the exit in the middle now, but there are no dark elves! The "e" spawns nothing at all, and generates a warning. That's fine - we haven't implemented any dark elves yet. In `map_builders/prefab_builder/mod.rs`, we'll add `e` to mean "Dark Elf" in the loader file:

```rust
fn char_to_map(&mut self, ch : char, idx: usize, build_data : &mut BuilderMap) {
    // Bounds check
    if idx >= build_data.map.tiles.len()-1 {
        return;
    }
    match ch {
        ' ' => build_data.map.tiles[idx] = TileType::Floor,
        '#' => build_data.map.tiles[idx] = TileType::Wall,
        '≈' => build_data.map.tiles[idx] = TileType::DeepWater,
        '@' => {
            let x = idx as i32 % build_data.map.width;
            let y = idx as i32 / build_data.map.width;
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.starting_position = Some(Position{ x:x as i32, y:y as i32 });
        }
        '>' => build_data.map.tiles[idx] = TileType::DownStairs,
        'e' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Dark Elf".to_string()));
        }
        'g' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Goblin".to_string()));
        }
        'o' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Orc".to_string()));
        }
        'O' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Orc Leader".to_string()));
        }
        '^' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Bear Trap".to_string()));
        }
        '%' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Rations".to_string()));
        }
        '!' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Health Potion".to_string()));
        }
        '☼' => {
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Watch Fire".to_string()));
        }
        _ => {
            rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
        }
    }
}
```

If you `cargo run`, the error is now replaced with `WARNING: We don't know how to spawn [Dark Elf]!` - that's progress.

To solve this, we'll define dark elves! Let's start with a very simple `spawns.json` entry:

```json
{
    "name" : "Dark Elf",
    "renderable": {
        "glyph" : "e",
        "fg" : "#FF0000",
        "bg" : "#000000",
        "order" : 1
    },
    "blocks_tile" : true,
    "vision_range" : 8,
    "movement" : "random_waypoint",
    "attributes" : {},
    "equipped" : [ "Dagger", "Shield", "Leather Armor", "Leather Boots" ],
    "faction" : "DarkElf",
    "gold" : "3d6",
    "level" : 6
},
```

We'll also give them a faction entry:

```json
{ "name" : "DarkElf", "responses" : { "Default" : "attack", "DarkElf" : "ignore" } }
```

If you `cargo run` now, you'll have some moderately powerful dark elves to deal with. The thing is, they aren't very "dark elfy": they are basically reskinned bandits. What do you think of when you think "dark elf" (other than *Drizzt Do'Urden*, whose copyright owners would smite me from afar if I included him)? They are quite evil, magical, fast-moving, and generally quite formidable. They also tend to have their own dark technology, and pepper their enemies with ranged weaponry! 

We aren't going to support ranged weaponry until the next chapter, but we can take some steps to make them more dark elven. Let's give them a more dark-elf sounding set of items. In the `equipped` tag, we'll go with:

```json
"equipped" : [ "Scimitar", "Buckler", "Drow Chain", "Drow Leggings", "Drow Boots" ],
```

We'll also need to make item entries for these. We'll make the *scimitar* basically a longsword, but a little nicer:

```json
{
    "name" : "Scimitar",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "attribute" : "might",
        "base_damage" : "1d6+2",
        "hit_bonus" : 1
    },
    "weight_lbs" : 2.5,
    "base_value" : 25.0,
    "initiative_penalty" : 1,
    "vendor_category" : "weapon"
},
```

We'll follow the trend for the *Drow Armor*: it's basically chain armor, but with much less initiative penalty:

```json
{
    "name" : "Drow Leggings",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00FFFF",
        "bg" : "#000000",
        "order" : 2
    },
    "wearable" : {
        "slot" : "Legs",
        "armor_class" : 0.4
    },
    "weight_lbs" : 10.0,
    "base_value" : 50.0,
    "initiative_penalty" : 0.1,
    "vendor_category" : "clothes"
},

{
    "name" : "Drow Chain",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00FF00",
        "bg" : "#000000",
        "order" : 2
    },
    "wearable" : {
        "slot" : "Torso",
        "armor_class" : 3.0
    },
    "weight_lbs" : 5.0,
    "base_value" : 50.0,
    "initiative_penalty" : 0.0,
    "vendor_category" : "armor"
},

{
    "name" : "Drow Boots",
    "renderable": {
        "glyph" : "[",
        "fg" : "#00FF00",
        "bg" : "#000000",
        "order" : 2
    },
    "wearable" : {
        "slot" : "Feet",
        "armor_class" : 0.4
    },
    "weight_lbs" : 2.0,
    "base_value" : 10.0,
    "initiative_penalty" : 0.1,
    "vendor_category" : "armor"
},
```

The result of these is that they are *fast* - they have much less initiative penalty than a similarly armored player. The other nice thing is that you can kill one, take their stuff - and have the same benefit!

At this point, we've added two playable levels - in only a few lines of code. Reaping the benefits of working so hard on a generic system! So now, let's make things a little more generic - and save ourselves some typing.

## Procedurally Generated Magical Items

We've been adding "Longsword +1", "Longsword -1", etc. quite a bit. We could sit and laboriously type out every magical variant of every item, and we'd have a pretty playable game. OR - we could automate some of the grunt work!

What if we could append a "template" attribute to a weapon definition in `spawns.json`, and have it automatically generate the variants for us? This isn't as far-fetched as it sounds. Let's sketch out what we'd like:

```json
{
    "name" : "Longsword",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "attribute" : "might",
        "base_damage" : "1d8",
        "hit_bonus" : 0
    },
    "weight_lbs" : 3.0,
    "base_value" : 15.0,
    "initiative_penalty" : 2,
    "vendor_category" : "weapon",
    "template_magic" : {
        "unidentified_name" : "Unidentified Longsword",
        "bonus_min" : 1,
        "bonus_max" : 5,
        "include_cursed" : true
    }
},
```

So we've added a `template_magic` section, describing the types of items we'd like to add. We need to extend `raws/item_structs.rs` to support loading this information:

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub wearable : Option<Wearable>,
    pub initiative_penalty : Option<f32>,
    pub weight_lbs : Option<f32>,
    pub base_value : Option<f32>,
    pub vendor_category : Option<String>,
    pub magic : Option<MagicItem>,
    pub attributes : Option<ItemAttributeBonus>,
    pub template_magic : Option<ItemMagicTemplate>
}
...
#[derive(Deserialize, Debug, Clone)]
pub struct ItemMagicTemplate {
    pub unidentified_name: String,
    pub bonus_min: i32,
    pub bonus_max: i32,
    pub include_cursed: bool
}
```

That's enough to load the extra information - it just doesn't do anything. We also need to go through and add `Clone` to the `#[derive]` list for all the structures in that file. We'll be using `clone()` to make a copy to then modify for each variant.

Unlike other additions, this doesn't modify our `spawn_named_item` function in `rawmaster.rs`; we want to modify the raw file templates *before* we get to spawning. Instead, we're going to post-process the item list built by the `load` function itself (including modifying the spawns list). At the top of the function, we'll read through every item and if it has the template attached (and is a weapon or armor item), we'll add it to a list to process:

```rust
pub fn load(&mut self, raws : Raws) {
    self.raws = raws;
    self.item_index = HashMap::new();
    let mut used_names : HashSet<String> = HashSet::new();

    struct NewMagicItem {
        name : String,
        bonus : i32
    }
    let mut items_to_build : Vec<NewMagicItem> = Vec::new();

    for (i,item) in self.raws.items.iter().enumerate() {
        if used_names.contains(&item.name) {
            rltk::console::log(format!("WARNING -  duplicate item name in raws [{}]", item.name));
        }
        self.item_index.insert(item.name.clone(), i);
        used_names.insert(item.name.clone());

        if let Some(template) = &item.template_magic {
            if item.weapon.is_some() || item.wearable.is_some() {
                if template.include_cursed {
                    items_to_build.push(NewMagicItem{
                        name : item.name.clone(),
                        bonus : -1
                    });
                }
                for bonus in template.bonus_min ..= template.bonus_max {
                    items_to_build.push(NewMagicItem{
                        name : item.name.clone(),
                        bonus
                    });
                }
            } else {
                rltk::console::log(format!("{} is marked as templated, but isn't a weapon or armor.", item.name));
            }
        }
    }
```

Then, after we're done with reading the items we'll add a loop to the end to create these items:

```rust
for nmw in items_to_build.iter() {
    let base_item_index = self.item_index[&nmw.name];
    let mut base_item_copy = self.raws.items[base_item_index].clone();

    if nmw.bonus == -1 {
        base_item_copy.name = format!("{} -1", nmw.name);
    } else {
        base_item_copy.name = format!("{} +{}", nmw.name, nmw.bonus);
    }

    base_item_copy.magic = Some(super::MagicItem{
        class : match nmw.bonus {
            2 => "rare".to_string(),
            3 => "rare".to_string(),
            4 => "rare".to_string(),
            5 => "legendary".to_string(),
            _ => "common".to_string()
        },
        naming : base_item_copy.template_magic.as_ref().unwrap().unidentified_name.clone(),
        cursed: if nmw.bonus == -1 { Some(true) } else { None }
    });

    if let Some(initiative_penalty) = base_item_copy.initiative_penalty.as_mut() {
        *initiative_penalty -= nmw.bonus as f32;
    }
    if let Some(base_value) = base_item_copy.base_value.as_mut() {
        *base_value += (nmw.bonus as f32 + 1.0) * 50.0;
    }
    if let Some(mut weapon) = base_item_copy.weapon.as_mut() {
        weapon.hit_bonus += nmw.bonus;
        let (n,die,plus) = parse_dice_string(&weapon.base_damage);
        let final_bonus = plus+nmw.bonus;
        if final_bonus > 0 {
            weapon.base_damage = format!("{}d{}+{}", n, die, final_bonus);
        } else if final_bonus < 0 {
            weapon.base_damage = format!("{}d{}-{}", n, die, i32::abs(final_bonus));
        }
    }
    if let Some(mut armor) = base_item_copy.wearable.as_mut() {
        armor.armor_class += nmw.bonus as f32;
    }

    let real_name = base_item_copy.name.clone();
    self.raws.items.push(base_item_copy);
    self.item_index.insert(real_name.clone(), self.raws.items.len()-1);

    self.raws.spawn_table.push(super::SpawnTableEntry{
        name : real_name.clone(),
        weight : 10 - i32::abs(nmw.bonus),
        min_depth : 1 + i32::abs((nmw.bonus-1)*3),
        max_depth : 100,
        add_map_depth_to_weight : None
    });
}
```

So this loops through all of the "Longsword +1", "Longsword -1", "Longsword +2" etc. that we created during the initial parsing. It then:

1. Takes a copy of the original item.
2. If the bonus is `-1`, it renames it "Item -x"; otherwise it renamed it "Item +x" where *x* is the bonus.
3. It creates a new `magic` entry for the item, and sets the common/rare/legendary status by bonus and sets the cursed flag as appropriate.
4. If the item has an initiative penalty, it *subtracts* the bonus from it (making cursed items worse, magical items better).
5. It ups the base value by bonus +1 * 50 gold.
6. If its a weapon, it adds the bonus to the `to_hit` bonus and damage dice. It does the damage dice by reformatting the dice number.
7. If its armor, it adds the bonus to the armor class.
8. It then inserts the new item into the spawn table, with a lower weight for better items and better items appearing later in the dungeon.

If you check the [online source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-69-mushrooms2/raws/spawns.json) - I've gone through and *removed* all the +1, +2 and simple cursed armor and weapons - and appended the `template_magic` to each of them. This results in the generation of 168 new items! That's a LOT better than typing them all in.

If you `cargo run` now, you'll find gradually improving magical items of all types throughout the dungeon. Nicer items appear as you get deeper into the dungeon, so there's a nice ramp-up in player power.

## Trait Items

With the `dagger of venom`, we introduced a new type of item: one that inflicts an effect when you hit. Given that this can be any effect in the game, there's a lot of possibilities for effects! Manually adding in all of the effects would *take a while* - it's probably quicker to come up with a generic system, and have real variety in our items as a result (as well as not forgetting to add them!).

Let's get started by adding a new section to `spawns.json`, dedicated to *weapon traits*:

```json
"weapon_traits" : [
    {
        "name" : "Venomous",
        "effects" : { "damage_over_time" : "2" }
    }
]
```

We'll add more traits later, for now we'll focus on making the system work at all! To read the data, we'll make a new file, `raws/weapon_traits.rs` (don't get confused by Rust traits and weapon traits; they aren't the same thing at all). We'll put in enough structure to allow Serde to read the JSON file:

```rust
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct WeaponTrait {
    pub name : String,
    pub effects : HashMap<String, String>
}
```

Now we need to extend the data in `raws/mod.rs` to include it. At the top of the file, include:

```rust
mod weapon_traits;
pub use weapon_traits::*;
```

And then we'll add it into the `Raws` structure, just like we did for spells:

```rust
#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>,
    pub mobs : Vec<Mob>,
    pub props : Vec<Prop>,
    pub spawn_table : Vec<SpawnTableEntry>,
    pub loot_tables : Vec<LootTable>,
    pub faction_table : Vec<FactionInfo>,
    pub spells : Vec<Spell>,
    pub weapon_traits : Vec<WeaponTrait>
}
```

In turn, we have to extend the constructor in `raws/rawmaster.rs` to include an empty traits list:

```rust
impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table : Vec::new(),
                spells : Vec::new(),
                weapon_traits : Vec::new()
            },
            item_index : HashMap::new(),
            mob_index : HashMap::new(),
            prop_index : HashMap::new(),
            loot_index : HashMap::new(),
            faction_index : HashMap::new(),
            spell_index : HashMap::new()
        }
    }
    ...
```

Thanks to the magic of Serde, that's all there is to actually *loading* the data! Now for the hard part: procedurally generating magic items that feature one or more traits. To avoid repeating ourselves, we're going to separate the code we wrote previously into reusable functions:

```rust
// Put this above the raws implementation
struct NewMagicItem {
    name : String,
    bonus : i32
}
...

// Inside the raws implementation
fn append_magic_template(items_to_build : &mut Vec<NewMagicItem>, item : &super::Item) {
    if let Some(template) = &item.template_magic {
        if item.weapon.is_some() || item.wearable.is_some() {
            if template.include_cursed {
                items_to_build.push(NewMagicItem{
                    name : item.name.clone(),
                    bonus : -1
                });
            }
            for bonus in template.bonus_min ..= template.bonus_max {
                items_to_build.push(NewMagicItem{
                    name : item.name.clone(),
                    bonus
                });
            }
        } else {
            rltk::console::log(format!("{} is marked as templated, but isn't a weapon or armor.", item.name));
        }
    }
}

fn build_base_magic_item(&self, nmw : &NewMagicItem) -> super::Item {
    let base_item_index = self.item_index[&nmw.name];
    let mut base_item_copy = self.raws.items[base_item_index].clone();
    base_item_copy.vendor_category = None; // Don't sell magic items!

    if nmw.bonus == -1 {
        base_item_copy.name = format!("{} -1", nmw.name);
    } else {
        base_item_copy.name = format!("{} +{}", nmw.name, nmw.bonus);
    }

    base_item_copy.magic = Some(super::MagicItem{
        class : match nmw.bonus {
            2 => "rare".to_string(),
            3 => "rare".to_string(),
            4 => "rare".to_string(),
            5 => "legendary".to_string(),
            _ => "common".to_string()
        },
        naming : base_item_copy.template_magic.as_ref().unwrap().unidentified_name.clone(),
        cursed: if nmw.bonus == -1 { Some(true) } else { None }
    });

    if let Some(initiative_penalty) = base_item_copy.initiative_penalty.as_mut() {
        *initiative_penalty -= nmw.bonus as f32;
    }
    if let Some(base_value) = base_item_copy.base_value.as_mut() {
        *base_value += (nmw.bonus as f32 + 1.0) * 50.0;
    }
    if let Some(mut weapon) = base_item_copy.weapon.as_mut() {
        weapon.hit_bonus += nmw.bonus;
        let (n,die,plus) = parse_dice_string(&weapon.base_damage);
        let final_bonus = plus+nmw.bonus;
        if final_bonus > 0 {
            weapon.base_damage = format!("{}d{}+{}", n, die, final_bonus);
        } else if final_bonus < 0 {
            weapon.base_damage = format!("{}d{}-{}", n, die, i32::abs(final_bonus));
        }
    }
    if let Some(mut armor) = base_item_copy.wearable.as_mut() {
        armor.armor_class += nmw.bonus as f32;
    }
    base_item_copy
}

fn build_magic_weapon_or_armor(&mut self, items_to_build : &[NewMagicItem]) {
    for nmw in items_to_build.iter() {
        let base_item_copy = self.build_base_magic_item(&nmw);

        let real_name = base_item_copy.name.clone();
        self.raws.items.push(base_item_copy);
        self.item_index.insert(real_name.clone(), self.raws.items.len()-1);

        self.raws.spawn_table.push(super::SpawnTableEntry{
            name : real_name.clone(),
            weight : 10 - i32::abs(nmw.bonus),
            min_depth : 1 + i32::abs((nmw.bonus-1)*3),
            max_depth : 100,
            add_map_depth_to_weight : None
        });
    }
}

fn build_traited_weapons(&mut self, items_to_build : &[NewMagicItem]) {
    items_to_build.iter().filter(|i| i.bonus > 0).for_each(|nmw| {
        for wt in self.raws.weapon_traits.iter() {
            let mut base_item_copy = self.build_base_magic_item(&nmw);
            if let Some(mut weapon) = base_item_copy.weapon.as_mut() {
                base_item_copy.name = format!("{} {}", wt.name, base_item_copy.name);
                if let Some(base_value) = base_item_copy.base_value.as_mut() {
                    *base_value *= 2.0;
                }
                    weapon.proc_chance = Some(0.25);
                    weapon.proc_effects = Some(wt.effects.clone());

                let real_name = base_item_copy.name.clone();
                self.raws.items.push(base_item_copy);
                self.item_index.insert(real_name.clone(), self.raws.items.len()-1);

                self.raws.spawn_table.push(super::SpawnTableEntry{
                    name : real_name.clone(),
                    weight : 9 - i32::abs(nmw.bonus),
                    min_depth : 2 + i32::abs((nmw.bonus-1)*3),
                    max_depth : 100,
                    add_map_depth_to_weight : None
                });
            }
        }
    });
}

pub fn load(&mut self, raws : Raws) {
    self.raws = raws;
    self.item_index = HashMap::new();
    let mut used_names : HashSet<String> = HashSet::new();
    let mut items_to_build = Vec::new();

    for (i,item) in self.raws.items.iter().enumerate() {
        if used_names.contains(&item.name) {
            rltk::console::log(format!("WARNING -  duplicate item name in raws [{}]", item.name));
        }
        self.item_index.insert(item.name.clone(), i);
        used_names.insert(item.name.clone());

        RawMaster::append_magic_template(&mut items_to_build, item);
    }
    for (i,mob) in self.raws.mobs.iter().enumerate() {
        if used_names.contains(&mob.name) {
            rltk::console::log(format!("WARNING -  duplicate mob name in raws [{}]", mob.name));
        }
        self.mob_index.insert(mob.name.clone(), i);
        used_names.insert(mob.name.clone());
    }
    for (i,prop) in self.raws.props.iter().enumerate() {
        if used_names.contains(&prop.name) {
            rltk::console::log(format!("WARNING -  duplicate prop name in raws [{}]", prop.name));
        }
        self.prop_index.insert(prop.name.clone(), i);
        used_names.insert(prop.name.clone());
    }

    for spawn in self.raws.spawn_table.iter() {
        if !used_names.contains(&spawn.name) {
            rltk::console::log(format!("WARNING - Spawn tables references unspecified entity {}", spawn.name));
        }
    }

    for (i,loot) in self.raws.loot_tables.iter().enumerate() {
        self.loot_index.insert(loot.name.clone(), i);
    }

    for faction in self.raws.faction_table.iter() {
        let mut reactions : HashMap<String, Reaction> = HashMap::new();
        for other in faction.responses.iter() {
            reactions.insert(
                other.0.clone(),
                match other.1.as_str() {
                    "ignore" => Reaction::Ignore,
                    "flee" => Reaction::Flee,
                    _ => Reaction::Attack
                }
            );
        }
        self.faction_index.insert(faction.name.clone(), reactions);
    }

    for (i,spell) in self.raws.spells.iter().enumerate() {
        self.spell_index.insert(spell.name.clone(), i);
    }

    self.build_magic_weapon_or_armor(&items_to_build);
    self.build_traited_weapons(&items_to_build);
}
```

You'll notice that there is a new function in there `build_traited_weapons`. This iterates through the magic items, filtering weapons only - and only those with a bonus (I don't really want to get into what a cursed venomous dagger does, just yet). It reads through all of the traits and makes a (rarer) version of each magical weapon with that trait applied.

Let's go ahead and add one more trait to `spawns.json`:

```json
"weapon_traits" : [
    {
        "name" : "Venomous",
        "effects" : { "damage_over_time" : "2" }
    },
    {
        "name" : "Dazzling",
        "effects" : { "confusion" : "2" }
    }
]
```

If you `cargo run` and play now, you'll sometimes find such wonders as *Dazzling Longsword +1*, or *Venomous Dagger +2*.

## Wrap-Up

In this chapter, we've built ourselves a mushroom grove level, and a second level transitioning to the dark elven stronghold. We've started to add dark elves, and to power-up (and save typing) we're automatically generating magical items from -1 to +5. We then generated "traited" versions of the same weapons. Now there's a *huge* amount of variety between runs, which should keep the gear-oriented player happy. There's also a nice progression of levels, and we're ready to tackle the dark elven city - and ranged weaponry!

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-69-mushrooms2)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-69-mushrooms2)
---

Copyright (C) 2019, Herbert Wolverson.

---