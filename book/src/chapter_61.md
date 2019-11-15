# Town Portals

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

We mentioned town portals in the design document, and it's becoming obvious how they would help: it's a real slog to travel back to town to sell your hard-earned loot (and possibly save up for upgrades to help against the itty-bitty draconic murderers!).

The basic idea of a town portal scroll is simple: you cast the spell, a portal opens and takes you back to town. You do your thing in town, and return to the portal - and it teleports you right back to where you were. Depending upon the game, it may heal the monsters on the level while they are gone. Generally, monsters don't follow you through the portal (if they did, you could kill the town with a well-placed portal!).

## Spawning town portal scrolls

We should start by defining them in `spawns.json` as another item:

```json
{
    "name" : "Town Portal Scroll",
    "renderable": {
        "glyph" : ")",
        "fg" : "#AAAAFF",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : {
            "town_portal" : ""
        }
    },
    "weight_lbs" : 0.5,
    "base_value" : 20.0,
    "vendor_category" : "alchemy"
},
```

We should also make them reasonably common in the spawn table:

```json
{ "name" : "Town Portal Scroll", "weight" : 4, "min_depth" : 0, "max_depth" : 100 },
```

That's enough to get them into the game: they spawn as drops, and are purchasable from the alchemist in town (admittedly that doesn't help you when you need one, but with some planning it can help!).

## Implementing town portals

The next stage is to make town portals *do something*. We already added an "effects" tag, causing it to be consumed on use and look for that tag. The other effects use a component to indicate what happens; so we'll open `components.rs` and make a new component type (and register in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TownPortal {}
```

We also need to open up `rawmaster.rs`, and edit `spawn_named_item` to add the tag:

```rust
if let Some(consumable) = &item_template.consumable {
    eb = eb.with(crate::components::Consumable{});
    for effect in consumable.effects.iter() {
        let effect_name = effect.0.as_str();
        match effect_name {
            "provides_healing" => { 
                eb = eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }) 
            }
            "ranged" => { eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }) },
            "damage" => { eb = eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }) }
            "area_of_effect" => { eb = eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }) }
            "confusion" => { eb = eb.with(Confusion{ turns: effect.1.parse::<i32>().unwrap() }) }
            "magic_mapping" => { eb = eb.with(MagicMapper{}) }
            "town_portal" => { eb = eb.with(TownPortal{}) }
            "food" => { eb = eb.with(ProvidesFood{}) }
            _ => {
                println!("Warning: consumable effect {} not implemented.", effect_name);
            }
        }
    }
}
```

All of our level transitions thus far have occurred via `RunState` in `main.rs`. So in `main.rs`, we'll add a new state:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput, 
    PreRun, 
    Ticking, 
    ShowInventory, 
    ShowDropItem, 
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode : VendorMode }
}
```

So that marks the effect. Now we need to make it function! Open up `inventory_system.rs` and we'll want to edit `ItemUseSystem`. After magic mapping, the following code simply logs an event, consumes the item and changes the game state:

```rust
// If its a town portal...
if let Some(_townportal) = town_portal.get(useitem.item) {
    used_item = true;
    gamelog.entries.insert(0, "You are telported back to town!".to_string());
    *runstate = RunState::TownPortal;
}
```

That leaves handling the state in `main.rs`:

```rust

```

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-61-townportal)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-61-townportal)
---

Copyright (C) 2019, Herbert Wolverson.

---