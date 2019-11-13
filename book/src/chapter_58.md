# Item Stats

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the previous chapter we talked about using initiative to make heavy armor have a movement cost, and making some weapons faster than others. The design document also talks about vendors. Finally, what RPG/roguelike is complete without annoying "you are overburdened" messages (and accompanying speed penalties) to make you manage your inventory? These features all point in one direction: additional item statistics, and integrating them into the game systems.

## Defining item information

We already have a component called `Item`; all items have it already, so it seems like the perfect place to add this information! Open up `components.rs`, and we'll edit the `Item` structure to include the information we need for initiative penalties, encumbrance and vendors:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub initiative_penalty : f32,
    pub weight_lbs : f32,
    pub base_value : f32
}
```

So we're defining an `initiative_penalty` - which will be added to your initiative roll to slow you down when equipped (or used, in the case of weapons); `weight_lbs` - which defines how much the item weighs, in pounds; and `base_value` which is the base price of an item in gold pieces (decimal, so we can allow silver also).

We need a way to enter this information, so we open up `raws/item_structs.rs` and edit the `Item` structure:

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
    pub base_value : Option<f32>
}
```

Note that we're making these *optional* - if you don't define them in the `spawns.json` file, they will default to zero. Lastly, we need to fix `raws/rawmaster.rs`'s `spawn_named_item` function to load these values. Replace the line that adds an `Item` with:

```rust
eb = eb.with(crate::components::Item{
    initiative_penalty : item_template.initiative_penalty.unwrap_or(0.0),
    weight_lbs : item_template.weight_lbs.unwrap_or(0.0),
    base_value : item_template.base_value.unwrap_or(0.0)            
});
```

This is taking advantage of `Option`'s `unwrap_or` function - either it returns the wrapped value (if there is one), *or* it returns 0.0. Handy feature to save typing!

These values won't exist until you go into `spawns.json` and start adding them. I've been taking values from [the roll20 compendium](https://roll20.net/compendium/dnd5e/Weapons) for weight and value, and pulling numbers out of the air for initiative penalty. I've entered them [in the source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-58-itemstats/raws/spawns.json) rather than repeat them all here. Here's an example:

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
        "attribute" : "Might",
        "base_damage" : "1d8",
        "hit_bonus" : 0
    },
    "weight_lbs" : 3.0,
    "base_value" : 15.0,
    "initiative_penalty" : 2
},
```

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-58-itemstats)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-58-itemstats)
---

Copyright (C) 2019, Herbert Wolverson.

---