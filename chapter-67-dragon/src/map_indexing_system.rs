use specs::prelude::*;
use super::{Map, Position, BlocksTile, TileSize};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        ReadStorage<'a, TileSize>,
                        Entities<'a>,);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers, sizes, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            if let Some(size) = sizes.get(entity) {
                // Multi-tile
                for y in position.y .. position.y + size.y {
                    for x in position.x .. position.x + size.x {
                        if x > 0 && x < map.width-1 && y > 0 && y < map.height-1 {
                            let idx = map.xy_idx(x, y);
                            if blockers.get(entity).is_some() {
                                map.blocked[idx] = true;
                            }

                            // Push the entity to the appropriate index slot. It's a Copy
                            // type, so we don't need to clone it (we want to avoid moving it out of the ECS!)
                            map.tile_content[idx].push(entity);
                        }
                    }
                }
            } else {
                // Single Tile
                if blockers.get(entity).is_some() {
                    map.blocked[idx] = true;
                }

                // Push the entity to the appropriate index slot. It's a Copy
                // type, so we don't need to clone it (we want to avoid moving it out of the ECS!)
                map.tile_content[idx].push(entity);
            }
        }
    }
}
