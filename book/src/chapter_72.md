# Text Layers

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The default 8x8 font can get quite hard to read for large blocks of text, especially when combined with post-processing effects. RLTK's graphical console modes (basically everything except `curses`) supports displaying multiple consoles on the same screen, optionally with different fonts. RLTK ships with a VGA font (8x16), which is *much* easier to read. We'll use that, *but only for the log*.

Initialization with a second layer in a VGA font is easy (see RLTK example 2 for details). Expand the builder code in `main.rs`:

```rust
let mut context = RltkBuilder::simple(80, 60)
    .with_title("Roguelike Tutorial")
    .with_font("vga8x16.png", 8, 16)
    .with_sparse_console(80, 30, "vga8x16.png")
    .build();
```

The main loop's "clear screen" needs to be expanded to clear both layers. In `main.rs` (the `tick` function), we have a bit of code we haven't touched in 70 chapters - clearing the screen at the beginning of a frame. Now we want to clear both consoles:

```rust
ctx.set_active_console(1);
ctx.cls();
ctx.set_active_console(0);
ctx.cls();
```

I ran into some problems with the `TextBlock` component and multiple consoles, so I wrote a replacement. In `src/gamelog/logstore.rs` we remove the `display_log` function and add a replacement:

```rust
pub fn print_log(console: &mut Box<dyn Console>, pos: Point) {
    let mut y = pos.y;
    let mut x = pos.x;
    LOG.lock().unwrap().iter().rev().take(6).for_each(|log| {
        log.iter().for_each(|frag| {
            console.print_color(x, y, frag.color, RGB::named(rltk::BLACK), &frag.text);
            x += frag.text.len() as i32;
            x += 1;
        });
        y += 1;
        x = pos.x;
    });
}
```

And correct the exports in `src/gamelog/mod.rs`:

```rust
pub use logstore::{clear_log, clone_log, restore_log, print_log};
```

Since the new code handles rendering, it's very easy to draw the log file! Change the log render in `gui.rs`:

```rust
// Draw the log
gamelog::print_log(&mut ctx.consoles[1].console, Point::new(1, 23));
```

If you `cargo run` now, you'll see a much easier to read log section:

![c72-s1.jpg](c72-s1.jpg)

## Let's Clean Up the GUI Code

Since we're working on the GUI, now would be a good time to clean it up. It would be nice to add some mouse support, too. We'll start by turning `gui.rs` into a multi-file module. It's huge, so breaking it up is a win in-and-of itself! Make a new folder, `src/gui` and *move* the `gui.rs` file into it. Then rename that file `mod.rs`. The game will work as before.

Then we do some rearranging:

* Make a new file, `gui/item_render.rs`. Add `mod item_render; pub use item_render::*;` to `gui/mod.rs`, and move the functions `get_item_color` and `get_item_display_name` into it.
* RLTK now supports drawing hollow boxes, so we can delete the `draw_hollow_box` function. Replace calls to `draw_hollow_box(ctx, ...)` with `ctx.draw_hollow_box(...)`.
* Make a new file, `gui/hud.rs`. Add `mod hud; pub use hud::*;` to `gui/mod.rs`. Move the following functions into it: `draw_attribute`, `draw_ui`.
* Make a new file, `gui/tooltips.rs`. Add `mod tooltips; pub use tooltips::*;` to `gui/mod.rs`. Move the `Tooltip` struct and implementation into it, along with the function `draw_tooltips`. You'll have to make that function `pub`.
* Make a new file, `gui/inventory_menu.rs`. Add `mod inventory_menu; pub use inventory_menu::*;` to `gui/mod.rs`. Move the inventory menu code into there.
* It's the same again for item dropping. Make `gui/drop_item_menu.rs`, add `mod drop_item_menu; pub use drop_item_menu::*;` to `mod.rs` and move the item dropping menu.
* Rinse and repeat for `gui/remove_item_menu.rs` and the move item code.
* Repeat once again for `gui/remove_curse_menu.rs`.
* Again - this time `gui/identify_menu.rs`, `gui/ranged_target.rs`, `gui/main_menu.rs`, `gui/game_over_menu.rs`, `gui/cheat_menu.rs` and `gui/vendor_menu.rs`.

There's a lot of import cleanup, also. I recommend referring to the [source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-72-textlayers) if you aren't sure what's needed. Once that's all done, the `gui/mod.rs` doesn't contain *any* functionality: just pointers to the individual files.

The game should run as it did before: but your compile times have improved (especially on incremental builds)!

---

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-72-textlayers)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-72-textlayers)
---

Copyright (C) 2019, Herbert Wolverson.

---