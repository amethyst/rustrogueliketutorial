//#[macro_use]
//mod single_thread;

#[macro_use]
mod multi_thread;

//pub use single_thread::*;
pub use multi_thread::*;

use specs::prelude::World;
use super::*;

pub trait UnifiedDispatcher {
    fn run_now<'a>(&mut self, ecs : *mut World);
}

construct_dispatcher!(
    (MapIndexingSystem, "map_index", &[]),
    (VisibilitySystem, "visibility", &[]),
    (EncumbranceSystem, "encumbrance", &[]),
    (InitiativeSystem, "initiative", &[]),
    (TurnStatusSystem, "turnstatus", &[]),
    (QuipSystem, "quips", &[]),
    (AdjacentAI, "adjacent", &[]),
    (VisibleAI, "visible", &[]),
    (ApproachAI, "approach", &[]),
    (FleeAI, "flee", &[]),
    (ChaseAI, "chase", &[]),
    (DefaultMoveAI, "default_move", &[]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee", &[]),
    (RangedCombatSystem, "ranged", &[]),
    (ItemCollectionSystem, "pickup", &[]),
    (ItemEquipOnUse, "equip", &[]),
    (ItemUseSystem, "use", &[]),
    (SpellUseSystem, "spells", &[]),
    (ItemIdentificationSystem, "itemid", &[]),
    (ItemDropSystem, "drop", &[]),
    (ItemRemoveSystem, "remove", &[]),
    (HungerSystem, "hunger", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}