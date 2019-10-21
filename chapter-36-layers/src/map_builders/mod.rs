use super::{Map, Rect, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automota;
use cellular_automota::CellularAutomotaBuilder;
mod drunkard;
use drunkard::*;
mod maze;
use maze::*;
mod dla;
use dla::*;
mod common;
use common::*;
mod voronoi;
use voronoi::*;
mod waveform_collapse;
use waveform_collapse::*;
mod prefab_builder;
use prefab_builder::*;
use specs::prelude::*;
mod room_based_spawner;
use room_based_spawner::*;
mod room_based_starting_position;
use room_based_starting_position::*;
mod room_based_stairs;
use room_based_stairs::*;

pub struct BuilderMap {
    pub spawn_list : Vec<(usize, String)>,
    pub map : Map,
    pub starting_position : Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history : Vec<Map>
}

impl BuilderMap {
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

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    new_depth : i32,
    pub build_data : BuilderMap
}

impl BuilderChain {
    pub fn new(new_depth : i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            new_depth,
            build_data : BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history : Vec::new()
            }
        }
    }

    pub fn start_with(&mut self, starter : Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder.")
        };
    }

    pub fn with(&mut self, metabuilder : Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs : &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MapBuilder {
    fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs : &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    /*
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);
    let mut result : Box<dyn MapBuilder>;
    match builder {
        1 => { result = BspDungeonBuilder::new(new_depth); }
        2 => { result = BspInteriorBuilder::new(new_depth); }
        3 => { result = CellularAutomotaBuilder::new(new_depth); }
        4 => { result = DrunkardsWalkBuilder::open_area(new_depth); }
        5 => { result = DrunkardsWalkBuilder::open_halls(new_depth); }
        6 => { result = DrunkardsWalkBuilder::winding_passages(new_depth); }
        7 => { result = DrunkardsWalkBuilder::fat_passages(new_depth); }
        8 => { result = DrunkardsWalkBuilder::fearful_symmetry(new_depth); }
        9 => { result = MazeBuilder::new(new_depth); }
        10 => { result = DLABuilder::walk_inwards(new_depth); }
        11 => { result = DLABuilder::walk_outwards(new_depth); }
        12 => { result = DLABuilder::central_attractor(new_depth); }
        13 => { result = DLABuilder::insectoid(new_depth); }
        14 => { result = VoronoiCellBuilder::pythagoras(new_depth); }
        15 => { result = VoronoiCellBuilder::manhattan(new_depth); }
        16 => { result = PrefabBuilder::constant(new_depth, prefab_builder::prefab_levels::WFC_POPULATED) },
        _ => { result = SimpleMapBuilder::new(new_depth); }
    }

    if rng.roll_dice(1, 3)==1 {
        result = WaveformCollapseBuilder::derived_map(new_depth, result);
    }

    if rng.roll_dice(1, 20)==1 {
        result = PrefabBuilder::sectional(new_depth, prefab_builder::prefab_sections::UNDERGROUND_FORT ,result);
    }

    result = PrefabBuilder::vaults(new_depth, result);

    result*/
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder
}

