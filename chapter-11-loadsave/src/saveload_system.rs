use specs::prelude::*;
use specs::saveload::{SimpleMarker, SerializeComponents};
use specs::error::NoError;
use super::components::*;
use std::fs::File;
use std::io::Write;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs : &World) {
    let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

    let writer = File::create("./savegame.json").unwrap();
    let mut serializer = serde_json::Serializer::new(writer);
    serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster, 
        Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
        AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
        WantsToDropItem
    );
}