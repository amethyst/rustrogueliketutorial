use serde::{Deserialize};
use super::{Renderable};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub blocks_tile : bool,
    pub vision_range : i32,
    pub ai : String,
    pub quips : Option<Vec<String>>,
    pub attributes : MobAttributes,
    pub skills : Option<HashMap<String, i32>>,
    pub level : Option<i32>,
    pub hp : Option<i32>,
    pub mana : Option<i32>,
    pub equipped : Option<Vec<String>>,
    pub natural : Option<MobNatural>
}

#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub might : Option<i32>,
    pub fitness : Option<i32>,
    pub quickness : Option<i32>,
    pub intelligence : Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct MobNatural {
    pub armor_class : Option<i32>,
    pub attacks: Option<Vec<NaturalAttack>>
}

#[derive(Deserialize, Debug)]
pub struct NaturalAttack {
    pub name : String,
    pub hit_bonus : i32,
    pub damage : String
}