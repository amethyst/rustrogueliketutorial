use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order : i32
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount : Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Consumable {}

#[derive(Component, Debug)]
pub struct Ranged {
    pub range : i32
}

#[derive(Component, Debug)]
pub struct InflictsDamage {
    pub damage : i32
}

#[derive(Component, Debug)]
pub struct AreaOfEffect {
    pub radius : i32
}

#[derive(Component, Debug)]
pub struct Confusion {
    pub turns : i32
}

#[derive(Component, Debug)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToUseItem {
    pub item : Entity,
    pub target : Option<rltk::Point>
}

#[derive(Component, Debug, Clone)]
pub struct WantsToDropItem {
    pub item : Entity
}
