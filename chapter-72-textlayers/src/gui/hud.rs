use rltk::prelude::*;
use specs::prelude::*;
use crate::{Pools, Map, Name, InBackpack,
    Equipped, HungerClock, HungerState, Attributes, Attribute, Consumable,
    StatusEffect, Duration, KnownSpells, Weapon, gamelog };
use super::{draw_tooltips, get_item_display_name, get_item_color};

fn draw_attribute(name : &str, attribute : &Attribute, y : i32, draw_batch: &mut DrawBatch) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray : RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    draw_batch.print_color(Point::new(50, y), name, ColorPair::new(attr_gray, black));
    let color : RGB =
        if attribute.modifiers < 0 { RGB::from_f32(1.0, 0.0, 0.0) }
        else if attribute.modifiers == 0 { RGB::named(rltk::WHITE) }
        else { RGB::from_f32(0.0, 1.0, 0.0) };
    draw_batch.print_color(Point::new(67, y), &format!("{}", attribute.base + attribute.modifiers), ColorPair::new(color, black));
    draw_batch.print_color(Point::new(73, y), &format!("{}", attribute.bonus), ColorPair::new(color, black));
    if attribute.bonus > 0 { 
        draw_batch.set(Point::new(72, y), ColorPair::new(color, black), to_cp437('+')); 
    }
}

fn box_framework(draw_batch : &mut DrawBatch) {
    let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);

    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 79, 59), ColorPair::new(box_gray, black)); // Overall box
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 49, 45), ColorPair::new(box_gray, black)); // Map box
    draw_batch.draw_hollow_box(Rect::with_size(0, 45, 79, 14), ColorPair::new(box_gray, black)); // Log box
    draw_batch.draw_hollow_box(Rect::with_size(49, 0, 30, 8), ColorPair::new(box_gray, black)); // Top-right panel

    // Draw box connectors
    draw_batch.set(Point::new(0, 45), ColorPair::new(box_gray, black), to_cp437('├'));
    draw_batch.set(Point::new(49, 8), ColorPair::new(box_gray, black), to_cp437('├'));
    draw_batch.set(Point::new(49, 0), ColorPair::new(box_gray, black), to_cp437('┬'));
    draw_batch.set(Point::new(49, 45), ColorPair::new(box_gray, black), to_cp437('┴'));
    draw_batch.set(Point::new(79, 8), ColorPair::new(box_gray, black), to_cp437('┤'));
    draw_batch.set(Point::new(79, 45), ColorPair::new(box_gray, black), to_cp437('┤'));
}

pub fn map_label(ecs: &World, draw_batch: &mut DrawBatch) {
    let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (22 - (name_length / 2)) as i32;
    draw_batch.set(Point::new(x_pos, 0), ColorPair::new(box_gray, black), to_cp437('┤'));
    draw_batch.set(Point::new(x_pos + name_length as i32 - 1, 0), ColorPair::new(box_gray, black), to_cp437('├'));
    draw_batch.print_color(Point::new(x_pos+1, 0), &map.name, ColorPair::new(white, black));
}

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!("Health: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana =   format!("Mana:   {}/{}", player_pools.mana.current, player_pools.mana.max);
    let xp =     format!("Level:  {}", player_pools.level);
    draw_batch.print_color(Point::new(50, 1), &health, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(50, 2), &mana, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(50, 3), &xp, ColorPair::new(white, black));
    draw_batch.bar_horizontal(
        Point::new(64, 1), 
        14, 
        player_pools.hit_points.current, 
        player_pools.hit_points.max, 
        ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK))
    );
    draw_batch.bar_horizontal(
        Point::new(64, 2), 
        14, 
        player_pools.mana.current, 
        player_pools.mana.max, 
        ColorPair::new(RGB::named(rltk::BLUE), RGB::named(rltk::BLACK))
    );
    let xp_level_start = (player_pools.level-1) * 1000;
    draw_batch.bar_horizontal(
        Point::new(64, 3), 
        14, 
        player_pools.xp - xp_level_start, 
        1000, 
        ColorPair::new(RGB::named(rltk::GOLD), RGB::named(rltk::BLACK))
    );
}

fn draw_attributes(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, draw_batch);
    draw_attribute("Quickness:", &attr.quickness, 5, draw_batch);
    draw_attribute("Fitness:", &attr.fitness, 6, draw_batch);
    draw_attribute("Intelligence:", &attr.intelligence, 7, draw_batch);
}

fn initiative_weight(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    draw_batch.print_color(
        Point::new(50, 9),
        &format!("{:.0} lbs ({} lbs max)",
            player_pools.total_weight,
            (attr.might.base + attr.might.modifiers) * 15
        ),
        ColorPair::new(white, black)
    );
    draw_batch.print_color(
        Point::new(50,10), 
        &format!("Initiative Penalty: {:.0}", player_pools.total_initiative_penalty),
        ColorPair::new(white, black)
    );
    draw_batch.print_color(
        Point::new(50,11), 
        &format!("Gold: {:.1}", player_pools.gold),
        ColorPair::new(RGB::named(rltk::GOLD), black)
    );
}

fn equipped(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) -> i32 {
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);
    let mut y = 13;
    let entities = ecs.entities();
    let equipped = ecs.read_storage::<Equipped>();
    let weapon = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = get_item_display_name(ecs, entity);
            draw_batch.print_color(
                Point::new(50, y), 
                &name,
                ColorPair::new(get_item_color(ecs, entity), black));
            y += 1;

            if let Some(weapon) = weapon.get(entity) {
                let mut weapon_info = if weapon.damage_bonus < 0 {
                    format!("┤ {} ({}d{}{})", &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus)
                } else if weapon.damage_bonus == 0 {
                    format!("┤ {} ({}d{})", &name, weapon.damage_n_dice, weapon.damage_die_type)
                } else {
                    format!("┤ {} ({}d{}+{})", &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus)
                };

                if let Some(range) = weapon.range {
                    weapon_info += &format!(" (range: {}, F to fire, V cycle targets)", range);
                }
                weapon_info += " ├";
                draw_batch.print_color(
                    Point::new(3, 45),
                    &weapon_info,
                    ColorPair::new(yellow, black));
            }
        }
    }
    y
}

fn consumables(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, mut y : i32) -> i32 {
    y += 1;
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);
    let entities = ecs.entities();
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity && index < 10 {
            draw_batch.print_color(
                Point::new(50, y), 
                &format!("↑{}", index),
                ColorPair::new(yellow, black)
            );
            draw_batch.print_color(
                Point::new(53, y), 
                &get_item_display_name(ecs, entity),
                ColorPair::new(get_item_color(ecs, entity), black)
            );
            y += 1;
            index += 1;
        }
    }
    y
}

fn spells(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity, mut y : i32) -> i32 {
    y += 1;
    let black = RGB::named(rltk::BLACK);
    let blue = RGB::named(rltk::CYAN);
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    let mut index = 1;
    for spell in known_spells.iter() {
        draw_batch.print_color(
            Point::new(50, y),
            &format!("^{}", index),
            ColorPair::new(blue, black)
        );
        draw_batch.print_color(
            Point::new(53, y),
            &format!("{} ({})", &spell.display_name, spell.mana_cost),
            ColorPair::new(blue, black)
        );
        index += 1;
        y += 1;
    }
    y
}

fn status(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let mut y = 44;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => {
            draw_batch.print_color(
                Point::new(50, y), 
                "Well Fed",
                ColorPair::new(RGB::named(rltk::GREEN), RGB::named(rltk::BLACK))
            );
            y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(
                Point::new(50, y),
                "Hungry",
                ColorPair::new(RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK))
            );
            y -= 1;
        }
        HungerState::Starving => {
            draw_batch.print_color(
                Point::new(50, y),
                "Starving",
                ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK))
            );
            y -= 1;
        }
    }
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target == *player_entity {
            draw_batch.print_color(
                Point::new(50, y),
                &format!("{} ({})", name.name, duration.turns),
                ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK)),
            );
            y -= 1;
        }
    }
}

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = ecs.fetch::<Entity>();

    box_framework(&mut draw_batch);
    map_label(ecs, &mut draw_batch);
    draw_stats(ecs, &mut draw_batch, &player_entity);
    draw_attributes(ecs, &mut draw_batch, &player_entity);
    initiative_weight(ecs, &mut draw_batch, &player_entity);
    let mut y = equipped(ecs, &mut draw_batch, &player_entity);
    y += consumables(ecs, &mut draw_batch, &player_entity, y);
    spells(ecs, &mut draw_batch, &player_entity, y);
    status(ecs, &mut draw_batch, &player_entity);
    gamelog::print_log(&mut ctx.consoles[1].console, Point::new(1, 23));

    draw_tooltips(ecs, ctx);
    draw_batch.submit(5000);
}