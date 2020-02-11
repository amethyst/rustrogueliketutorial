use super::{BuilderChain, XStart, YStart, AreaStartingPosition, RoomSorter, RoomSort,
    CullUnreachable, VoronoiSpawning, BspDungeonBuilder, DistantExit, BspCorridors,
    CorridorSpawner, RoomDrawer, BuilderMap, MetaMapBuilder, DLABuilder, TileType,
    AreaEndingPosition, XEnd, YEnd};
use rltk::RandomNumberGenerator;

pub fn dwarf_fort_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarven Fortress");
    chain.start_with(BspDungeonBuilder::new());
    chain.with(RoomSorter::new(RoomSort::CENTRAL));
    chain.with(RoomDrawer::new());
    chain.with(BspCorridors::new());
    chain.with(CorridorSpawner::new());
    chain.with(DragonsLair::new());

    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::TOP));
    chain.with(CullUnreachable::new());
    chain.with(AreaEndingPosition::new(XEnd::RIGHT, YEnd::BOTTOM));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(DragonSpawner::new());
    chain
}

pub struct DragonsLair {}

impl MetaMapBuilder for DragonsLair {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DragonsLair {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonsLair> {
        Box::new(DragonsLair{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        build_data.map.depth = 7;
        build_data.take_snapshot();

        let mut builder = BuilderChain::new(6, build_data.width, build_data.height, "New Map");
        builder.start_with(DLABuilder::insectoid());
        builder.build_map(rng);

        // Add the history to our history
        for h in builder.build_data.history.iter() {
            build_data.history.push(h.clone());
        }
        build_data.take_snapshot();

        // Merge the maps
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Wall && builder.build_data.map.tiles[idx] == TileType::Floor {
                *tt = TileType::Floor;
            }
        }
        build_data.take_snapshot();
    }
}

pub struct DragonSpawner {}

impl MetaMapBuilder for DragonSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DragonSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonSpawner> {
        Box::new(DragonSpawner{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Find a central location that isn't occupied
        let seed_x = build_data.map.width / 2;
        let seed_y = build_data.map.height / 2;
        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if crate::map::tile_walkable(*tiletype) {
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

        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;
        let dragon_pt = rltk::Point::new(start_x, start_y);

        // Remove all spawns within 25 tiles of the drake
        let w = build_data.map.width as i32;
        build_data.spawn_list.retain(|spawn| {
            let spawn_pt = rltk::Point::new(
                spawn.0 as i32 % w,
                spawn.0 as i32 / w
            );
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(dragon_pt, spawn_pt);
            distance > 25.0
        });

        // Add the dragon
        let dragon_idx = build_data.map.xy_idx(start_x, start_y);
        build_data.spawn_list.push((dragon_idx, "Black Dragon".to_string()));
    }
}
