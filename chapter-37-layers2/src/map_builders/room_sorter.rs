use super::{MetaMapBuilder, BuilderMap };
use rltk::RandomNumberGenerator;

#[allow(dead_code)]
pub enum RoomSort { LEFTMOST, RIGHTMOST, TOPMOST, BOTTOMMOST }

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
            RoomSort::BOTTOMMOST => build_data.rooms.as_mut().unwrap().sort_by(|a,b| b.y2.cmp(&a.y2) )
        }
    }
}