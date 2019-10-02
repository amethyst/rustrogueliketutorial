use super::{MapBuilder, Map, Rect, apply_room_to_map, 
    TileType, Position, spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct BspDungeonBuilder {}

fn add_subrects(rects : &mut Vec<Rect>, rect : Rect) {
    let width = i32::abs(rect.x1 - rect.x2);
    let height = i32::abs(rect.y1 - rect.y2);
    let half_width = i32::max(width / 2, 1);
    let half_height = i32::max(height / 2, 1);

    rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
	rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
	rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
	rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
}

fn get_random_rect(rects : &mut Vec<Rect>, rng : &mut RandomNumberGenerator) -> Rect {
    if rects.len() == 1 { return rects[0]; }
    let idx = (rng.roll_dice(1, rects.len() as i32)-1) as usize;
    rects[idx]
}

fn get_random_sub_rect(rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
    let mut result = rect;
    let rect_width = i32::abs(rect.x1 - rect.x2);
    let rect_height = i32::abs(rect.y1 - rect.y2);

    let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
    let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;

    result.x1 += rng.roll_dice(1, 6)-1;
    result.y1 += rng.roll_dice(1, 6)-1;
    result.x2 = result.x1 + w;
    result.y2 = result.y1 + h;

    result
}

fn is_possible(map : &mut Map, mut rect : Rect) -> bool {
    if rect.x1 > 0 {
        rect.x1 -= 1;
        rect.x2 += 1;
    }
    if rect.y1 > 0 {
        rect.y1 -= 1;
        rect.y1 += 1;
    }

    for y in rect.y1 ..= rect.y2 {
        for x in rect.x1 ..= rect.x2 {
            if x > map.width-1 { return false; }
            if y > map.height-1 { return false; }
            if x < 0 { return false; }
            if y < 0 { return false; }
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] != TileType::Wall { return false; }
        }
    }

    true
}

fn draw_corridor(map : &mut Map, x1:i32, y1:i32, x2:i32, y2:i32) {
    let mut x = x1;
    let mut y = y1;

    while x != x2 || y != y2 {
        if x < x2 {
            x += 1;
        } else if x > x2 {
            x -= 1;
        } else if y < y2 {
            y += 1;
        } else if y > y2 {
            y -= 1;
        }

        let idx = map.xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    }
}

impl MapBuilder for BspDungeonBuilder {
    fn build(new_depth: i32) -> (Map, Position) {
        let mut map = Map::new(new_depth);
        let mut rng = RandomNumberGenerator::new();

        let mut rects : Vec<Rect> = Vec::new(); // Vector to hold our rectangles as we divide
        rects.push( Rect::new(2, 2, map.width-5, map.height-5) ); // Start with a single map-sized rectangle
        let first_room = rects[0];
        add_subrects(&mut rects, first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = get_random_rect(&mut rects, &mut rng);
            let candidate = get_random_sub_rect(rect, &mut rng);

            if is_possible(&mut map, candidate) {
                apply_room_to_map(&mut map, &candidate);
                map.rooms.push(candidate);
                add_subrects(&mut rects, rect);
            }

            n_rooms += 1;
        }

        // Now we sort the rooms
        map.rooms.sort_by(|a,b| a.x1.cmp(&b.x1) );

        // Now we want corridors
        for i in 0..map.rooms.len()-1 {
            let room = map.rooms[i];
            let next_room = map.rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            draw_corridor(&mut map, start_x, start_y, end_x, end_y);
        }

        let player_start = map.rooms[0].center();
        let stairs = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = map.xy_idx(stairs.0, stairs.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
        (map, Position{ x : player_start.0, y : player_start.1 })
    }

    fn spawn(map : &Map, ecs : &mut World, new_depth: i32) {
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, new_depth);
        }
    }
}