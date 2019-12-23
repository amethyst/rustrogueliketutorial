use super::{MetaMapBuilder, BuilderMap, Rect };
use rltk::RandomNumberGenerator;

#[allow(dead_code)]
pub enum RoomSort { LEFTMOST, RIGHTMOST, TOPMOST, BOTTOMMOST, CENTRAL }

pub struct RoomSorter {
    sort_by : RoomSort
}

impl MetaMapBuilder for RoomSorter {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    #[allow(dead_code)]
    pub fn new(sort_by : RoomSort) -> Box<RoomSorter> {
        Box::new(RoomSorter{ sort_by })
    }

    fn sorter(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        match self.sort_by {
            RoomSort::LEFTMOST => build_data.rooms.as_mut().unwrap().sort_by(|a,b| a.x1.cmp(&b.x1) ),
            RoomSort::RIGHTMOST => build_data.rooms.as_mut().unwrap().sort_by(|a,b| b.x2.cmp(&a.x2) ),
            RoomSort::TOPMOST => build_data.rooms.as_mut().unwrap().sort_by(|a,b| a.y1.cmp(&b.y1) ),
            RoomSort::BOTTOMMOST => build_data.rooms.as_mut().unwrap().sort_by(|a,b| b.y2.cmp(&a.y2) ),
            RoomSort::CENTRAL => {
                let map_center = rltk::Point::new( build_data.map.width / 2, build_data.map.height / 2 );
                let center_sort = |a : &Rect, b : &Rect| {
                    let a_center = a.center();
                    let a_center_pt = rltk::Point::new(a_center.0, a_center.1);
                    let b_center = b.center();
                    let b_center_pt = rltk::Point::new(b_center.0, b_center.1);
                    let distance_a = rltk::DistanceAlg::Pythagoras.distance2d(a_center_pt, map_center);
                    let distance_b = rltk::DistanceAlg::Pythagoras.distance2d(b_center_pt, map_center);
                    distance_a.partial_cmp(&distance_b).unwrap()
                };

                build_data.rooms.as_mut().unwrap().sort_by(center_sort);
            }
        }
    }
}
