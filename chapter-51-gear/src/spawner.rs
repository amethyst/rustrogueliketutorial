extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };
extern crate specs;
use specs::prelude::*;
use super::{Pools, Pool, Player, Renderable, Name, Position, Viewshed, Rect,  
    SerializeMe, random_table::RandomTable, HungerClock, HungerState, Map, TileType, raws::*,
    Attribute, Attributes, Skills, Skill, EquipmentSlot };
use crate::specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;
use crate::{attr_bonus, player_hp_at_level, mana_at_level};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    let mut skills = Skills{ skills: HashMap::new() };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);

    let player = ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(HungerClock{ state: HungerState::WellFed, duration: 20 })
        .with(Attributes{
            might: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            fitness: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            quickness: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11) },
        })
        .with(skills)
        .with(Pools{
            hit_points : Pool{ 
                current: player_hp_at_level(11, 1), 
                max: player_hp_at_level(11, 1) 
            },
            mana: Pool{
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1)
            },
            xp: 0,
            level: 1
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Starting equipment
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Rusty Longsword", 
        SpawnType::Equipped{by : player, slot: EquipmentSlot::Melee}
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Dried Sausage", 
        SpawnType::Carried{by : player}
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Beer", 
        SpawnType::Carried{by : player}
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Stained Tunic", 
        SpawnType::Equipped{by : player, slot: EquipmentSlot::Torso}
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Torn Trousers", 
        SpawnType::Equipped{by : player, slot: EquipmentSlot::Legs}
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), "Old Boots", 
        SpawnType::Equipped{by : player, slot: EquipmentSlot::Feet}
    );

    player
}

const MAX_MONSTERS : i32 = 4;

fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}

/// Fills a room with stuff!
pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room : &Rect, map_depth: i32, spawn_list : &mut Vec<(usize, String)>) {
    let mut possible_targets : Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

/// Fills a region with stuff!
pub fn spawn_region(_map: &Map, rng: &mut RandomNumberGenerator, area : &[usize], map_depth: i32, spawn_list : &mut Vec<(usize, String)>) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };

            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result = spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), &spawn.1, SpawnType::AtPosition{ x, y});
    if spawn_result.is_some() {
        return;
    }

    println!("WARNING: We don't know how to spawn [{}]!", spawn.1);
}
