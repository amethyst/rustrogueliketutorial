use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::{Map, TileType};
use crate::components::{Position, Viewshed, OtherLevelPosition};
use crate::map_builders::level_builder;
use specs::prelude::*;
use rltk::Point;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps : HashMap<i32, Map>
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap{ maps: HashMap::new() }
    }

    pub fn store_map(&mut self, map : &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth : i32) -> Option<Map> {
        if self.maps.contains_key(&depth) {
            let mut result = self.maps[&depth].clone();
            Some(result)
        } else {
            None
        }
    }
}

fn transition_to_new_map(ecs : &mut World, new_depth: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = level_builder(new_depth, &mut rng, 80, 50);
    builder.build_map(&mut rng);
    if new_depth > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
            builder.build_data.map.tiles[up_idx] = TileType::UpStairs;
        }
    }
    let mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = ecs.write_resource::<Map>();
        *worldmap_resource = builder.build_data.map.clone();
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }

    // Spawn bad guys
    std::mem::drop(rng);
    builder.spawn_entities(ecs);

    // Place the player and update resources
    let (player_x, player_y) = (player_start.x, player_start.y);
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new(player_x, player_y);
    let mut position_components = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let player_pos_comp = position_components.get_mut(*player_entity);
    if let Some(player_pos_comp) = player_pos_comp {
        player_pos_comp.x = player_x;
        player_pos_comp.y = player_y;
    }

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let vs = viewshed_components.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }

    // Store the newly minted map
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&builder.build_data.map);

    mapgen_history
}

fn transition_to_existing_map(ecs: &mut World, new_depth: i32, offset: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    // Find the down stairs and place the player
    let w = map.width;
    let stair_type = if offset < 0 { TileType::DownStairs } else { TileType::UpStairs };
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == stair_type {
            let mut player_position = ecs.write_resource::<Point>();
            *player_position = Point::new(idx as i32 % w, idx as i32 / w);
            let mut position_components = ecs.write_storage::<Position>();
            let player_pos_comp = position_components.get_mut(*player_entity);
            if let Some(player_pos_comp) = player_pos_comp {
                player_pos_comp.x = idx as i32 % w;
                player_pos_comp.y = idx as i32 / w;
            }
        }
    }

    *worldmap_resource = map;

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let vs = viewshed_components.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }
}

pub fn freeze_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    // Find positions and make OtherLevelPosition
    let mut pos_to_delete : Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player_entity {
            other_level_positions.insert(entity, OtherLevelPosition{ x: pos.x, y: pos.y, depth: map_depth }).expect("Insert fail");
            pos_to_delete.push(entity);
        }
    }

    // Remove positions
    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}

pub fn thaw_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    // Find OtherLevelPosition
    let mut pos_to_delete : Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        if entity != *player_entity && pos.depth == map_depth {
            positions.insert(entity, Position{ x: pos.x, y: pos.y }).expect("Insert fail");
            pos_to_delete.push(entity);
        }
    }

    // Remove positions
    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}

pub fn level_transition(ecs : &mut World, new_depth: i32, offset: i32) -> Option<Vec<Map>> {
    // Obtain the master dungeon map
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    // Do we already have a map?
    if dungeon_master.get_map(new_depth).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_depth, offset);
        None
    } else {
        std::mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, new_depth))
    }
}
