use super::*;
use specs::prelude::*;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::RunState;

pub fn item_trigger(creator : Option<Entity>, item: Entity, targets : &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(creator, item, targets, ecs);

    // If it was a consumable, then it gets deleted
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}

fn event_trigger(creator : Option<Entity>, entity: Entity, targets : &Targets, ecs: &mut World) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Providing food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog.entries.insert(0, format!("You eat the {}.", names.get(entity).unwrap().name));
    }

    // Magic mapper
    if ecs.read_storage::<MagicMapper>().get(entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        gamelog.entries.insert(0, "The map is revealed to you!".to_string());
        *runstate = RunState::MagicMapReveal{ row : 0};
    }

    // Town Portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog.entries.insert(0, "You are already in town, so the scroll does nothing.".to_string());
        } else {
            gamelog.entries.insert(0, "You are telported back to town!".to_string());
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
        }
    }

    // Healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(creator, EffectType::Healing{amount: heal.heal_amount}, targets.clone());
    }

    // Damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(creator, EffectType::Damage{ amount: damage.damage }, targets.clone());
    }

    // Confusion
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(entity) {
        add_effect(creator, EffectType::Confusion{ turns : confusion.turns }, targets.clone());
    }
}