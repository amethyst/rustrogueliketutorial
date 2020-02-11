use rltk::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

pub fn game_over(ctx : &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Your journey has ended!");
    ctx.print_color_centered(17, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "One day, we'll tell you all about how you did.");
    ctx.print_color_centered(18, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "That day, sadly, is not in this chapter..");

    ctx.print_color_centered(19, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), &format!("You lived for {} turns.", crate::gamelog::get_event_count("Turn")));
    ctx.print_color_centered(20, RGB::named(rltk::RED), RGB::named(rltk::BLACK), &format!("You suffered {} points of damage.", crate::gamelog::get_event_count("Damage Taken")));
    ctx.print_color_centered(21, RGB::named(rltk::RED), RGB::named(rltk::BLACK), &format!("You inflicted {} points of damage.", crate::gamelog::get_event_count("Damage Inflicted")));

    ctx.print_color_centered(23, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu
    }
}
    