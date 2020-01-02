use super::{Map, TileType};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum Symmetry { None, Horizontal, Vertical, Both }

pub fn apply_horizontal_tunnel(map : &mut Map, x1:i32, x2:i32, y:i32) -> Vec<usize> {
    let mut corridor = Vec::new();
    for x in min(x1,x2) ..= max(x1,x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize && map.tiles[idx as usize] != TileType::Floor {
            map.tiles[idx as usize] = TileType::Floor;
            corridor.push(idx as usize);
        }
    }
    corridor
}

pub fn apply_vertical_tunnel(map : &mut Map, y1:i32, y2:i32, x:i32) -> Vec<usize> {
    let mut corridor = Vec::new();
    for y in min(y1,y2) ..= max(y1,y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize && map.tiles[idx as usize] != TileType::Floor {
            corridor.push(idx);
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
    corridor
}

pub fn draw_corridor(map: &mut Map, x1:i32, y1:i32, x2:i32, y2:i32) -> Vec<usize> {
    let mut corridor = Vec::new();
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
        if map.tiles[idx] != TileType::Floor {
            corridor.push(idx);
            map.tiles[idx] = TileType::Floor;
        }
    }

    corridor
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, x: i32, y:i32) {
    match mode {
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            let center_x = map.width / 2;
            if x == center_x {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x - x);
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
            }
        }
        Symmetry::Vertical => {
            let center_y = map.height / 2;
            if y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_y = i32::abs(center_y - y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
        Symmetry::Both => {
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            if x == center_x && y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x - x);
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
                let dist_y = i32::abs(center_y - y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
    }
}

fn apply_paint(map: &mut Map, brush_size: i32, x: i32, y: i32) {
    match brush_size {
        1 => {
            let digger_idx = map.xy_idx(x, y);
            map.tiles[digger_idx] = TileType::Floor;
        }

        _ => {
            let half_brush_size = brush_size / 2;
            for brush_y in y-half_brush_size .. y+half_brush_size {
                for brush_x in x-half_brush_size .. x+half_brush_size {
                    if brush_x > 1 && brush_x < map.width-1 && brush_y > 1 && brush_y < map.height-1 {
                        let idx = map.xy_idx(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}
