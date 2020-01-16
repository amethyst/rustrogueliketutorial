use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DistantExit {
    #[allow(dead_code)]
    pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x,
            starting_pos.y
        );
        build_data.map.populate_blocked();
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width as usize, build_data.map.height as usize, &map_starts , &build_data.map, 1000.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start != std::f32::MAX {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }

        // Place a staircase
        let stairs_idx = exit_tile.0;
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
