use crate::map_indexing_system::MapIndexingSystem;
use crate::visibility_system::VisibilitySystem;
use specs::prelude::*;

macro_rules! build_dispatcher {
    (
        $dispatch:expr,
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        $(
            $dispatch.systems.push(Box::new($type{}));
        )*
    };
}

pub trait UnifiedDispatcher {
    fn run_now<'a>(&mut self, ecs : *mut World);
}

pub struct SingleThreadedDispatcher<'a> {
    systems : Vec<Box<dyn RunNow<'a>>>
}

impl<'a> UnifiedDispatcher for SingleThreadedDispatcher<'a> {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            for sys in self.systems.iter_mut() {
                sys.run_now(&*ecs);
            }
        }
    }
}

impl SingleThreadedDispatcher<'_> {
    pub fn new() -> Self {
        let mut dispatch = SingleThreadedDispatcher{
            systems : Vec::new()
        };

        build_dispatcher!(dispatch,
            (MapIndexingSystem, "map_index", &[]),
            (VisibilitySystem, "visibility", &[])
        );

        dispatch
    }
}

/*
(MapIndexingSystem, "map_index", &[]),
(VisibilitySystem, "visibility", &[])
*/

/*
fn run_systems(&mut self) {
    let mut mapindex = MapIndexingSystem{};
    mapindex.run_now(&self.ecs);
    let mut vis = VisibilitySystem{};
    vis.run_now(&self.ecs);
    let mut encumbrance = ai::EncumbranceSystem{};
    encumbrance.run_now(&self.ecs);
    let mut initiative = ai::InitiativeSystem{};
    initiative.run_now(&self.ecs);
    let mut turnstatus = ai::TurnStatusSystem{};
    turnstatus.run_now(&self.ecs);
    let mut quipper = ai::QuipSystem{};
    quipper.run_now(&self.ecs);
    let mut adjacent = ai::AdjacentAI{};
    adjacent.run_now(&self.ecs);
    let mut visible = ai::VisibleAI{};
    visible.run_now(&self.ecs);
    let mut approach = ai::ApproachAI{};
    approach.run_now(&self.ecs);
    let mut flee = ai::FleeAI{};
    flee.run_now(&self.ecs);
    let mut chase = ai::ChaseAI{};
    chase.run_now(&self.ecs);
    let mut defaultmove = ai::DefaultMoveAI{};
    defaultmove.run_now(&self.ecs);
    let mut moving = movement_system::MovementSystem{};
    moving.run_now(&self.ecs);
    let mut triggers = trigger_system::TriggerSystem{};
    triggers.run_now(&self.ecs);
    let mut melee = MeleeCombatSystem{};
    melee.run_now(&self.ecs);
    let mut ranged = RangedCombatSystem{};
    ranged.run_now(&self.ecs);
    let mut pickup = ItemCollectionSystem{};
    pickup.run_now(&self.ecs);
    let mut itemequip = inventory_system::ItemEquipOnUse{};
    itemequip.run_now(&self.ecs);
    let mut itemuse = ItemUseSystem{};
    itemuse.run_now(&self.ecs);
    let mut spelluse = SpellUseSystem{};
    spelluse.run_now(&self.ecs);
    let mut item_id = inventory_system::ItemIdentificationSystem{};
    item_id.run_now(&self.ecs);
    let mut drop_items = ItemDropSystem{};
    drop_items.run_now(&self.ecs);
    let mut item_remove = ItemRemoveSystem{};
    item_remove.run_now(&self.ecs);
    let mut hunger = hunger_system::HungerSystem{};
    hunger.run_now(&self.ecs);
    effects::run_effects_queue(&mut self.ecs);
    let mut particles = particle_system::ParticleSpawnSystem{};
    particles.run_now(&self.ecs);
    let mut lighting = lighting_system::LightingSystem{};
    lighting.run_now(&self.ecs);

    self.ecs.maintain();
}
*/