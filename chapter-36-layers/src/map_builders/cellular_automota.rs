use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct CellularAutomotaBuilder {}

impl InitialMapBuilder for CellularAutomotaBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomotaBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<CellularAutomotaBuilder> {
        Box::new(CellularAutomotaBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 { build_data.map.tiles[idx] = TileType::Floor } 
                else { build_data.map.tiles[idx] = TileType::Wall }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automota rules
        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height-1 {
                for x in 1..build_data.map.width-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    }
                    else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }

        /*
        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);

        // Spawn the entities
        for area in self.noise_areas.iter() {
            spawner::spawn_region(&self.map, rng, area.1, self.depth, &mut self.spawn_list);
        }
        */
    }
}
