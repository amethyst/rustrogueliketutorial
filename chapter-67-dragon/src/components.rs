extern crate specs;
use specs::prelude::*;
extern crate specs_derive;
extern crate rltk;
use rltk::{RGB};
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use std::collections::HashMap;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub display_name : String,
    pub mana_cost : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub spells : Vec<KnownSpell>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialAbility {
    pub spell : String,
    pub chance : f32,
    pub range : f32,
    pub min_range : f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SpecialAbilities {
    pub abilities : Vec<SpecialAbility>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SpellTemplate {
    pub mana_cost : i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub color : RGB,
    pub range: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Initiative {
    pub current : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Vendor {
    pub categories : Vec<String>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MyTurn {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Faction {
    pub name : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyMove {
    pub dest_idx : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyTeleport {
    pub dest_x : i32,
    pub dest_y : i32,
    pub dest_depth : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToApproach {
    pub idx : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToFlee {
    pub indices : Vec<i32>
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint{ path : Option<Vec<i32>> }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MoveMode {
    pub mode : Movement
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ObfuscatedName {
    pub name : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct IdentifiedItem {
    pub name : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points : Pool,
    pub mana : Pool,
    pub xp : i32,
    pub level : i32,
    pub total_weight : f32,
    pub total_initiative_penalty : f32,
    pub gold : f32,
    pub god_mode : bool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub base : i32,
    pub modifiers : i32,
    pub bonus : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub might : Attribute,
    pub fitness : Attribute,
    pub quickness : Attribute,
    pub intelligence : Attribute
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill { Melee, Defense, Magic }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills : HashMap<Skill, i32>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Chasing {
    pub target : Entity
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub table : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentChanged {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub initiative_penalty : f32,
    pub weight_lbs : f32,
    pub base_value : f32
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum MagicItemClass { Common, Rare, Legendary }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {
    pub class : MagicItemClass
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct AttributeBonus {
    pub might : Option<i32>,
    pub fitness : Option<i32>,
    pub quickness : Option<i32>,
    pub intelligence : Option<i32>
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct CursedItem {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {
    pub max_charges : i32,
    pub charges : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesRemoveCurse {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesIdentification {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Confusion {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Slow {
    pub initiative_penalty : f32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct DamageOverTime {
    pub damage : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Duration {
    pub turns : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct StatusEffect {
    pub target : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesMana {
    pub mana_amount : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeachesSpell {
    pub spell : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item : Entity,
    pub target : Option<rltk::Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToCastSpell {
    pub spell : Entity,
    pub target : Option<rltk::Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item : Entity
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Shield, Head, Torso, Legs, Feet, Hands }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot : EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute { Might, Quickness }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MeleeWeapon {
    pub attribute : WeaponAttribute,
    pub damage_n_dice : i32,
    pub damage_die_type : i32,
    pub damage_bonus : i32,
    pub hit_bonus : i32,
    pub proc_chance : Option<f32>,
    pub proc_target : Option<String>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Wearable {
    pub armor_class : f32,
    pub slot : EquipmentSlot
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NaturalAttack {
    pub name : String,
    pub damage_n_dice : i32,
    pub damage_die_type : i32,
    pub damage_bonus : i32,
    pub hit_bonus : i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NaturalAttackDefense {
    pub armor_class : Option<i32>,
    pub attacks : Vec<NaturalAttack>
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms : f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph : u8,
    pub color : RGB,
    pub lifetime_ms : f32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph : u8,
    pub color : RGB,
    pub lifetime_ms : f32
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state : HungerState,
    pub duration : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TownPortal {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only : bool
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available : Vec<String>
}

// Serialization helper code. We need to implement ConvertSaveLoad for each type that contains an
// Entity.

pub struct SerializeMe;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : super::map::Map
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map : super::map::MasterDungeonMap
}
