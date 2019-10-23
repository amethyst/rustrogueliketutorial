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
        // Check for east-west door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            build_data.map.tiles[idx-1] == TileType::Floor &&
            build_data.map.tiles[idx+1] == TileType::Floor &&
            build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall &&
            build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall
        {
            return true;
        }

        // Check for north-south door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            build_data.map.tiles[idx-1] == TileType::Wall &&
            build_data.map.tiles[idx+1] == TileType::Wall &&
            build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Floor &&
            build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Floor
        {
            return true;
        }

        false
    }

    fn doors(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // To avoid nested borrowing
            for hall in halls.iter() {
                if hall.len() > 2 { // We aren't interested in tiny corridors
                    if self.door_possible(build_data, hall[0]) {
                        build_data.spawn_list.push((hall[0], "Door".to_string()));
                    }
                }
            }
        }
    }
}