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

pub fn build_patterns(map : &Map, chunk_size: i32) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    println!("Map is {} x {}. Chunk Size: {}. There are {},{} chunks.", map.width, map.height, chunk_size, chunks_x, chunks_y);

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
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
        }
    }

    // Dedupe
    println!("Pre de-duplication, there are {} patterns", patterns.len());
    let set: HashSet<Vec<TileType>> = patterns.drain(..).collect(); // dedup
    patterns.extend(set.into_iter());
    println!("There are {} patterns", patterns.len());

    patterns
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
            let north_idx = x as usize;
            if new_chunk.pattern[north_idx] == TileType::Floor {
                new_chunk.exits[0][x as usize] = true;
                n_exits += 1;
            }

            // Check for south-bound exits
            let south_idx = ((chunk_size * (chunk_size-1)) + x) as usize;            
            if new_chunk.pattern[south_idx] == TileType::Floor {
                new_chunk.exits[1][x as usize] = true;
                n_exits += 1;
            }

            // Check for west-bound exits
            let west_idx = (x * (chunk_size-1)) as usize;
            if new_chunk.pattern[west_idx] == TileType::Floor {
                new_chunk.exits[2][x as usize] = true;
                n_exits += 1;
            }

            // Check for east-bound exits
            let east_idx = ((x * (chunk_size-1))+x) as usize;
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
    for (i, c) in constraints.iter_mut().enumerate() {
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
                        0 => opposite = 1,
                        1 => opposite = 0,
                        2 => opposite = 3,
                        _ => opposite = 2
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

        println!("Chunk {} is compatible with {},{},{},{} others.", 
            i,
            c.compatible_with[0].len(),
            c.compatible_with[1].len(),c.compatible_with[2].len(),
            c.compatible_with[3].len()
        );
    }

    constraints
}

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size : i32,
    chunks : Vec<Option<usize>>,
    chunks_x : usize,
    chunks_y : usize,
    remaining : Vec<usize>
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map : &Map) -> Solver {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining : Vec<usize> = Vec::new();
        for i in 0..(chunks_x*chunks_y) {
            remaining.push(i);
        }

        Solver {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining
        }
    }

    fn chunk_idx(&self, x:usize, y:usize) -> usize {
        ((y * self.chunks_x) + x) as usize
    }

    pub fn iteration(&mut self, map: &mut Map, rng : &mut super::RandomNumberGenerator) -> bool {
        if self.remaining.is_empty() { return true; }

        // Pick a random chunk we haven't dealt with yet and get its index, remove from remaining list
        let remaining_index = (rng.roll_dice(1, self.remaining.len() as i32)-1) as usize;
        let chunk_index = self.remaining[remaining_index];
        self.remaining.remove(remaining_index);

        let chunk_x = chunk_index % self.chunks_x;
        let chunk_y = chunk_index / self.chunks_x;
        println!("Working on chunk: {},{}", chunk_x, chunk_y);

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
            let mut options_to_check : HashSet<usize> = HashSet::new();
            for o in options.iter() {
                for i in o.iter() {
                    options_to_check.insert(*i);
                }
            }
            println!("We have {} neighbors, and are considering {} possible choices", neighbors, options_to_check.len());

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
                return true;
            } else {
                let new_chunk_idx = if possible_options.len() == 1 { 0 } 
                    else { rng.roll_dice(1, possible_options.len() as i32)-1 };

                println!("Placing");

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