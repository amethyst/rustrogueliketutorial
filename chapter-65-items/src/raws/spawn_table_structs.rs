use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct SpawnTableEntry {
    pub name : String,
    pub weight : i32,
    pub min_depth: i32,
    pub max_depth: i32,
    pub add_map_depth_to_weight : Option<bool>
}
