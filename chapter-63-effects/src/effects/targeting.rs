use specs::prelude::*;
use crate::components::Position;
use crate::map::Map;

pub fn entity_position(ecs: &World, target: Entity) -> Option<i32> {
    if let Some(pos) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }
    None
}

pub fn aoe_tiles(map: &Map, target: rltk::Point, radius: i32) -> Vec<i32> {
    let mut blast_tiles = rltk::field_of_view(target, radius, &*map);
    blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
    let mut result = Vec::new();
    for t in blast_tiles.iter() {
        result.push(map.xy_idx(t.x, t.y) as i32);
    }
    result
}