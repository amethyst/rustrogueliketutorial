rltk::add_wasm_support!();
use rltk::{Rltk, GameState, Console};

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World", "resources");
    let gs = State{ };
    rltk::main_loop(context, gs);
}