# Magic Mapping

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

A really common item in roguelikes is the *scroll of magic mapping*. You read it, and the dungeon is revealed. Fancier roguelikes have nice graphics for it. In this chapter, we'll start by making it work - and then make it pretty!

## Adding a magic map component

We have everything we need except for an indicator that an item is a scroll (or any other item, really) of magic mapping. So in `components.rs` we'll add a component for it:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}
```

As always, we need to register it in `main.rs` and `saveload_system.rs`. We'll head over to `spawners.rs` and create a new function for it, as well as adding it to the loot tables:

```rust
fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN3),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Scroll of Magic Mapping".to_string() })
        .with(Item{})
        .with(MagicMapper{})
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
```

And the loot table:

```rust
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 400)
}
```

Notice that we've given it a weight of 400 - absolutely ridiculous. We'll fix it later, for now we *really* want to spawn the scroll so that we can test it! Lastly, we add it to the actual spawn function:

```rust
match spawn.1.as_ref() {
    "Goblin" => goblin(ecs, x, y),
    "Orc" => orc(ecs, x, y),
    "Health Potion" => health_potion(ecs, x, y),
    "Fireball Scroll" => fireball_scroll(ecs, x, y),
    "Confusion Scroll" => confusion_scroll(ecs, x, y),
    "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
    "Dagger" => dagger(ecs, x, y),
    "Shield" => shield(ecs, x, y),
    "Longsword" => longsword(ecs, x, y),
    "Tower Shield" => tower_shield(ecs, x, y),
    "Rations" => rations(ecs, x, y),
    "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
    _ => {}
}
```

If you were to `cargo run` now, you'd likely find scrolls you can pick up - but they won't do anything.

## Mapping the level - the simple version

We'll modify `inventory_system.rs` to detect if you just used a mapping scroll, and reveal the whole map:

```rust
// If its a magic mapper...
let is_mapper = magic_mapper.get(useitem.item);
match is_mapper {
    None => {}
    Some(_) => {
        used_item = true;
        for r in map.revealed_tiles.iter_mut() {
            *r = true;
        }
        gamelog.entries.insert(0, "The map is revealed to you!".to_string());
    }
}
```

There are some framework changes also (see the source); we've done this often enough, I don't think it needs repeating here again. If you `cargo run` the project now, find a scroll (they are *everywhere*) and use it - the map is instantly revealed:

![Screenshot](./c20-s1.gif)

## Making it pretty


**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-20-magicmapping)**

---

Copyright (C) 2019, Herbert Wolverson.

---