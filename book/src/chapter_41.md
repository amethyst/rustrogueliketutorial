# Decoupling map size from terminal size

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

So far, we've firmly tied map size to terminal resolution. You have an 80x50 screen, and use a few lines for the user interface - so everything we've made is 80 tiles wide and 43 tiles high. As you've seen in previous chapters, you can do a *lot* with 3,440 tiles - but sometimes you want more (and sometimes you want less). You may also want a big, open world setting - but we're not going to go there yet! This chapter will start by decoupling the *camera* from the *map*, and then enable map size and screen size to differ. The difficult topic of resizing the user interface will be left for future development.

## Introducing a Camera

A common abstraction in games is to separate *what* you are viewing (the map and entities) from *how* you are viewing it - the camera. The camera typically follows your brave adventurer around the map, showing you the world from *their* point of view. In 3D games, the camera can be pretty complicated; in top-down roguelikes (viewing the map from above), it typically centers the view on the player's `@`.

Predictably enough, we'll start by making a new file: `camera.rs`. To enable it, add `pub mod camera` towards the top of `main.rs` (with the other module access).

We'll start out by making a function, `render_camera`, and doing some calculations we'll need:

```rust
use specs::prelude::*;
use super::{Map,TileType,Position,Renderable,Hidden};
use rltk::{Point, Rltk, Console, RGB};

pub fn render_camera(ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = player_pos.x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = player_pos.y + y_chars as i32;
    ...
```

I've broken this down into steps to make it clear what's going on:

1. We start by retrieving the map from the ECS World.
2. We then retrieve the player's position from the ECS World.
3. We ask RLTK for the current console dimensions, in character space (so with an 8x8 font, `80x50`).
4. We calculate the center of the console.
5. We set `min_x` to the left-most tile, *relative to the player*. So the player's `x` position, minus the center of the console. This will center the `x` axis on the player.
6. We set `max_x` to the be `min_x` plus the console width - again, ensuring that the player is centered.
7. We do the same for `min_y` and `max_y`.

So we've established where the camera is in *world space* - that is, coordinates on the map itself. We've also established that with our *camera view*, that should be the center of the rendered area.

Now we'll render the actual map:

```rust
let map_width = map.width-1;
let map_height = map.height-1;

let mut y = 0;
for ty in min_y .. max_y {
    let mut x = 0;
    for tx in min_x .. max_x {
        if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
            let idx = map.xy_idx(tx, ty);
            if map.revealed_tiles[idx] {
                let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                ctx.set(x, y, fg, bg, glyph);
            }
        }            
        x += 1;
    }
    y += 1;
}
```

This is similar to our old `draw_map` code, but a little more complicated. Lets walk through it:

1. We set `y` to 0; we're using `x` and `y` to represent actual *screen* coordinates.
2. We loop `ty` from `min_y` to `max_y`. We're using `tx` and `ty` for *map* coordinates - or "tile space" coordinates (hence the `t`).
    1. We set `x` to zero, because we're starting a new row on the screen.
    2. We loop from `min_x` to `max_x` in the variable `tx` - so we're covering the visible *tile space* in `tx`.
        1. We do a *clipping* check. We check that `tx` and `ty` are actually inside the *map* boundaries. It's quite likely that the player will visit the edge of the map, and you don't want to crash because they can see tiles that aren't in the map area!
        2. We calculate the `idx` (index) of the `tx/ty` position, telling us where on the map this screen location is.
        3. If it is revealed, we call the mysterious `get_tile_glyph` function for this index (more on that in a moment), and set the results on the screen.
        4. Regardless of clipping, we add 1 to `x` - we're moving to the next column.
    3. We add one to `y`, since we're now moving down the screen.
3. We've rendered a map!

That's actually quite simple - we're rendering what is effectively a window looking into part of the map, rather than the whole map - and centering the window on the player.

Next, we need to render our entities:

```rust
let positions = ecs.read_storage::<Position>();
let renderables = ecs.read_storage::<Renderable>();
let hidden = ecs.read_storage::<Hidden>();
let map = ecs.fetch::<Map>();

let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
for (pos, render, _hidden) in data.iter() {
    let idx = map.xy_idx(pos.x, pos.y);
    if map.visible_tiles[idx] { 
        let entity_screen_x = pos.x - min_x;
        let entity_screen_y = pos.y - min_y;
        if entity_screen_x > 0 && entity_screen_x < map_width && entity_screen_y > 0 && entity_screen_y < map_height {
            ctx.set(entity_screen_x, entity_screen_y, render.fg, render.bg, render.glyph);
        }
    }
}
```

If this looks familiar, it's because it's the *same* as the render code that used to live in `main.rs`. There are two major differences: we subtract `min_x` and `min_y` from the `x` and `y` coordinates, to line the entities up with our camera view. We also perform *clipping* on the coordinates - we won't try and render anything that isn't on the screen.

We previously referred to `get_tile_glyph`, so here it is:

```rust
fn get_tile_glyph(idx: usize, map : &Map) -> (u8, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.0, 0.5, 0.5);
        }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_f32(0., 1.0, 0.);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
    }
    if map.bloodstains.contains(&idx) { bg = RGB::from_f32(0.75, 0., 0.); }
    if !map.visible_tiles[idx] { 
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // Don't show stains out of visual range
    }

    (glyph, fg, bg)
}
```

This is very similar to the code from `draw_map` we wrote ages ago, but instead of drawing to the map it returns a glyph, foreground and background colors. It still handles bloodstains, greying out areas that you can't see, and calls `wall_glyph` for nice walls. We've simply copied `wall_glyph` over from `map.rs`:

```rust
fn wall_glyph(map : &Map, x: i32, y:i32) -> u8 {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
    if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
    if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
    if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 9 } // Pillar because we can't see neighbors
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        _ => { 35 } // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
```

Finally, in `main.rs` find the following code:

```rust
...
RunState::GameOver{..} => {}
_ => {
    draw_map(&self.ecs.fetch::<Map>(), ctx);
    let positions = self.ecs.read_storage::<Position>();
    let renderables = self.ecs.read_storage::<Renderable>();
    let hidden = self.ecs.read_storage::<Hidden>();
    let map = self.ecs.fetch::<Map>();

    let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
    for (pos, render, _hidden) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
    }
    gui::draw_ui(&self.ecs, ctx);                
}
...
```

We can now replace that with a *much* shorter piece of code:

```rust
RunState::GameOver{..} => {}
_ => {
    camera::render_camera(&self.ecs, ctx);
    gui::draw_ui(&self.ecs, ctx);                
}
```

If you `cargo run` the project now, you'll see that we can still play - and the camera is centered on the player:

![Screenshot](./c41-s1.jpg).

### Oops - we didn't move the tooltips or targeting!

If you play for a bit, you'll probably notice that tool-tips aren't working (they are still bound to the map coordinates), and using a targeted effect highlights completely the wrong part of the screen for your aiming. We should fix that!


...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-41-camera)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-41-camera)
---

Copyright (C) 2019, Herbert Wolverson.

---