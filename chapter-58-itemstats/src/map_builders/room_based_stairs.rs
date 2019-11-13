use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let stairs_position = rooms[rooms.len()-1].center();
            let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room Based Stairs only works after rooms have been created");
        }
    }
}
