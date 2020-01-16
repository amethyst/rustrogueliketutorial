extern crate rltk;
use rltk::{ BaseMap, Algorithm2D, Point };
extern crate specs;
use specs::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
mod tiletype;
pub use tiletype::{TileType, tile_walkable, tile_opaque, tile_cost};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,
    pub view_blocked : HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = !tile_walkable(*tile);
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new(new_depth : i32, width: i32, height: i32) -> Map {
        let map_tile_count = (width*height) as usize;
        Map{
            tiles : vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles : vec![false; map_tile_count],
            visible_tiles : vec![false; map_tile_count],
            blocked : vec![false; map_tile_count],
            tile_content : vec![Vec::new(); map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked : HashSet::new()
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:i32) -> bool {
        let idx_u = idx as usize;
        if idx_u > 0 && idx_u < self.tiles.len() {
            tile_opaque(self.tiles[idx_u]) || self.view_blocked.contains(&idx_u)
        } else {
            true
        }
    }

    fn get_available_exits(&self, idx:i32) -> Vec<(i32, f32)> {
        const DIAGONAL_COST : f32 = 1.5;
        let mut exits : Vec<(i32, f32)> = Vec::new();
        let x = idx % self.width;
        let y = idx / self.width;
        let tt = self.tiles[idx as usize];

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, tile_cost(tt))) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, tile_cost(tt))) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, tile_cost(tt))) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, tile_cost(tt))) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-self.width)-1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-self.width)+1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+self.width)-1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+self.width)+1, tile_cost(tt) * DIAGONAL_COST)); }

        exits
    }

    fn get_pathing_distance(&self, idx1:i32, idx2:i32) -> f32 {
        let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

