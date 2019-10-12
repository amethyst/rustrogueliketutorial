use rltk::rex::XpFile;
use super::{Map, TileType};
use std::collections::HashSet;

pub fn load_test_image(new_depth: i32) -> Map {
    let xp = XpFile::from_resource("../../resources/wfc-demo1.xp").unwrap();
    let mut map : Map = Map::new(new_depth);

    for layer in &xp.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if x < map.width as usize && y < map.height as usize {
                    let idx = map.xy_idx(x as i32, y as i32);
                    match cell.ch {
                        32 => map.tiles[idx] = TileType::Floor, // #
                        35 => map.tiles[idx] = TileType::Wall, // #
                        _ => {}
                    }
                }
            }
        }
    }

    map
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pattern : Vec<TileType>,
    exits: [Vec<bool>; 4],
    has_exits: bool,
    compatible_with: [Vec<usize>; 4]
}

pub fn build_patterns(map : &Map, chunk_size: i32, include_flipping: bool, dedupe: bool) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            // Normal orientation
            let mut pattern : Vec<TileType> = Vec::new();
            let start_x = cx * chunk_size;
            let end_x = (cx+1) * chunk_size;
            let start_y = cy * chunk_size;
            let end_y = (cy+1) * chunk_size;

            for y in start_y .. end_y {
                for x in start_x .. end_x {
                    let idx = map.xy_idx(x, y);
                    pattern.push(map.tiles[idx]);
                }
            }
            patterns.push(pattern);

            if include_flipping {
                // Flip horizontal
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(end_x - x, y);
                        pattern.push(map.tiles[idx]);
                    }
                }

                // Flip vertical
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(x, end_y - y);
                        pattern.push(map.tiles[idx]);
                    }
                }

                // Flip both
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        let idx = map.xy_idx(end_x - x, end_y - y);
                        pattern.push(map.tiles[idx]);
                    }
                }
            }
        }
    }

    // Dedupe
    if dedupe {
        println!("Pre de-duplication, there are {} patterns", patterns.len());
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect(); // dedup
        patterns.extend(set.into_iter());
        println!("There are {} patterns", patterns.len());
    }

    patterns
}

fn tile_idx_in_chunk(chunk_size: i32, x:i32, y:i32) -> usize {
    ((y * chunk_size) + x) as usize
}

pub fn patterns_to_constaints(patterns: Vec<Vec<TileType>>, chunk_size : i32) -> Vec<MapChunk> {
    // Move into the new constraints object
    let mut constraints : Vec<MapChunk> = Vec::new();
    for p in patterns {
        let mut new_chunk = MapChunk{
            pattern: p,
            exits: [ Vec::new(), Vec::new(), Vec::new(), Vec::new() ],
            has_exits : true,
            compatible_with: [ Vec::new(), Vec::new(), Vec::new(), Vec::new() ]
        };
        for exit in new_chunk.exits.iter_mut() {
            for _i in 0..chunk_size {
                exit.push(false);
            }
        }

        let mut n_exits = 0;
        for x in 0..chunk_size {
            // Check for north-bound exits            
            let north_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[north_idx] == TileType::Floor {
                new_chunk.exits[0][x as usize] = true;
                n_exits += 1;
            }

            // Check for south-bound exits
            let south_idx = tile_idx_in_chunk(chunk_size, x, chunk_size-1);
            if new_chunk.pattern[south_idx] == TileType::Floor {
                new_chunk.exits[1][x as usize] = true;
                n_exits += 1;
            }

            // Check for west-bound exits
            let west_idx = tile_idx_in_chunk(chunk_size, 0, x);
            if new_chunk.pattern[west_idx] == TileType::Floor {
                new_chunk.exits[2][x as usize] = true;
                n_exits += 1;
            }

            // Check for east-bound exits
            let east_idx = tile_idx_in_chunk(chunk_size, chunk_size-1, x);
            if new_chunk.pattern[east_idx] == TileType::Floor {
                new_chunk.exits[3][x as usize] = true;
                n_exits += 1;
            }
        }

        if n_exits == 0 {
            new_chunk.has_exits = false;
        }

        constraints.push(new_chunk);
    }

    // Build compatibility matrix
    let ch = constraints.clone();
    for c in constraints.iter_mut() {
        for (j,potential) in ch.iter().enumerate() {
            // If there are no exits at all, it's compatible
            if !c.has_exits || !potential.has_exits {
                for compat in c.compatible_with.iter_mut() {
                    compat.push(j);
                }
            } else {
                // Evaluate compatibilty by direction
                for (direction, exit_list) in c.exits.iter_mut().enumerate() {
                    let opposite;
                    match direction {
                        0 => opposite = 1, // Our North, Their South
                        1 => opposite = 0, // Our South, Their North
                        2 => opposite = 3, // Our West, Their East
                        _ => opposite = 2 // Our East, Their West
                    }

                    let mut it_fits = false;
                    let mut has_any = false;
                    for (slot, can_enter) in exit_list.iter().enumerate() {
                        if *can_enter {
                            has_any = true;
                            if potential.exits[opposite][slot] {
                                it_fits = true;
                            }
                        }
                    }
                    if it_fits {
                        c.compatible_with[direction].push(j);
                    }
                    if !has_any {
                        // There's no exits on this side, we don't care what goes there
                        for compat in c.compatible_with.iter_mut() {
                            compat.push(j);
                        }
                    }
                }
            }
        }
    }

    constraints
}

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
                println!("Oh no! It's not possible!");
                self.possible = false;
                return true;
            } else {
                let new_chunk_idx = if possible_options.len() == 1 { 0 } 
                    else { rng.roll_dice(1, possible_options.len() as i32)-1 };

                self.chunks[chunk_index] = Some(new_chunk_idx as usize);
                let left_x = chunk_x as i32 * self.chunk_size as i32;
                let right_x = (chunk_x as i32+1) * self.chunk_size as i32;
                let top_y = chunk_y as i32 * self.chunk_size as i32;
                let bottom_y = (chunk_y as i32+1) * self.chunk_size as i32;


                let mut i : usize = 0;
                for y in top_y .. bottom_y {
                    for x in left_x .. right_x {
                        let mapidx = map.xy_idx(x, y);
                        let tile = self.constraints[new_chunk_idx as usize].pattern[i];
                        map.tiles[mapidx] = tile;
                        i += 1;
                    }
                }
            }
        }

        false
    }
}