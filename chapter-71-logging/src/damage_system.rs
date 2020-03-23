use specs::prelude::*;
use super::{Pools, Player, Name, RunState, Position,
    InBackpack, Equipped, LootTable};

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            crate::gamelog::Logger::new()
                                .color(rltk::RED)
                                .append(&victim_name.name)
                                .append("is dead!")
                                .log();
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // Drop everything held by dead people
    let mut to_spawn : Vec<(String, Position)> = Vec::new();
    { // To avoid keeping hold of borrowed entries, use a scope
        let mut to_drop : Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
        for victim in dead.iter() {
            let pos = positions.get(*victim);
            for (entity, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }
            for (entity, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }

            if let Some(table) = loot_tables.get(*victim) {
                let drop_finder = crate::raws::get_item_drop(
                    &crate::raws::RAWS.lock().unwrap(),
                    &mut rng,
                    &table.table
                );
                if let Some(tag) = drop_finder {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, pos.clone()));
                    }
                }
            }
        }

        for drop in to_drop.iter() {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert position");
        }
    }

    {
        for drop in to_spawn.iter() {
            crate::raws::spawn_named_item(
                &crate::raws::RAWS.lock().unwrap(),
                ecs,
                &drop.0,
                crate::raws::SpawnType::AtPosition{x : drop.1.x, y: drop.1.y}
            );
        }
    }

    // Fire death events
    use crate::effects::*;
    use crate::Map;
    use crate::components::{OnDeath, AreaOfEffect};
    for victim in dead.iter() {
        let death_effects = ecs.read_storage::<OnDeath>();
        if let Some(death_effect) = death_effects.get(*victim) {
            let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
            for effect in death_effect.abilities.iter() {
                if rng.roll_dice(1,100) <= (effect.chance * 100.0) as i32 {
                    let map = ecs.fetch::<Map>();
                    if let Some(pos) = ecs.read_storage::<Position>().get(*victim) {
                        let spell_entity = crate::raws::find_spell_entity(ecs, &effect.spell).unwrap();
                        let tile_idx = map.xy_idx(pos.x, pos.y);
                        let target = 
                            if let Some(aoe) = ecs.read_storage::<AreaOfEffect>().get(spell_entity) {
                                Targets::Tiles { tiles : aoe_tiles(&map, rltk::Point::new(pos.x, pos.y), aoe.radius) }
                            } else {
                                Targets::Tile{ tile_idx : tile_idx as i32 }
                            };
                        add_effect(
                            None,
                            EffectType::SpellUse{ spell: crate::raws::find_spell_entity( ecs, &effect.spell ).unwrap() },
                            target
                        );
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
