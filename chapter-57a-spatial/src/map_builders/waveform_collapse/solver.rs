use super::{MapChunk, Map};
use std::collections::HashSet;

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size : i32,
    chunks : Vec<Option<usize>>,
    chunks_x : usize,
    chunks_y : usize,
    remaining : Vec<(usize, i32)>, // (index, # neighbors)
    pub possible: bool
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map : &Map) -> Solver {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining : Vec<(usize, i32)> = Vec::new();
        for i in 0..(chunks_x*chunks_y) {
            remaining.push((i, 0));
        }

        Solver {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining,
            possible: true
        }
    }

    fn chunk_idx(&self, x:usize, y:usize) -> usize {
        ((y * self.chunks_x) + x) as usize
    }

    fn count_neighbors(&self, chunk_x:usize, chunk_y:usize) -> i32 {
        let mut neighbors = 0;

        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x-1, chunk_y);
            match self.chunks[left_idx] {
                None => {}
                Some(_) => {
                    neighbors += 1;
                }
            }
        }

        if chunk_x < self.chunks_x-1 {
            let right_idx = self.chunk_idx(chunk_x+1, chunk_y);
            match self.chunks[right_idx] {
                None => {}
                Some(_) => {
                    neighbors += 1;
                }
            }
        }

        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y-1);
            match self.chunks[up_idx] {
                None => {}
                Some(_) => {
                    neighbors += 1;
                }
            }
        }

        if chunk_y < self.chunks_y-1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y+1);
            match self.chunks[down_idx] {
                None => {}
                Some(_) => {
                    neighbors += 1;
                }
            }
        }
        neighbors
    }

    pub fn iteration(&mut self, map: &mut Map, rng : &mut super::RandomNumberGenerator) -> bool {
        if self.remaining.is_empty() { return true; }

        // Populate the neighbor count of the remaining list
        let mut remain_copy = self.remaining.clone();
        let mut neighbors_exist = false;
        for r in remain_copy.iter_mut() {
            let idx = r.0;
            let chunk_x = idx % self.chunks_x;
            let chunk_y = idx / self.chunks_x;
            let neighbor_count = self.count_neighbors(chunk_x, chunk_y);
            if neighbor_count > 0 { neighbors_exist = true; }
            *r = (r.0, neighbor_count);
        }
        remain_copy.sort_by(|a,b| b.1.cmp(&a.1));
        self.remaining = remain_copy;

        // Pick a random chunk we haven't dealt with yet and get its index, remove from remaining list
        let remaining_index = if !neighbors_exist {
            (rng.roll_dice(1, self.remaining.len() as i32)-1) as usize
        } else {
            0usize
        };
        let chunk_index = self.remaining[remaining_index].0;
        self.remaining.remove(remaining_index);

        let chunk_x = chunk_index % self.chunks_x;
        let chunk_y = chunk_index / self.chunks_x;

        let mut neighbors = 0;
        let mut options : Vec<Vec<usize>> = Vec::new();

        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x-1, chunk_y);
            match self.chunks[left_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[3].clone());
                }
            }
        }

        if chunk_x < self.chunks_x-1 {
            let right_idx = self.chunk_idx(chunk_x+1, chunk_y);
            match self.chunks[right_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[2].clone());
                }
            }
        }

        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y-1);
            match self.chunks[up_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[1].clone());
                }
            }
        }

        if chunk_y < self.chunks_y-1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y+1);
            match self.chunks[down_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[0].clone());
                }
            }
        }

        if neighbors == 0 {
            // There is nothing nearby, so we can have anything!
            let new_chunk_idx = (rng.roll_dice(1, self.constraints.len() as i32)-1) as usize;
            self.chunks[chunk_index] = Some(new_chunk_idx);
            let left_x = chunk_x as i32 * self.chunk_size as i32;
            let right_x = (chunk_x as i32+1) * self.chunk_size as i32;
            let top_y = chunk_y as i32 * self.chunk_size as i32;
            let bottom_y = (chunk_y as i32+1) * self.chunk_size as i32;


            let mut i : usize = 0;
            for y in top_y .. bottom_y {
                for x in left_x .. right_x {
                    let mapidx = map.xy_idx(x, y);
                    let tile = self.constraints[new_chunk_idx].pattern[i];
                    map.tiles[mapidx] = tile;
                    i += 1;
                }
            }
        }
        else {
            // There are neighbors, so we try to be compatible with them
            let mut options_to_check : HashSet<usize> = HashSet::new();
            for o in options.iter() {
                for i in o.iter() {
                    options_to_check.insert(*i);
                }
            }

            let mut possible_options : Vec<usize> = Vec::new();
            for new_chunk_idx in options_to_check.iter() {
                let mut possible = true;
                for o in options.iter() {
                    if !o.contains(new_chunk_idx) { possible = false; }
                }
                if possible {
                    possible_options.push(*new_chunk_idx);
                }
            }

            if possible_options.is_empty() {
                rltk::console::log("Oh no! It's not possible!");
                self.possible = false;
                return true;
            } else {
                let new_chunk_idx = if possible_options.len() == 1 { 0 }
                    else { rng.roll_dice(1, possible_options.len() as i32)-1 };

                self.chunks[chunk_index] = Some(possible_options[new_chunk_idx as usize]);
                let left_x = chunk_x as i32 * self.chunk_size as i32;
                let right_x = (chunk_x as i32+1) * self.chunk_size as i32;
                let top_y = chunk_y as i32 * self.chunk_size as i32;
                let bottom_y = (chunk_y as i32+1) * self.chunk_size as i32;


                let mut i : usize = 0;
                for y in top_y .. bottom_y {
                    for x in left_x .. right_x {
                        let mapidx = map.xy_idx(x, y);
                        let tile = self.constraints[possible_options[new_chunk_idx as usize]].pattern[i];
                        map.tiles[mapidx] = tile;
                        i += 1;
                    }
                }
            }
        }

        false
    }
}
