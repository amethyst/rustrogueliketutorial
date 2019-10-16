use super::{MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER,
    remove_unreachable_areas_returning_most_distant};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
mod prefab_levels;
mod prefab_sections;
mod prefab_rooms;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum PrefabMode { 
    RexLevel{ template : &'static str },
    Constant{ level : prefab_levels::PrefabLevel },
    Sectional{ section : prefab_sections::PrefabSection },
    RoomVaults
}

#[allow(dead_code)]
pub struct PrefabBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    previous_builder : Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self)  {
        self.build();
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }

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

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth : i32, previous_builder : Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode : PrefabMode::RoomVaults,
            previous_builder,
            spawn_list : Vec::new()
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel{template} => self.load_rex_map(&template),
            PrefabMode::Constant{level} => self.load_ascii_map(&level),
            PrefabMode::Sectional{section} => self.apply_sectional(&section),
            PrefabMode::RoomVaults => self.apply_room_vaults()
        }
        self.take_snapshot();

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut start_idx;
        if self.starting_position.x == 0 {
            self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();
        }
        let mut has_exit = false;
        for t in self.map.tiles.iter() {
            if *t == TileType::DownStairs { has_exit = true; }
        }

        if !has_exit {
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);

            // Find all tiles we can reach from the starting point
            let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }        
    }

    fn char_to_map(&mut self, ch : char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position{ x:x as i32, y:y as i32 };
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Health Potion".to_string()));
            }
            _ => {
                println!("Unknown glyph loading map: {}", (ch as u8) as char);
            }
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template : &str) -> Vec<char> {
        let mut string_vec : Vec<char> = template.chars().filter(|a| *a != '\r' && *a !='\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
        string_vec
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F) 
        where F : FnMut(i32, i32, &(usize, String)) -> bool
    {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();   
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if filter(x, y, e) {
                self.spawn_list.push(
                    (idx, e.1.to_string())
                )
            }
        }        
        self.take_snapshot(); 
    }

    #[allow(dead_code)]
    fn apply_sectional(&mut self, section : &prefab_sections::PrefabSection) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);
        
        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width-1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height-1) - section.height as i32
        }

        // Build the map
        self.apply_previous_iteration(|x,y,e| {
            x < chunk_x || x > (chunk_x + section.width as i32) || y < chunk_y || y > (chunk_y + section.height as i32)
        });       

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx > 0 && tx < self.map.width as usize -1 && ty < self.map.height as usize -1 && ty > 0 {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self) {
        use prefab_rooms::*;
        let mut rng = RandomNumberGenerator::new();

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_x,_y,_e| true);

        // Note that this is a place-holder and will be moved out of this function
        let master_vault_list = vec![TOTALLY_NOT_A_TRAP];

        // Filter the vault list down to ones that are applicable to the current depth
        let possible_vaults : Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| { self.depth >= v.first_depth && self.depth <= v.last_depth })
            .collect();

        if possible_vaults.is_empty() { return; } // Bail out if there's nothing to build

        let vault_index = if possible_vaults.len() == 1 { 0 } else { (rng.roll_dice(1, possible_vaults.len() as i32)-1) as usize };
        let vault = possible_vaults[vault_index];

        // We'll make a list of places in which the vault could fit
        let mut vault_positions : Vec<Position> = Vec::new();

        let mut idx = 0usize;
        loop {
            let x = (idx % self.map.width as usize) as i32;
            let y = (idx / self.map.width as usize) as i32;

            // Check that we won't overflow the map
            if x > 1 
                && (x+vault.width as i32) < self.map.width-2
                && y > 1 
                && (y+vault.height as i32) < self.map.height-2
            {

                let mut possible = true;
                for ty in 0..vault.height as i32 {
                    for tx in 0..vault.width as i32 {

                        let idx = self.map.xy_idx(tx + x, ty + y);
                        if self.map.tiles[idx] != TileType::Floor {
                            possible = false;
                        }
                    }
                }

                if possible {
                    vault_positions.push(Position{ x,y });
                    break;
                }

            }

            idx += 1;
            if idx >= self.map.tiles.len()-1 { break; }
        }

        if !vault_positions.is_empty() {
            let pos_idx = if vault_positions.len()==1 { 0 } else { (rng.roll_dice(1, vault_positions.len() as i32)-1) as usize };
            let pos = &vault_positions[pos_idx];

            let chunk_x = pos.x;
            let chunk_y = pos.y;

            let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
            let mut i = 0;
            for ty in 0..vault.height {
                for tx in 0..vault.width {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                    i += 1;
                }
            }
            self.take_snapshot();
        }
    }
}
