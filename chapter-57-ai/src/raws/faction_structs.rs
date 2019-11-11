use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct FactionInfo {
    pub name : String,
    pub responses : HashMap<String, String>
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Reaction {
    Ignore, Attack, Flee
}
