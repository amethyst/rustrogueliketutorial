extern crate specs;
use specs::prelude::*;
use super::{CombatStats, WantsToMelee, Name, SufferDamage, gamelog::GameLog, 
    MeleePowerBonus, DefenseBonus, Equipped, Position, Renderable, ParticleLifetime};
use rltk::RGB;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, MeleePowerBonus>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, ParticleLifetime>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, combat_stats, mut inflict_damage, 
            melee_power_bonuses, defense_bonuses, equipped, mut positions, mut renderables, mut particles) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in (&entities, &melee_power_bonuses, &equipped).join() {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in (&entities, &defense_bonuses, &equipped).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus));

                    if damage == 0 {
                        log.entries.insert(0, format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.insert(0, format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                        inflict_damage.insert(wants_melee.target, SufferDamage{ amount: damage }).expect("Unable to do damage");                        
                    }

                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        let particle = entities.create();
                        positions.insert(particle, Position{ x:pos.x, y:pos.y }).expect("Unable to insert position");
                        renderables.insert(particle, 
                            Renderable{ glyph: rltk::to_cp437('░'),
                                fg: RGB::named(rltk::CYAN),
                                bg: RGB::named(rltk::BLACK),
                                render_order: 0 }).expect("Unable to insert renderable");
                        particles.insert(particle, ParticleLifetime{ lifetime_ms : 100.0 }).expect("Unable to insert particle lifetime");
                    }
                }
            }
        }

        wants_melee.clear();
    }
}