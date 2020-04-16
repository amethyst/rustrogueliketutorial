use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct LootTable {
    pub name : String,
    pub drops : Vec<LootDrop>
}

#[derive(Deserialize, Debug)]
pub struct LootDrop {
    pub name : String,
    pub weight : i32
}
