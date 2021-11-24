use super::{MetaMapBuilder, BuilderMap, TileType};
use crate::map;

#[allow(dead_code)]
pub enum XEnd { LEFT, CENTER, RIGHT }

#[allow(dead_code)]
pub enum YEnd{ TOP, CENTER, BOTTOM }

pub struct AreaEndingPosition {
    x : XEnd,
    y : YEnd
}

impl MetaMapBuilder for AreaEndingPosition {
    fn build_map(&mut self, build_data : &mut BuilderMap)  {
        self.build(build_data);
    }
}

impl AreaEndingPosition {
    #[allow(dead_code)]
    pub fn new(x : XEnd, y : YEnd) -> Box<AreaEndingPosition> {
        Box::new(AreaEndingPosition{
            x, y
        })
    }

    fn build(&mut self, build_data : &mut BuilderMap) {
        let seed_x;
        let seed_y;

        match self.x {
            XEnd::LEFT => seed_x = 1,
            XEnd::CENTER => seed_x = build_data.map.width / 2,
            XEnd::RIGHT => seed_x = build_data.map.width - 2
        }

        match self.y {
            YEnd::TOP => seed_y = 1,
            YEnd::CENTER => seed_y = build_data.map.height / 2,
            YEnd::BOTTOM => seed_y = build_data.map.height - 2
        }

        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if map::tile_walkable(*tiletype) {
                available_floors.push(
                    (
                        idx,
                        rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                            rltk::Point::new(seed_x, seed_y)
                        )
                    )
                );
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        build_data.map.tiles[available_floors[0].0] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
