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
    if map.depth == 1 {
        gamelog.entries.insert(0, "You are already in town, so the scroll does nothing.".to_string());
    } else {
        used_item = true;
        gamelog.entries.insert(0, "You are telported back to town!".to_string());
        *runstate = RunState::TownPortal;
    }
}
```

That leaves handling the state in `main.rs`:

```rust
RunState::TownPortal => {
    // Spawn the portal
    spawner::spawn_town_portal(&mut self.ecs);

    // Transition
    let map_depth = self.ecs.fetch::<Map>().depth;
    let destination_offset = 0 - (map_depth-1);
    self.goto_level(destination_offset);
    self.mapgen_next_state = Some(RunState::PreRun);
    newrunstate = RunState::MapGeneration;
}
```

So this is relatively straight-forward: it calls the as-yet-unwritten `spawn_town_portal` function, retrieves the depth, and uses the same logic as `NextLevel` and `PreviousLevel` to switch to the town level (the offset calculated to result in a depth of 1). The rabbit hole naturally leads us to `spawner.rs`, and the `spawn_town_portal` function. Let's write it:

```rust
pub fn spawn_town_portal(ecs: &mut World) {
    // Get current position & depth
    let map = ecs.fetch::<Map>();
    let player_depth = map.depth;
    let player_pos = ecs.fetch::<rltk::Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // Find part of the town for the portal
    let dm = ecs.fetch::<MasterDungeonMap>();
    let town_map = dm.get_map(1).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
        }
    }
    let portal_x = (stairs_idx as i32 % town_map.width)-2;
    let portal_y = stairs_idx as i32 / town_map.width;

    std::mem::drop(dm);

    // Spawn the portal itself
    ecs.create_entity()
        .with(OtherLevelPosition { x: portal_x, y: portal_y, depth: 1 })
        .with(Renderable {
            glyph: rltk::to_cp437('♥'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(EntryTrigger{})
        .with(TeleportTo{ x: player_x, y: player_y, depth: player_depth, player_only: true })
        .with(Name{ name : "Town Portal".to_string() })
        .build();
}
```

This is a busy function, so we'll step through it:

1. We retrieve the player's depth and position, and then drop access to the resources (to prevent the borrow from continuing).
2. We look up the town map in the `MasterDungeonMap`, and find the spawn point. We move two tiles to the west, and store that as `portal_x` and `portal_y`. We then drop access to the dungeon map, again to avoid keeping the borrow.
3. We create an entity for the portal. We give it an `OtherLevelPosition`, indicating that it is in the town - at the coordinates we calculated. We give it a `Renderable` (a cyan heart), a `Name` (so it shows up in tooltips). We also give it an `EntryTrigger` - so entering it will trigger an effect. Finally, we give it a `TeleportTo` component; we haven't written that yet, but you can see we're specifying destination coordinates (back to where the player started). There's also a `player_only` setting - if the teleporter works for everyone, town drunks might walk into the portal by mistake leading to the (hilarious) situation where they teleport into dungeons and die horribly. To avoid that, we'll make this teleporter only affect the player!

Since we've used it, we better make `TeleportTo` in `components.rs` (and registered in `main.rs` and `saveload_system.rs`). It's pretty simple:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only : bool
}
```

We'll worry about making teleporters work in a moment.

To help test the systems, we'll start the player with a town portal scroll. In `spawner.rs`, we'll modify `player`:

```rust
spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Town Portal Scroll", SpawnType::Carried{by : player});
```

If you `cargo run` now, you start with a `Town Portal Scroll`. Trying to use it in town gives you a "does nothing" message. Going to another level and then using it teleports you right back to town, with a portal present - exactly what we had in mind (but with no way back, yet):

![Screenshot](./c61-s1.gif)

## Implementing teleporters

Now we need to make the portal go *back* to your point-of-origin in the dungeon. Since we've implemented triggers that can have `TeleportTo`, it's worth taking the time to make teleport triggers more general (so you could have teleport traps, for example - or inter-room teleporters, or even a portal to the final level). There's actually a lot to consider here:

* Teleporters can affect anyone who enters the tile, *unless* you've flagged them as "player only".
* Teleporting could happen across the current level, in which case it's like a regular move.
* Teleporting could also happen across levels, in which case there are two possibilities:
    * The player is teleporting, and we need to adjust game state like other level transitions.
    * Another entity is teleporting, in which case we need to remove its `Position` component and add an `OtherLevelPosition` component so they are in-place when the player goes there.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-61-townportal)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-61-townportal)
---

Copyright (C) 2019, Herbert Wolverson.

---