use super::{MapBuilder, Map,  
    TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    remove_unreachable_areas_returning_most_distant, generate_voronoi_spawn_regions};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

pub struct DLABuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>
}

impl MapBuilder for DLABuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self)  {
        self.build();
    }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl DLABuilder {
    pub fn new(new_depth : i32) -> DLABuilder {
        DLABuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new()
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Carve a starting seed
        self.starting_position = Position{ x: self.map.width/2, y : self.map.height/2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx-1] = TileType::Floor;
        self.map.tiles[start_idx+1] = TileType::Floor;
        self.map.tiles[start_idx-self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx+self.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (total_tiles / 4) as usize;
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        while floor_tile_count  < desired_floor_tiles {
            let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
            let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
            let mut prev_x = digger_x;
            let mut prev_y = digger_y;
            let mut digger_idx = self.map.xy_idx(digger_x, digger_y);
            while self.map.tiles[digger_idx] == TileType::Wall {
                prev_x = digger_x;
                prev_y = digger_y;
                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => { if digger_x > 2 { digger_x -= 1; } }
                    2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
                    3 => { if digger_y > 2 { digger_y -=1; } }
                    _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
                }
                digger_idx = self.map.xy_idx(digger_x, digger_y);
            }
            let prev_idx = self.map.xy_idx(prev_x, prev_y);
            self.map.tiles[prev_idx] = TileType::Floor;

            self.take_snapshot();

            floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
