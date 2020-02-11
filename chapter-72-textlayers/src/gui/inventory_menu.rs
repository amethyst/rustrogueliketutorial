use rltk::prelude::*;
use specs::prelude::*;
use crate::{State, InBackpack };
use super::{get_item_display_name, item_result_menu};

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn show_inventory(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let mut draw_batch = DrawBatch::new();

    let mut items : Vec<(Entity, String)> = Vec::new();
    (&entities, &backpack).join()
        .filter(|item| item.1.owner == *player_entity )
        .for_each(|item| {
            items.push((item.0, get_item_display_name(&gs.ecs, item.0)))
        });

    let result = item_result_menu(
        &mut draw_batch,
        "Inventory",
        items.len(),
        &items,
        ctx.key
    );
    draw_batch.submit(6000);
    result
}
