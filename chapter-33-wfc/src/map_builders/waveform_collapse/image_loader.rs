use rltk::rex::XpFile;
use super::{Map, TileType};
use image::{ImageBuffer, Rgba, DynamicImage, GenericImageView};
use image::Pixel;

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

fn tile_to_rgb(tt : TileType) -> Rgba::<u8> {
    match tt {
        TileType::DownStairs => Rgba::<u8>::from_channels(255, 255, 0, 0),
        TileType::Wall => Rgba::<u8>::from_channels(0, 255, 0, 0),
        TileType::Floor => Rgba::<u8>::from_channels(0, 0, 255, 0)
    }
}

fn rgb_to_tile(color : Rgba::<u8>) -> TileType {
    if color[0] == 255 && color[1] == 0 && color[2] == 0 { return TileType::DownStairs; }
    if color[0] == 0 && color[1] == 255 && color[2] == 0 { return TileType::Wall; }
    TileType::Floor
}

pub fn map_to_image(map : &Map) -> DynamicImage {
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(map.width as u32, map.height as u32);

    for (i,p) in image.pixels_mut().enumerate() {
        let x = i as i32 % map.width;
        let y = i as i32 / map.width;
        let idx = map.xy_idx(x,y);
        *p = tile_to_rgb(map.tiles[idx]);
    }

    DynamicImage::ImageRgba8(image)
}

pub fn image_to_map(image : &DynamicImage, new_depth: i32) -> Map {
    let mut map = Map::new(new_depth);

    for (i, p) in image.pixels().enumerate() {
        let x = i as i32 % map.width;
        let y = i as i32 / map.width;
        let idx = map.xy_idx(x,y);
        map.tiles[idx] = rgb_to_tile(p.2);
    }

    map
}