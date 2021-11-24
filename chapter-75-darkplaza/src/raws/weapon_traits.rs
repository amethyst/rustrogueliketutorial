use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct WeaponTrait {
    pub name : String,
    pub effects : HashMap<String, String>
}
