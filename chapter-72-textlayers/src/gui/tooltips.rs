use rltk::prelude::*;
use specs::prelude::*;
use crate::{Pools, Map, Name, Hidden, camera, Attributes, StatusEffect, Duration };
use super::get_item_display_name;

struct Tooltip {
    lines : Vec<String>
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines : Vec::new() }
    }

    fn add<S:ToString>(&mut self, line : S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 { self.lines.len() as i32 + 2i32 }

    fn render(&self, draw_batch : &mut DrawBatch, x : i32, y : i32) {
        let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray : RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        draw_batch.draw_box(Rect::with_size(x, y, self.width()-1, self.height()-1), ColorPair::new(white, box_gray));
        for (i,s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            draw_batch.print_color(Point::new(x+1, y+i as i32+1), &s, ColorPair::new(col, black));
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx : &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;
    if mouse_pos.0 < 1 || mouse_pos.0 > 49 || mouse_pos.1 < 1 || mouse_pos.1 > 40 {
        return;
    }
    if mouse_map_pos.0 >= map.width-1 || mouse_map_pos.1 >= map.height-1 || mouse_map_pos.0 < 1 || mouse_map_pos.1 < 1
    {
        return;
    }
    if !map.in_bounds(rltk::Point::new(mouse_map_pos.0, mouse_map_pos.1)) { return; }
    let mouse_idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[mouse_idx] { return; }

    let mut tip_boxes : Vec<Tooltip> = Vec::new();
    crate::spatial::for_each_tile_content(mouse_idx, |entity| {
        if hidden.get(entity).is_some() { return; }
        let mut tip = Tooltip::new();
        tip.add(get_item_display_name(ecs, entity));

        // Comment on attributes
        let attr = attributes.get(entity);
        if let Some(attr) = attr {
            let mut s = "".to_string();
            if attr.might.bonus < 0 { s += "Weak. " };
            if attr.might.bonus > 0 { s += "Strong. " };
            if attr.quickness.bonus < 0 { s += "Clumsy. " };
            if attr.quickness.bonus > 0 { s += "Agile. " };
            if attr.fitness.bonus < 0 { s += "Unheathy. " };
            if attr.fitness.bonus > 0 { s += "Healthy." };
            if attr.intelligence.bonus < 0 { s += "Unintelligent. "};
            if attr.intelligence.bonus > 0 { s += "Smart. "};
            if s.is_empty() {
                s = "Quite Average".to_string();
            }
            tip.add(s);
        }

        // Comment on pools
        let stat = pools.get(entity);
        if let Some(stat) = stat {
            tip.add(format!("Level: {}", stat.level));
        }

        // Status effects
        let statuses = ecs.read_storage::<StatusEffect>();
        let durations = ecs.read_storage::<Duration>();
        let names = ecs.read_storage::<Name>();
        for (status, duration, name) in (&statuses, &durations, &names).join() {
            if status.target == entity {
                tip.add(format!("{} ({})", name.name, duration.turns));
            }
        }

        tip_boxes.push(tip);
    });

    if tip_boxes.is_empty() { return; }

    let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
    let white = RGB::named(rltk::WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 < 40 {
        // Render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    draw_batch.set(Point::new(arrow_x, arrow_y), ColorPair::new(white, box_gray), arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height/2) > 50 {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < 40 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + tt.width())
        };
        tt.render(&mut draw_batch, x, y);
        y += tt.height();
    }

    draw_batch.submit(7000);
}
