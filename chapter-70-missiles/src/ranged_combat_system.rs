extern crate specs;
use specs::prelude::*;
use super::{Attributes, Skills, WantsToShoot, Name, gamelog::GameLog,
    HungerClock, HungerState, Pools, skill_bonus,
    Skill, Equipped, Weapon, EquipmentSlot, WeaponAttribute, Wearable, NaturalAttackDefense,
    effects::*, Map, Position};
use rltk::{to_cp437, RGB, Point};

pub struct RangedCombatSystem {}

impl<'a> System<'a> for RangedCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToShoot>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Attributes>,
                        ReadStorage<'a, Skills>,
                        ReadStorage<'a, HungerClock>,
                        ReadStorage<'a, Pools>,
                        WriteExpect<'a, rltk::RandomNumberGenerator>,
                        ReadStorage<'a, Equipped>,
                        ReadStorage<'a, Weapon>,
                        ReadStorage<'a, Wearable>,
                        ReadStorage<'a, NaturalAttackDefense>,
                        ReadStorage<'a, Position>,
                        ReadExpect<'a, Map>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_shoot, names, attributes, skills,
            hunger_clock, pools, mut rng, equipped_items, weapon, wearables, natural,
            positions, map) = data;

        for (entity, wants_shoot, name, attacker_attributes, attacker_skills, attacker_pools) in (&entities, &wants_shoot, &names, &attributes, &skills, &pools).join() {
            // Are the attacker and defender alive? Only attack if they are
            let target_pools = pools.get(wants_shoot.target).unwrap();
            let target_attributes = attributes.get(wants_shoot.target).unwrap();
            let target_skills = skills.get(wants_shoot.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_shoot.target).unwrap();

                // Fire projectile effect
                let apos = positions.get(entity).unwrap();
                let dpos = positions.get(wants_shoot.target).unwrap();
                add_effect(
                    None, 
                    EffectType::ParticleProjectile{ 
                        glyph: to_cp437('*'),
                        fg : RGB::named(rltk::CYAN), 
                        bg : RGB::named(rltk::BLACK), 
                        lifespan : 300.0, 
                        speed: 50.0, 
                        path: rltk::line2d(
                            rltk::LineAlg::Bresenham, 
                            Point::new(apos.x, apos.y), 
                            Point::new(dpos.x, dpos.y)
                        )
                     }, 
                    Targets::Tile{tile_idx : map.xy_idx(apos.x, apos.y) as i32}
                );

                // Define the basic unarmed attack - overridden by wielding check below if a weapon is equipped
                let mut weapon_info = Weapon{
                    range: None,
                    attribute : WeaponAttribute::Might,
                    hit_bonus : 0,
                    damage_n_dice : 1,
                    damage_die_type : 4,
                    damage_bonus : 0,
                    proc_chance : None,
                    proc_target : None
                };

                if let Some(nat) = natural.get(entity) {
                    if !nat.attacks.is_empty() {
                        let attack_index = if nat.attacks.len()==1 { 0 } else { rng.roll_dice(1, nat.attacks.len() as i32) as usize -1 };
                        weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                        weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                        weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                        weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                    }
                }

                let mut weapon_entity : Option<Entity> = None;
                for (weaponentity,wielded,melee) in (&entities, &equipped_items, &weapon).join() {
                    if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                        weapon_info = melee.clone();
                        weapon_entity = Some(weaponentity);
                    }
                }

                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = if weapon_info.attribute == WeaponAttribute::Might
                    { attacker_attributes.might.bonus }
                    else { attacker_attributes.quickness.bonus};
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = weapon_info.hit_bonus;
                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(entity) { // Well-Fed grants +1
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }
                let modified_hit_roll = natural_roll + attribute_hit_bonus + skill_hit_bonus
                    + weapon_hit_bonus + status_hit_bonus;
                //println!("Natural roll: {}", natural_roll);
                //println!("Modified hit roll: {}", modified_hit_roll);

                let mut armor_item_bonus_f = 0.0;
                for (wielded,armor) in (&equipped_items, &wearables).join() {
                    if wielded.owner == wants_shoot.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                }
                let base_armor_class = match natural.get(wants_shoot.target) {
                    None => 10,
                    Some(nat) => nat.armor_class.unwrap_or(10)
                };
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = armor_item_bonus_f as i32;
                let armor_class = base_armor_class + armor_quickness_bonus + armor_skill_bonus
                    + armor_item_bonus;

                //println!("Armor class: {}", armor_class);
                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    // Target hit! Until we support weapons, we're going with 1d4
                    let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(0, base_damage + attr_damage_bonus + 
                        skill_damage_bonus + weapon_damage_bonus);

                    /*println!("Damage: {} + {}attr + {}skill + {}weapon = {}",
                        base_damage, attr_damage_bonus, skill_damage_bonus,
                        weapon_damage_bonus, damage
                    );*/
                    add_effect(
                        Some(entity),
                        EffectType::Damage{ amount: damage },
                        Targets::Single{ target: wants_shoot.target }
                    );
                    log.entries.insert(0, format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));

                    // Proc effects
                    if let Some(chance) = &weapon_info.proc_chance {
                        let roll = rng.roll_dice(1, 100);
                        //println!("Roll {}, Chance {}", roll, chance);
                        if roll <= (chance * 100.0) as i32 {
                            //println!("Proc!");
                            let effect_target = if weapon_info.proc_target.unwrap() == "Self" {
                                Targets::Single{ target: entity }
                            } else {
                                Targets::Single { target : wants_shoot.target }
                            };
                            add_effect(
                                Some(entity),
                                EffectType::ItemUse{ item: weapon_entity.unwrap() },
                                effect_target
                            )
                        }
                    }

                } else  if natural_roll == 1 {
                    // Natural 1 miss
                    log.entries.insert(0, format!("{} considers attacking {}, but misjudges the timing.", name.name, target_name.name));
                    add_effect(
                        None,
                        EffectType::Particle{ glyph: rltk::to_cp437('‼'), fg: rltk::RGB::named(rltk::BLUE), bg : rltk::RGB::named(rltk::BLACK), lifespan: 200.0 },
                        Targets::Single{ target: wants_shoot.target }
                    );
                } else {
                    // Miss
                    log.entries.insert(0, format!("{} attacks {}, but can't connect.", name.name, target_name.name));
                    add_effect(
                        None,
                        EffectType::Particle{ glyph: rltk::to_cp437('‼'), fg: rltk::RGB::named(rltk::CYAN), bg : rltk::RGB::named(rltk::BLACK), lifespan: 200.0 },
                        Targets::Single{ target: wants_shoot.target }
                    );
                }
            }
        }

        wants_shoot.clear();
    }
}
