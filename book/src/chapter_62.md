# Magic Items and Item Identification

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Magical items are a mainstay of both D&D and roguelikes. From the humble "Sword +1", to mighty "Holy Avenger" - and back again to "Cursed Backbiter" - items helped to define the genre. In roguelikes, it's also traditional to not automatically know what items are; you find an "Unidentified Longsword", and have no idea what it does (or if is cursed) until you find a way to identify it. You find a "Scroll of *cat walked on keyboard*" (the unpronounceable names seem to be a feature!), and until you identify or read it - you don't know what to expect. Some games turn this into entire meta-games - gambling on frequency, vendor prices and similar to give you clues as to what you just found. Even *Diablo*, the most mainstream roguelike (even if it went real-time!) of them all has retained this play feature - but tends to make Identify scrolls extremely plentiful (as well as helpful old Scotsmen).

## Classes of magic item

It's common in modern games to differentiate magic items as being *magical*, *rare* or *legendary* (along with item sets, which we won't go into yet). These are typically differentiated by color, so you can tell at a glance if an item is even worth considering. This also gives an opportunity to denote that something *is* a magic item - so we'll open up `components.rs` (and register in `main.rs` and `saveload_system.rs`) and make `MagicItem`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum MagicItemClass { Common, Rare, Legendary }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {
    pub class : MagicItemClass
}
```

The next step is to let items be denoted as magical, and having one of these classes. Add the following to `raws/item_structs.rs`:

```rust
#[derive(Deserialize, Debug)]
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
    pub magic : Option<MagicItem>
}

#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String
}
```

Why are we using a full struct here, rather than just a string? We're going to want to specify more information here later in the chapter as we start to flesh out magical items.

You can now decorate items in `spawns.json`, for example:

```json
{
    "name" : "Health Potion",
    "renderable": {
        "glyph" : "!",
        "fg" : "#FF00FF",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : { "provides_healing" : "8" }
    },
    "weight_lbs" : 0.5,
    "base_value" : 50.0,
    "vendor_category" : "alchemy",
    "magic" : { "class" : "common" }
},
```

I've added the `common` magic tag to the magical scrolls and potions already in the JSON list, see the source for details - it's pretty straightforward at this point. Next up, we need to modify `spawn_named_item` in `raws/rawmaster.rs` to apply the appropriate component tags:

```rust
if let Some(magic) = &item_template.magic {
    let class = match magic.class.as_str() {
        "rare" => MagicItemClass::Rare,
        "legendary" => MagicItemClass::Legendary,
        _ => MagicItemClass::Common
    };
    eb = eb.with(MagicItem{ class });
}
```

Now that we have this data, we need to *use* it. For now, we'll just want to set the display *color* of item names whenever they appear in the GUI - to give a better idea of magic item value (just like all those MMO games!). In `gui.rs`, we'll make a generic function for this purpose:

```rust
pub fn get_item_color(ecs : &World, item : Entity) -> RGB {
    if let Some(magic) = ecs.read_storage::<MagicItem>().get(item) {
        match magic.class {
            MagicItemClass::Common => return RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => return RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => return RGB::from_f32(0.71, 0.15, 0.93)
        }
    }
    RGB::from_f32(1.0, 1.0, 1.0)
}
```

Now we need to go through all of the functions in `gui.rs` that display an item name, and replace the hard-coded color with a call to this function. In `draw_ui` (line 121 of `gui.rs`), expand the equipped list a little:

```rust
// Equipped
let mut y = 13;
let entities = ecs.entities();
let equipped = ecs.read_storage::<Equipped>();
let name = ecs.read_storage::<Name>();
for (entity, equipped_by, item_name) in (&entities, &equipped, &name).join() {
    if equipped_by.owner == *player_entity {
        ctx.print_color(50, y, get_item_color(ecs, entity), black, &item_name.name);
        y += 1;
    }
}
```

The same change in the consumables section:

```rust
ctx.print_color(53, y, get_item_color(ecs, entity), black, &item_name.name);
```

We're going to leave tooltips alone, improving them (and the log) are the subject of a (currently hypothetical) future chapter. In `show_inventory` (around line 321), `drop_item_menu` (around line 373), `remove_item_menu` (around line 417), and `vendor_sell_menu` (around line 660):

```rust
ctx.print_color(21, y, get_item_color(&gs.ecs, entity), RGB::from_f32(0.0, 0.0, 0.0), &name.name.to_string());
```

Be warned: these lines will change *again* once we add item identification!

With that in place, if you `cargo run` you will see your `Town Portal Scroll` is now nicely highlighted as a common magical item:

![Screenshot](./c62-s1.jpg)

## Identification: Scrolls

It's pretty common in Roguelikes for potions to have thoroughly unpronounceable names when you don't know what they do. Presumably, this represents some sort of guttural utterings that trigger the magical effect (and as much fun as it would be to build a giant grammar around this, the tutorial would be even bigger!). So a *Scroll of Lorem Ipsum* might be *any* of the scrolls in the game, and it's up to you to decide to identify by using it (a gamble, it may not be what you want at all!), get it identified, or just ignore it because you don't like the risk.

Let's start by opening up `spawner.rs`, going to the `player` function and removing the line that gives a free `Town Portal`. It's overly generous, and would mean you'd have to start knowing what it is!

So here's the fun part: if we were to simply assign an unidentified name to scrolls, players could simply learn the names - and identification would be little more than a memory game. So we need to assign the names *when the game starts* (and not when the raw files load, since you may play more than once per session). Let's start in `raws/item_structs.rs` and add another field to `MagicItem` indicating that "this is a scroll, and should use scroll naming."

```rust
#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String
}
```

Now we have to go through `spawns.json` and add naming tags to our "magic" entries. I've opted for "scroll" for naming scrolls (and left the others as empty strings for now). For example, here's the magic missile scroll:

```json
{
    "name" : "Magic Missile Scroll",
    "renderable": {
        "glyph" : ")",
        "fg" : "#00FFFF",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : { 
            "ranged" : "6",
            "damage" : "20"
        }
    },
    "weight_lbs" : 0.5,
    "base_value" : 50.0,
    "vendor_category" : "alchemy",
    "magic" : { "class" : "common", "naming" : "scroll" }
},
```

We already have a structure that persists through the whole game (but remains a global resource), and resets whenever we change level: the `MasterDungeonMap`. It makes some sense to use this to store state about the whole game, since it's already the dungeon master! We're also already serializing it, which helps a lot!

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-62-magictems)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-62-magicitems)
---

Copyright (C) 2019, Herbert Wolverson.

---