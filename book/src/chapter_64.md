# Cursed Items and Mitigation Thereof

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Now that we have a solid magical items framework, it's time to add in cursed items. These are a mainstay of the Roguelike genre, albeit one that if over-used can really annoy your players! Cursed items are part of the item identification mini-game: they provide a risk to equipping/using an item before you know what it does. If there's no risk to equipping everything you find, the player will do just that to find out what they are - and the mini-game is pointless. On the other hand, if there are *too many* cursed items, the player will become extremely conservative in item use and won't touch things until they know for sure what they are. So, like many things in life, it's a tough balance to strike.

## Your Basic Longsword -1

As a simple example, we'll start by implementing a cursed longsword. We already have a `Longsword +1`, so it's relatively easy to define the JSON (from `spawns.json`) for one that has penalties instead of benefits:

```json
{
    "name" : "Longsword -1",
    "renderable": {
        "glyph" : "/",
        "fg" : "#FFAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "weapon" : {
        "range" : "melee",
        "attribute" : "might",
        "base_damage" : "1d8-1",
        "hit_bonus" : -1
    },
    "weight_lbs" : 2.0,
    "base_value" : 100.0,
    "initiative_penalty" : 3,
    "vendor_category" : "weapon",
    "magic" : { "class" : "common", "naming" : "Unidentified Longsword", "cursed" : true }
},
```

You'll notice that there's a to-hit and damage penalty, more of an initiative penalty, and we've added `cursed: true` to the `magic` section. Most of this already just works, but the `cursed` part is new. To start supporting this, we open up `raws/item_structs.rs` and add in template support:

```rust
#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
    pub cursed: Option<bool>
}
```

We've made it an `Option` - so you don't have to specify it for non-cursed items. Now we need a new component to indicate that an item is, in fact, cursed. In `components.rs` (and registered in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct CursedItem {}
```

Next up, we'll adjust `spawn_named_item` in `raws/rawmaster.rs` to handle adding the `CursedItem` component to cursed items:

```rust
if let Some(magic) = &item_template.magic {
    let class = match magic.class.as_str() {
        "rare" => MagicItemClass::Rare,
        "legendary" => MagicItemClass::Legendary,
        _ => MagicItemClass::Common
    };
    eb = eb.with(MagicItem{ class });

    if !identified.contains(&item_template.name) {
        match magic.naming.as_str() {
            "scroll" => {
                eb = eb.with(ObfuscatedName{ name : scroll_names[&item_template.name].clone() });
            }
            "potion" => {
                eb = eb.with(ObfuscatedName{ name: potion_names[&item_template.name].clone() });
            }
            _ => {
                eb = eb.with(ObfuscatedName{ name : magic.naming.clone() });
            }
        }
    }

    if let Some(cursed) = magic.cursed {
        if cursed { eb = eb.with(CursedItem{}); }
    }
}
```

Let's pop back to `spawns.json` and give them a chance to spawn. For now, we'll make them appear *everywhere* so it's easy to test them:

```json
{ "name" : "Longsword -1", "weight" : 100, "min_depth" : 1, "max_depth" : 100 },
```

That gets us far enough that you can run the game, and cursed longswords will appear and have poor combat performance. Identification already works, so equipping a cursed sword tells you what it is - but there's absolutely no penalty for doing so, other than it having poor stats when you use it. That's a great start!

## Letting the player know that it's cursed

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-64-curses)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-64-curses)
---

Copyright (C) 2019, Herbert Wolverson.

---