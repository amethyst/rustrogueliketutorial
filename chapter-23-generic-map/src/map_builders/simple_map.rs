use super::{MapBuilder, Map, Rect, apply_room_to_map,
    apply_horizontal_tunnel, apply_vertical_tunnel, TileType,
    Position, spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct SimpleMapBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    rooms: Vec<Rect>
}

impl MapBuilder for SimpleMapBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }
}

impl SimpleMapBuilder {
    pub fn new(new_depth : i32) -> SimpleMapBuilder {
        SimpleMapBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            rooms: Vec::new()
        }
    }

    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(&mut self.map, &new_room);

                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);
            }
        }

        let stairs_position = self.rooms[self.rooms.len()-1].center();
        let stairs_idx = self.map.xy_idx(stairs_position.0, stairs_position.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        let start_pos = self.rooms[0].center();
        self.starting_position = Position{ x: start_pos.0, y: start_pos.1 };
    }
}
