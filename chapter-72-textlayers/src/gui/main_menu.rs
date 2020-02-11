use rltk::prelude::*;
use crate::{State, RunState, rex_assets::RexAssets };

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let mut draw_batch = DrawBatch::new();
    let save_exists = crate::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    draw_batch.draw_double_box(Rect::with_size(24, 18, 31, 10), ColorPair::new(RGB::named(rltk::WHEAT), RGB::named(rltk::BLACK)));

    draw_batch.print_color_centered(20, "Rust Roguelike Tutorial", ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)));
    draw_batch.print_color_centered(21, "by Herbert Wolverson", ColorPair::new(RGB::named(rltk::CYAN), RGB::named(rltk::BLACK)));
    draw_batch.print_color_centered(22, "Use Up/Down Arrows and Enter", ColorPair::new(RGB::named(rltk::GRAY), RGB::named(rltk::BLACK)));

    let mut y = 24;
    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            draw_batch.print_color_centered(y, "Begin New Game", ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)));
        } else {
            draw_batch.print_color_centered(y, "Begin New Game", ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)));
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                draw_batch.print_color_centered(y, "Load Game", ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)));
            } else {
                draw_batch.print_color_centered(y, "Load Game", ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)));
            }
            y += 1;
        }

        if selection == MainMenuSelection::Quit {
            draw_batch.print_color_centered(y, "Quit", ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)));
        } else {
            draw_batch.print_color_centered(y, "Quit", ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)));
        }

        draw_batch.submit(6000);

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}
