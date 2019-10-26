use serde::{Deserialize};
use super::{Renderable};

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub blocks_tile : bool,
    pub stats : MobStats,
    pub vision_range : i32
}

#[derive(Deserialize, Debug)]
pub struct MobStats {
    pub max_hp : i32,
    pub hp : i32,
    pub power : i32,
    pub defense : i32
}