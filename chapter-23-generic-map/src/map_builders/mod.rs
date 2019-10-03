use super::{Map, Rect, TileType, Position, spawner};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod common;
use common::*;
use specs::prelude::*;

trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
    fn spawn(map : &Map, ecs : &mut World, new_depth: i32);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 2);
    println!("Builder roll: {}", builder);
    match builder {
        2 => BspDungeonBuilder::build(new_depth),
        _ => SimpleMapBuilder::build(new_depth)
    }    
}

pub fn spawn(map : &Map, ecs : &mut World, new_depth: i32) {
    BspDungeonBuilder::spawn(map, ecs, new_depth);
}