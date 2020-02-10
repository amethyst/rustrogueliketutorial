use super::{MetaMapBuilder, BuilderMap, TileType };
use rltk::RandomNumberGenerator;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    #[allow(dead_code)]
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement{ })
    }

    fn door_possible(&self, build_data : &mut BuilderMap, idx : usize) -> bool {
        let mut blocked = false;
        for spawn in build_data.spawn_list.iter() {
            if spawn.0 == idx { blocked = true; }
        }
        if blocked { return false; }

        let x = (idx % build_data.map.width as usize) as i32;
        let y = (idx / build_data.map.width as usize) as i32;

        // Check for east-west door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[idx-1] == TileType::Floor) &&
            (x < build_data.map.width-2 && build_data.map.tiles[idx+1] == TileType::Floor) &&
            (y > 1 && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall) &&
            (y < build_data.map.height-2 && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall)
        {
            return true;
        }

        // Check for north-south door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[idx-1] == TileType::Wall) &&
            (x < build_data.map.width-2 && build_data.map.tiles[idx+1] == TileType::Wall) &&
            (y > 1 && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Floor) &&
            (y < build_data.map.height-2 && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Floor)
        {
            return true;
        }

        false
    }

    fn doors(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // To avoid nested borrowing
            for hall in halls.iter() {
                if hall.len() > 2 { // We aren't interested in tiny corridors
                    if self.door_possible(build_data, hall[0]) {
                        build_data.spawn_list.push((hall[0], "Door".to_string()));
                    }
                }
            }
        } else {
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor && self.door_possible(build_data, i) && rng.roll_dice(1,3)==1 {
                    build_data.spawn_list.push((i, "Door".to_string()));
                }
            }
        }
    }
}
