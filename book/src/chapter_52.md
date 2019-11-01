# User Interface

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Along with the town, the first thing your player sees will be your user interface. We've chugged along with a reasonably decent one for a while, but not a *great* one. Ideally, a user interface should make the game approachable to new players - while still offering enough depth for returning ones. It should support keyboard and mouse input (I know many long-time roguelike players hate the mouse; but many newer ones love it), and offer feedback as to what the symbol soup actually *means*. Symbols are a great way to represent the world, but there is a learning curve while your brain comes to associate `g` with a goblin and imagines the scary little blighter.

[Cogmind](https://www.gridsagegames.com/cogmind/) is an inspiration for ASCII (and simple tile) user interfaces. If you haven't played it, I wholeheartedly recommend giving it a look. Also, in a conversation with the creator of [Red Blob Games](https://www.redblobgames.com/), he gave some very insightful commentary on the importance of a good UI: building a UI up-front helps you realize if you can *show* the player what you are making, and can give a really good "feel" for what you are building. So once you are passed initial prototyping, building a user interface can act as a guide for the rest. He's a very wise man, so we'll take his advice!

## Prototyping a User Interface

I like to sketch out UI in [Rex Paint](https://www.gridsagegames.com/rexpaint/). Here's what I first came up with for the tutorial game:

![Screenshot](./c52-s1.jpg)

This isn't a bad start, as far as ASCII user interfaces go. Some pertinent notes:

* We've expanded the terminal to `80x60`, which is a pretty common resolution for these games (Cogmind defaults to it).
*  We've *shrunk* the amount of screen devoted to the map, so as to show you more pertinent information on the screen at once; it's actually `50x48`. 
* The bottom panel is the log, which I colored in and gave some silly fake text just to make it clear what goes there. We'll definitely want to improve our logging experience to help immerse the player. 
* The top-right shows some important information: your health and mana, both numerically and with bars. Below that, we're showing your attributes - and highlighting the ones that are improved in some way (we didn't say how!). 
* The next panel down lists your equipped inventory.
* Below that, we show your *consumables* - complete with a hot-key (shift + number) to activate them.
* Below that, we're showing an example spell - that's not implemented yet, but the idea stands.
* At the bottom of the right panel, we're listing *status effects*. The design document says that you start with a hangover, so we've listed it (even if it isn't written yet). You also start *well fed*, so we'll show that, too.

## Changing the console size

In `main.rs`, the first thing our `main` function does is to bootstrap RLTK. We specify resolution and window title here. So we'll update it to match what we want:

```rust
let mut context = Rltk::init_simple8x8(80, 60, "Rusty Roguelike", "resources");
```

If you `cargo run` now, you'll see a bigger console - and nothing making use of the extra space!

![Screenshot](./c52-s2.jpg)

We'll worry about fixing the main menu later. Let's start making the game look like the prototype sketch.

## Restricting the rendered map

The prototype has the map starting at `1,1` and running to `48,44`. So open up `camera.rs`, and we'll change the boundaries. Instead of using the screen bounds, we'll use our desired viewport:

```rust
pub fn get_screen_bounds(ecs: &World, _ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    //let (x_chars, y_chars) = ctx.get_char_size();
    let (x_chars, y_chars) = (48, 44);

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}
```

Instead of reading the screen size and scaling to it, we're constraining the map to the desired viewport. We've kept the `ctx` parameter even though we aren't using it, so as to not break all the other places that use it.

The map viewport is now nicely constrained:

![Screenshot](./c52-s3.jpg)

## Drawing boxes

We'll go into `gui.rs` (specifically `draw_ui`) and start to place the basic boxes that make up the user interface. We'll also comment out the parts we aren't using yet. The RLTK box function works well, but it *fills in the box*. That's not what we need here, so at the top of `gui.rs` I added a new function:

```rust
pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}
```
This is actually copied from RLTK, but with the fill removed.

Next, we begin work on `draw_ui`:

```rust
pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    use rltk::to_cp437;
    let box_gray : RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);

    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray, black); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray, black); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray, black); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 8, box_gray, black); // Top-right panel
}
```

This gives us a cropped map, and the basic box outline from the prototype graphic:

![Screenshot](./c52-s4.jpg)

Now we add some box connectors in, making it look smoother:

```rust
ctx.set(0, 45, box_gray, black, to_cp437('├'));
ctx.set(49, 8, box_gray, black, to_cp437('├'));
ctx.set(49, 0, box_gray, black, to_cp437('┬'));
ctx.set(49, 45, box_gray, black, to_cp437('┴'));
ctx.set(79, 8, box_gray, black, to_cp437('┤'));
ctx.set(79, 45, box_gray, black, to_cp437('┤'));
```

![Screenshot](./c52-s5.jpg)

## Adding a map name

It looks really nice to show the map name at the top - but maps don't current *have* a name! Let's rectify that. Open up `map/mod.rs` and modify the `Map` structure:

```rust
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,
    pub view_blocked : HashSet<usize>,
    pub name : String,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}
```

We'll also modify the constructor, using the `to_string` pattern we've used elsewhere to let you send anything somewhat string-like:

```rust
/// Generates an empty map, consisting entirely of solid walls
pub fn new<S : ToString>(new_depth : i32, width: i32, height: i32, name: S) -> Map {
    let map_tile_count = (width*height) as usize;
    Map{
        tiles : vec![TileType::Wall; map_tile_count],
        width,
        height,
        revealed_tiles : vec![false; map_tile_count],
        visible_tiles : vec![false; map_tile_count],
        blocked : vec![false; map_tile_count],
        tile_content : vec![Vec::new(); map_tile_count],
        depth: new_depth,
        bloodstains: HashSet::new(),
        view_blocked : HashSet::new(),
        name : name.to_string()
    }
}
```

In `map_builders/waveform_collapse/mod.rs` (lines 39, 62 and 78) update the call to `Map::new` to read `build_data.map = Map::new(build_data.map.depth, build_data.width, build_data.height, &build_data.map.name);`.

In `map_builders/mod.rs` update the `BuilderChain` constructor:

```rust
impl BuilderChain {
    pub fn new<S : ToString>(new_depth : i32, width: i32, height: i32, name : S) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data : BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history : Vec::new(),
                width,
                height
            }
        }
    }
    ...
```

Also, line 268 changes to: `let mut builder = BuilderChain::new(new_depth, width, height, "New Map");`.

`main.rs` line 465 changes to: `gs.ecs.insert(Map::new(1, 64, 64, "New Map"));`.

Finally, in `map_builders/town.rs` change the constructor to name our town. I suggest you pick a name that isn't my company!

```rust
pub fn town_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "The Town of Bracketon");
    chain.start_with(TownBuilder::new());
    chain
}
```

Whew! After all that, let's draw the map name in `gui.rs`:

```rust
// Draw the town name
let map = ecs.fetch::<Map>();
let name_length = map.name.len() + 2;
let x_pos = (22 - (name_length / 2)) as i32;
ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
ctx.set(x_pos + name_length as i32, 0, box_gray, black, to_cp437('├'));
ctx.print_color(x_pos+1, 0, white, black, &map.name);
std::mem::drop(map);
```

So we fetch the map from the ECS `World`, calculate the name's length (plus two for the wrapping characters). Then we figure out the centered position (over the map pane; so 22, half the pane width, *minus* half the length of the name). Then we draw the endcaps and the name. You can `cargo run` to see the improvement:

![Screenshot](./c52-s6.jpg)

## Showing health, mana and attributes

We can modify the existing code for health and mana. The following will work:

```rust
// Draw stats
let player_entity = ecs.fetch::<Entity>();
let pools = ecs.read_storage::<Pools>();
let player_pools = pools.get(*player_entity).unwrap();
let health = format!("Health: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
let mana =   format!("Mana:   {}/{}", player_pools.mana.current, player_pools.mana.max);
ctx.print_color(50, 1, white, black, &health);
ctx.print_color(50, 2, white, black, &mana);
ctx.draw_bar_horizontal(64, 1, 14, player_pools.hit_points.current, player_pools.hit_points.max, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
ctx.draw_bar_horizontal(64, 2, 14, player_pools.mana.current, player_pools.mana.max, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
```

Underneath, we want to display the attributes. Since we're formatting each of them the same, lets introduce a function:

```rust
fn draw_attribute(name : &str, attribute : &Attribute, y : i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray : RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_gray, black, name);
    let color : RGB =
        if attribute.modifiers < 0 { RGB::from_f32(1.0, 0.0, 0.0) }
        else if attribute.modifiers == 0 { RGB::named(rltk::WHITE) }
        else { RGB::from_f32(0.0, 1.0, 0.0) };
    ctx.print_color(67, y, color, black, &format!("{}", attribute.base + attribute.modifiers));
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 { ctx.set(72, y, color, black, rltk::to_cp437('+')); }
}
```

So this attribute prints the name at `50,y` in a lighter grey. Then we determine color based on modifiers; if there are non, we use white. If they are bad (negative) we use red. If they are good (positive) we use green. So that lets us print the value + modifiers (total) at `67,y`. We'll print the bonus at `73,y`. If the bonus is positive, we'll add a `+` symbol.

Now we can call it from our `draw_ui` function:

```rust
// Attributes
let attributes = ecs.read_storage::<Attributes>();
let attr = attributes.get(*player_entity).unwrap();
draw_attribute("Might:", &attr.might, 4, ctx);
draw_attribute("Quickness:", &attr.quickness, 5, ctx);
draw_attribute("Fitness:", &attr.fitness, 6, ctx);
draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);
```

`cargo run` now, and you'll see we are definitely making progress:

![Screenshot](./c52-s7.jpg)

## Adding in equipped items

A nice feature of the prototype UI is that it shows what equipment we have equipped. That's actually quite easy, so let's do it! We iterate `Equipped` items and if they `owner` equals the player, we display their `Name`:

```rust
// Equipped
let mut y = 9;
let equipped = ecs.read_storage::<Equipped>();
let name = ecs.read_storage::<Name>();
for (equipped_by, item_name) in (&equipped, &name).join() {
    if equipped_by.owner == *player_entity {
        ctx.print_color(50, y, white, black, &item_name.name);
        y += 1;
    }
}
```

## Adding consumables

This is also easy:

```rust
// Consumables
y += 1;
let green = RGB::from_f32(0.0, 1.0, 0.0);
let yellow = RGB::named(rltk::YELLOW);
let consumables = ecs.read_storage::<Consumable>();
let backpack = ecs.read_storage::<InBackpack>();
let mut index = 1;
for (carried_by, _consumable, item_name) in (&backpack, &consumables, &name).join() {
    if carried_by.owner == *player_entity && index < 10 {
        ctx.print_color(50, y, yellow, black, &format!("↑{}", index));
        ctx.print_color(53, y, green, black, &item_name.name);
        y += 1;
        index += 1;
    }
}
```

We add 1 to `y`, to force it down a line. Then set `index` to `1` (not zero, because we're aiming for keys across the keyboard!). Then we `join` `backpack`, `consumables` and `name`. For each item, we check that `owner` is the player, and `index` is still less than 10. If it is, we print the name in the format `↑1 Dried Sausage` - where `1` is the `index`. Add one to to the index, increment `y` and we're good to go.

`cargo run` now, and you'll see we are definitely getting closer:

We'll worry about making the consumables hot-keys work momentarily. Lets finish the UI, first!

![Screenshot](./c52-s8.jpg)

## Status effects

We'll gloss over this a little because we currently only have one. This is a direct port of the previous code, so no need for too much explanation:

```rust
// Status
let hunger = ecs.read_storage::<HungerClock>();
let hc = hunger.get(*player_entity).unwrap();
match hc.state {
    HungerState::WellFed => ctx.print_color(50, 44, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Well Fed"),
    HungerState::Normal => {}
    HungerState::Hungry => ctx.print_color(50, 44, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "Hungry"),
    HungerState::Starving => ctx.print_color(50, 44, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "Starving"),
}
```

## Displaying the log

Again, this is pretty much a direct copy:

```rust
// Draw the log
let log = ecs.fetch::<GameLog>();
let mut y = 46;
for s in log.entries.iter() {
    if y < 59 { ctx.print(2, y, &s.to_string()); }
    y += 1;
}
```

Again, making it nicely colored is a future topic.

## Tool-tips

We'll restore the call to draw the tooltips:

```rust
draw_tooltips(ecs, ctx);
```

Inside `draw_tooltips`, we first have to compensate for the map now being offset from the screen. We simply add 1 to `mouse_map_pos`:

```rust
mouse_map_pos.0 += min_x - 1;
mouse_map_pos.1 += min_y - 1;
```

That gets our *old* tooltip system working - but the prototype shows a spiffy new display! So we need to create a way to make these pretty tooltips, and arrange them.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-52-ui)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-52-ui)
---

Copyright (C) 2019, Herbert Wolverson.

---