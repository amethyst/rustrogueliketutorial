use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub wearable : Option<Wearable>,
    pub initiative_penalty : Option<f32>,
    pub weight_lbs : Option<f32>,
    pub base_value : Option<f32>,
    pub vendor_category : Option<String>,
    pub magic : Option<MagicItem>,
    pub attributes : Option<ItemAttributeBonus>,
    pub template_magic : Option<ItemMagicTemplate>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Renderable {
    pub glyph: String,
    pub fg : String,
    pub bg : String,
    pub order: i32,
    pub x_size : Option<i32>,
    pub y_size : Option<i32>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Consumable {
    pub effects : HashMap<String, String>,
    pub charges : Option<i32>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Weapon {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32,
    pub proc_chance : Option<f32>,
    pub proc_target : Option<String>,
    pub proc_effects : Option<HashMap<String, String>>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot : String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
    pub cursed: Option<bool>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemAttributeBonus {
    pub might : Option<i32>,
    pub fitness : Option<i32>,
    pub quickness : Option<i32>,
    pub intelligence : Option<i32>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemMagicTemplate {
    pub unidentified_name: String,
    pub bonus_min: i32,
    pub bonus_max: i32,
    pub include_cursed: bool
}
