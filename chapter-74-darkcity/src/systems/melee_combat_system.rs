use specs::prelude::*;
use crate::{Attributes, Skills, WantsToMelee, Name,
    HungerClock, HungerState, Pools, skill_bonus,
    Skill, Equipped, Weapon, EquipmentSlot, WeaponAttribute, Wearable, NaturalAttackDefense,
    effects::*};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Attributes>,
                        ReadStorage<'a, Skills>,
                        ReadStorage<'a, HungerClock>,
                        ReadStorage<'a, Pools>,
                        ReadStorage<'a, Equipped>,
                        ReadStorage<'a, Weapon>,
                        ReadStorage<'a, Wearable>,
                        ReadStorage<'a, NaturalAttackDefense>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut wants_melee, names, attributes, skills,
            hunger_clock, pools, equipped_items, weapon, wearables, natural) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (&entities, &wants_melee, &names, &attributes, &skills, &pools).join() {
            // Are the attacker and defender alive? Only attack if they are
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

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
                        let attack_index = if nat.attacks.len()==1 { 0 } else { crate::rng::roll_dice(1, nat.attacks.len() as i32) as usize -1 };
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

                let natural_roll = crate::rng::roll_dice(1, 20);
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
                    if wielded.owner == wants_melee.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                }
                let base_armor_class = match natural.get(wants_melee.target) {
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
                    let base_damage = crate::rng::roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
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
                        Targets::Single{ target: wants_melee.target }
                    );
                    crate::gamelog::Logger::new()
                        .npc_name(&name.name)
                        .append("hits")
                        .npc_name(&target_name.name)
                        .append("for")
                        .damage(damage)
                        .append("hp.")
                        .log();

                    // Proc effects
                    if let Some(chance) = &weapon_info.proc_chance {
                        let roll = crate::rng::roll_dice(1, 100);
                        //println!("Roll {}, Chance {}", roll, chance);
                        if roll <= (chance * 100.0) as i32 {
                            //println!("Proc!");
                            let effect_target = if weapon_info.proc_target.as_deref() == Some("Self") {
                                Targets::Single{ target: entity }
                            } else {
                                Targets::Single { target : wants_melee.target }
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
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("considers attacking")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but misjudges the timing!")
                        .log();
                    add_effect(
                        None,
                        EffectType::Particle{ glyph: rltk::to_cp437('‼'), fg: rltk::RGB::named(rltk::BLUE), bg : rltk::RGB::named(rltk::BLACK), lifespan: 200.0 },
                        Targets::Single{ target: wants_melee.target }
                    );
                } else {
                    // Miss
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("attacks")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but can't connect.")
                        .log();
                    add_effect(
                        None,
                        EffectType::Particle{ glyph: rltk::to_cp437('‼'), fg: rltk::RGB::named(rltk::CYAN), bg : rltk::RGB::named(rltk::BLACK), lifespan: 200.0 },
                        Targets::Single{ target: wants_melee.target }
                    );
                }
            }
        }

        wants_melee.clear();
    }
}
