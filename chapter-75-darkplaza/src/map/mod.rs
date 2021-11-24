use rltk::{ BaseMap, Algorithm2D, Point };
use specs::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
mod tiletype;
pub use tiletype::{TileType, tile_walkable, tile_opaque, tile_cost};
mod themes;
pub use themes::*;
mod dungeon;
pub use dungeon::{MasterDungeonMap, level_transition, freeze_level_entities, thaw_level_entities};
pub mod camera;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,
    pub view_blocked : HashSet<usize>,
    pub name : String,
    pub outdoors : bool,
    pub light : Vec<rltk::RGB>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !crate::spatial::is_blocked(idx)
    }

    pub fn populate_blocked(&mut self) {
        crate::spatial::populate_blocked_from_map(self);
    }

    pub fn populate_blocked_multi(&mut self, width : i32, height : i32) {
        self.populate_blocked();
        for y in 1 .. self.height-1 {
            for x in 1 .. self.width - 1 {
                let idx = self.xy_idx(x, y);
                if !crate::spatial::is_blocked(idx) {
                    for cy in 0..height {
                        for cx in 0..width {
                            let tx = x + cx;
                            let ty = y + cy;
                            if tx < self.width-1 && ty < self.height-1 {
                                let tidx = self.xy_idx(tx, ty);
                                if crate::spatial::is_blocked(tidx) {
                                    crate::spatial::set_blocked(idx, true);
                                }
                            } else {
                                crate::spatial::set_blocked(idx, true);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn clear_content_index(&mut self) {
        crate::spatial::clear();
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new<S : ToString>(new_depth : i32, width: i32, height: i32, name: S) -> Map {
        let map_tile_count = (width*height) as usize;
        crate::spatial::set_size(map_tile_count);
        Map{
            tiles : vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles : vec![false; map_tile_count],
            visible_tiles : vec![false; map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked : HashSet::new(),
            name : name.to_string(),
            outdoors : true,
            light: vec![rltk::RGB::from_f32(0.0, 0.0, 0.0); map_tile_count]
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        if idx > 0 && idx < self.tiles.len() {
            tile_opaque(self.tiles[idx]) || self.view_blocked.contains(&idx)
        } else {
            true
        }
    }

    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        const DIAGONAL_COST : f32 = 1.5;
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let tt = self.tiles[idx as usize];
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, tile_cost(tt))) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, tile_cost(tt))) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, tile_cost(tt))) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, tile_cost(tt))) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, tile_cost(tt) * DIAGONAL_COST)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, tile_cost(tt) * DIAGONAL_COST)); }

        exits
    }

    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

