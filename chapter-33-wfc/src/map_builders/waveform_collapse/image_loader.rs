use rltk::rex::XpFile;
use super::{Map, TileType, MapChunk};
use std::collections::HashSet;

/// Loads a RexPaint file, and converts it into our map format
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
