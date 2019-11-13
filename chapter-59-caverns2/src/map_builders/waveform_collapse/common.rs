use super::TileType;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pub pattern : Vec<TileType>,
    pub exits: [Vec<bool>; 4],
    pub has_exits: bool,
    pub compatible_with: [Vec<usize>; 4]
}

pub fn tile_idx_in_chunk(chunk_size: i32, x:i32, y:i32) -> usize {
    ((y * chunk_size) + x) as usize
}
