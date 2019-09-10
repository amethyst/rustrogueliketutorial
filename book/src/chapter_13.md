# Difficulty

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Currently, you can advance through multiple dungeon levels - but they all have the same spawns. There's no ramp-up of difficulty as you advance, and no easy-mode to get you through the beginning. This chapter aims to change that.

# Adding a wait key

An important tactical element of most roguelikes is the ability to skip a turn - let the monsters come to you (and not get the first hit!). As part of turning the game into a more tactical challenge, lets quickly implement turn skipping. In `player.rs` (along with the rest of the input), we'll add numeric keypad 5 and space to be skip:

```rust
// Skip Turn
VirtualKeyCode::Numpad5 => return RunState::PlayerTurn,
VirtualKeyCode::Space => return RunState::PlayerTurn,
```

This adds a nice tactical dimension to the game: you can lure enemies towards you, and benefit from tactical placement. Another frequently found feature of roguelikes is waiting providing some healing if there are no enemies nearby. We'll only implement that for the player, since mobs suddenly healing up is disconcerting! So we'll change that to:

```rust
// Skip Turn
VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
```

Now we implement `skip_turn`:

```rust
fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}
```

This looks up various entities, and then iterates the player's viewshed using the `tile_content` system. It checks what the player can see for monsters; if no monster is present, it heals the player by 1 hp. This encourages cerebral play - and can be balanced with the inclusion of a hunger clock at a later date. It also makes the game *really easy* - but we're getting to that!

# Increased difficulty as you delve: spawn tables



**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-12-delvingdeeper)**

---

Copyright (C) 2019, Herbert Wolverson.

---