# Loading and Saving the Game

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

In the last few chapters, we've focused on getting a playable (if not massively fun) game going. You can run around, slay monsters, and make use of various items. That's a great start! Most games let you stop playing, and come back later to continue. Fortunately, Rust (and associated libraries) makes it relatively easy.

# A Main Menu

If you're going to resume a game, you need somewhere from which to do so! A main menu also gives you the option to abandon your last save, possibly view credits, and generally tell the world that your game is here - and written by you. It's an important thing to have, so we'll put one together.

Being in the menu is a *state* - so we'll add it to the ever-expanding `RunState` enum. We want to include menu state inside it, so the definition winds up looking like this:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, 
    PreRun, 
    PlayerTurn, 
    MonsterTurn, 
    ShowInventory, 
    ShowDropItem, 
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection }
}
```

In `gui.rs`, we add a couple of enum types to handle main menu selections:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection{ selected : MainMenuSelection }, Selected{ selected: MainMenuSelection } }
```

Your GUI is probably now telling you that `main.rs` has errors! It's right - we need to handle the new `RunState` option. We'll need to change things around a bit to ensure that we aren't also rendering the GUI and map when in the menu. So we rearrange `tick`:

```rust
fn tick(&mut self, ctx : &mut Rltk) {
    let mut newrunstate;
    {
        let runstate = self.ecs.fetch::<RunState>();
        newrunstate = *runstate;
    }

    ctx.cls();        

    match newrunstate {
        RunState::MainMenu{..} => {}
        _ => {
            draw_map(&self.ecs, ctx);

            {
                let positions = self.ecs.read_storage::<Position>();
                let renderables = self.ecs.read_storage::<Renderable>();
                let map = self.ecs.fetch::<Map>();

                let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
                for (pos, render) in data.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                }

                gui::draw_ui(&self.ecs, ctx);
            } 
        }
    }
    ...
```

We'll also handle the `MainMenu` state in our large `match` for `RunState`:

```rust
RunState::MainMenu{ .. } => {
    let result = gui::main_menu(self, ctx);
    match result {
        gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
        gui::MainMenuResult::Selected{ selected } => {
            match selected {
                gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                gui::MainMenuSelection::LoadGame => newrunstate = RunState::PreRun,
                gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
            }
        }
    }
}
```

We're basically updating the state with the new menu selection, and if something has been selected we change the game state. For `Quit`, we simply terminate the process. For now, we'll make loading/starting a game do the same thing: go into the `PreRun` state to setup the game.

The last thing to do is to write the menu itself. In `menu.rs`:

```rust
pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rust Roguelike Tutorial");
    
    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
        } else {
            ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
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
```

That's a bit of a mouthful, but it displays menu options and lets you select them with the up/down keys and enter. It's very careful to not modify state itself, to keep things clear.

# Including Serde

`Serde` is pretty much the gold-standard for serialization in Rust. It makes a lot of things easier! So the first step is to include it. In your project's `Cargo.toml` file, we'll expand the `dependencies` section to include it:

```toml
[dependencies]
rltk = { git = "https://github.com/thebracket/rltk_rs", features = ["serialization"] }
specs = { version = "0.15.0", features = ["serde"] }
specs-derive = "0.4.0"
serde= { version = "1.0.93", features = ["derive"] }
serde_json = "1.0.39"
```

It may be worth calling `cargo run` now - it will take a while, downloading the new dependencies (and all of their dependencies) and building them for you. It should keep them around so you don't have to wait this long every time you build.

# Adding a "SaveGame" state

We'll extend `RunState` once more to support game saving:

```rust
#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, 
    PreRun, 
    PlayerTurn, 
    MonsterTurn, 
    ShowInventory, 
    ShowDropItem, 
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame
}
```

In `tick`, we'll add dummy code for now:

```rust
RunState::SaveGame => {
    newrunstate = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::LoadGame };
}
```

In `player.rs`, we'll add another keyboard handler - escape:

```rust
// Save and Quit
VirtualKeyCode::Escape => return RunState::SaveGame,
```

If you `cargo run` now, you can start a game and press escape to quit to the menu.

# Getting started with saving the game

Now that the scaffolding is in place, it's time to actually save something! Lets start simple, to get a feel for Serde. In the `tick` function, we extend the save system to just dump a JSON representation of the map to the console:

```rust
RunState::SaveGame => {
    let data = serde_json::to_string(&*self.ecs.fetch::<Map>()).unwrap();
    println!("{}", data);

    newrunstate = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::LoadGame };
}
```

We'll also need to add an `extern crate serde;` to the top of `main.rs`.

This won't compile, because we need to tell `Map` to serialize itself! Fortunately, `serde` provides some helpers to make this easy. At the top of `map.rs`, we add `use serde::{Serialize, Deserialize};`. We then decorate the map to derive serialization and de-serialization code:

```rust
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}
```

Note that we've decorated `tile_content` with directives to not serialize/de-serialize it. This prevents us from needing to store the entities, and since this data is rebuilt every frame - it doesn't matter. The game still won't compile; we need to add similar decorators to `TileType` and `Rect`:

```rust
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, Floor
}
```

```rust
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x1 : i32,
    pub x2 : i32,
    pub y1 : i32,
    pub y2 : i32
}
```

Lastly, we should extend the game saving code to dump the map to the console:

```rust
let data = serde_json::to_string(&*self.ecs.fetch::<Map>()).unwrap();
println!("{}", data);
```

If you `cargo run` the project now, when you hit escape it will dump a huge blob of JSON data to the console. That's the game map!

# Saving entity state

Now that we've seen how useful `serde` is, we should start to use it for the game itself. This is harder than one might expect, because of how `specs` handles `Entity` structures: their ID # is purely synthetic, with no guaranty that you'll get the same one next time! Also, you may not want to save *everything* - so `specs` introduces a concept of *markers* to help with this. It winds up being a bit more of a mouthful than it really needs to be, but gives a pretty powerful serialization system.

## Introducing Markers

First of all, in `main.rs` we'll tell Rust that we'd like to make use of the marker functionality:

```rust
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
```

In `components.rs`, we'll add a marker type:

```rust
pub struct SerializeMe;
```

Back in `main.rs`, we'll add `SerializeMe` to the list of things that we *register*:

```rust
gs.ecs.register::<SimpleMarker<SerializeMe>>();
```

We'll also add an entry to the ECS resources, which gets used to determine the next identity:

```rust
gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
```

Finally, in `spawners.rs` we tell each entity builder to include the marker. Here's the complete entry for the `Player`:

```rust
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}
```

The new line (`.marked::<SimpleMarker<SerializeMe>>()`) needs to be repeated for all of our spawners in this file. It's worth looking at the source for this chapter; to avoid making a *huge* chapter full of source code, I've omitted the repeated details.

## Serializing components that don't contain an Entity

It's pretty easy to serialize a type that doesn't have an Entity in it: mark it with `#[derive(Component, Serialize, Deserialize, Clone)]`. So we go through all the simple component types in `components.rs`; for example, here's `Position`:

```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

## Serializing components that point to entities

Here is where it gets a little messy. There are no provided `derive` functions for handling serialization of `Entity`, so we have to do it the hard way. The good news is that we're not doing it very often. Here's a helper for `InBackpack`:

```rust
// InBackpack wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct InBackpackData<M>(M);

impl<M: Marker + Serialize> ConvertSaveload<M> for InBackpack
where
    for<'de> M: Deserialize<'de>,
{
    type Data = InBackpackData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.owner).unwrap();
        Ok(InBackpackData(marker))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let entity = ids(data.0).unwrap();
        Ok(InBackpack{owner: entity})
    }
}
```

So we start off by making a "data" class for `InBackpack`, which simply stores the entity at which it points. Then we implement `convert_info` and `convert_from` to satisfy Specs' `ConvertSaveLoad` trait. In `convert_into`, we use the `ids` map to get a saveable ID number for the item, and return an `InBackpackData` using this marker. `convert_from` does the reverse: we get the ID, look up the ID, and return an `InBackpack` method.

So that's not *too* bad. If you look at the source, we've done this for all of the types that store `Entity` data - some of which have other data, or multiple `Entity` types.

## Actually saving something

The code for loading and saving gets large, so we've moved it into `saveload_system.rs`. Then include a `mod saveload_system;` in `main.rs`, and replace the `SaveGame` state with:

```rust
RunState::SaveGame => {
    saveload_system::save_game(&mut self.ecs);
    newrunstate = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::LoadGame };
}
```

So... onto implementing `save_game`. Serde and Specs work decently together, but the bridge is still pretty roughly defined. I kept running into problems like it failing to compile if I had more than 16 component types! To get around this, I build a *macro*. I recommend just copying the macro until you feel ready to learn Rust's (impressive) macro system.

```rust
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}
```

The short version of what it does is that it takes your ECS as the first parameter, and a tuple with your entity store and "markers" stores in it (you'll see this in a moment). Every parameter after that is a *type* - listing a type stored in your ECS. These are *repeating* rules, so it issues one `SerializeComponent::serialize` call per type. It's not as efficient as doing them all at once, but it works - and doesn't fall over when you exceed 16 types! The `save_game` function then looks like this:

```rust
pub fn save_game(ecs : &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper{ map : mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster, 
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, SerializationHelper
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}
```

What's going on here, then? 

1. We start by creating a new component type - `SerializationHelper` that stores a copy of the map (see, we are using the map stuff from above!). It then creates a new entity, and gives it the new component - with a copy of the map (the `clone` command makes a deep copy). This is needed so we don't need to serialize the map separately.
2. We enter a block to avoid borrow-checker issues.
3. We set `data` to be a tuple, containing the `Entity` store and `ReadStorage` for `SimpleMarker`. These will be used by the save macro.
4. We open a `File` called `savegame.json` in the current directory.
5. We obtain a JSON serializer from Serde.
6. We call the `serialize_individually` macro with all of our types.
7. We delete the temporary helper entity we created.

If you `cargo run` and start a game, then save it - you'll find a `savegame.json` file has appeared - with your game state in it. Yay!

# Restoring Game State

Now that we have the game data, it's time to load it!

## Is there a saved game?

First, we need to know if there *is* a saved game to load. In `saveload_system.rs`, we add the following function:

```rust
pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}
```

Then in `gui.rs`, we extend the `main_menu` function to check for the existence of a file - and not offer to load it if it isn't there:

```rust
pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rust Roguelike Tutorial");
    
    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
            } else {
                ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

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
```

Finally, we'll modify the calling code in `main.rs` to call game loading:

```rust
RunState::MainMenu{ .. } => {
    let result = gui::main_menu(self, ctx);
    match result {
        gui::MainMenuResult::NoSelection{ selected } => newrunstate = RunState::MainMenu{ menu_selection: selected },
        gui::MainMenuResult::Selected{ selected } => {
            match selected {
                gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                gui::MainMenuSelection::LoadGame => {
                    saveload_system::load_game(&mut self.ecs);
                    newrunstate = RunState::AwaitingInput;
                }
                gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
            }
        }
    }
}
```

## Actually loading the game

In `saveload_system.rs`, we're going to need another macro! This is pretty much the same as the `serialize_individually` macro - but reverses the process, and includes some slight changes:

```rust
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}
```

This is called from a new function, `load_game`:

```rust
pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (&mut ecs.entities(), &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(), &mut SimpleMarkerAllocator::<SerializeMe>::new());

        deserialize_individually!(ecs, de, d, Position, Renderable, Player, Viewshed, Monster, 
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, SerializationHelper
        );
    }

    let mut deleteme : Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e,h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
            deleteme = Some(e);
        }
        for (e,_p,pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
}
```

That's quite the mouthful, so lets step through it:

1. Inside a block (to keep the borrow checker happy), we iterate all entities in the game. We add them to a vector, and then iterate the vector - deleting the entities. This is a two-step process to avoid invalidating the iterator in the first pass.
2. We open the `savegame.json` file, and attach a JSON deserializer.
3. Then we build the tuple for the macro, which requires mutable access to the entities store, write access to the marker store, and an allocator (from Specs).
4. Now we pass that to the macro we just made, which calls the de-serializer for each type in turn. Since we saved in the same order, it will pick up everything.
5. Now we go into another block, to avoid borrow conflicts with the previous code and the entity deletion.
6. We first iterate all entities with a `SerializationHelper` type. If we find it, we get access to the resource storing the map - and replace it. Since we aren't serializing `tile_content`, we replace it with an empty set of vectors.
7. Then we find the player, by iterating entities with a `Player` type and a `Position` type. We store the world resources for the player entity and his/her position.
8. Finally, we delete the helper entity - so we won't have a duplicate if we save the game again.

If you `cargo run` now, you can load your saved game!

# Just add permadeath!

It wouldn't really be a roguelike if we let you keep your save game after you reload! So we'll add one more function to `saveload_system`:

```rust
pub fn delete_save() {
    if Path::new("./savegame.json").exists() { std::fs::remove_file("./savegame.json").expect("Unable to delete file"); } 
}
```

We'll add a call to `main.rs` to delete the save after we load the game:

```rust
gui::MainMenuSelection::LoadGame => {
    saveload_system::load_game(&mut self.ecs);
    newrunstate = RunState::AwaitingInput;
    saveload_system::delete_save();
}
```

## Web Assembly

The example as-is will compile and run on the web assembly (`wasm32`) platform: but as soon as you try to save the game, it crashes. Unfortunately (well, fortunately if you like your computer not being attacked by every website you go to!), `wasm` is sandboxed - and doesn't have the ability to save files locally.

Supporting saving via `LocalStorage` (a browser/JavaScript feature) is planned for a future version of RLTK. In the meantime, we'll add some wrappers to avoid the crash - and simply not actually save the game on `wasm32`.

Rust offers *conditional compilation* (if you are familiar with C, it's a lot like the `#define` madness you find in big, cross-platform libraries). In `saveload_system.rs`, we'll modify `save_game` to only compile on non-web assembly platforms:

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs : &mut World) {
```

That `#` tag is scary looking, but it makes sense if you unwrap it. `#[cfg()]` means "only compile if the current configuration matches the contents of the parentheses. `not()` inverts the result of a check, so when we check that `target_arch = "wasm32")` (are we compiling for `wasm32`) the result is inverted. The end result of this is that the function only compiles if you *aren't* building for `wasm32`.

That's all well and good, but there are calls to that function - so compilation on `wasm` will fail. We'll add a *stub* function to take its place:

```rust
#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs : &mut World) {
}
```

The `#[cfg(target_arch = "wasm32")]` prefix means "only compile this for web assembly". We've kept the function signature the same, but added a `_` before `_ecs` - telling the compiler that we intend not to use that variable. Then we keep the function empty.

The result? You can compile for `wasm32` and the `save_game` function simply doesn't *do* anything at all. The rest of the structure remains, so the game correctly returns to the main menu - but with no resume function.

(Why does the check that the file exists work? Rust is smart enough to say "no filesystem, so the file can't exist". Thanks, Rust!)

# Wrap-up

This has been a long chapter, with quite heavy content. The great news is that we now have a framework for loading and saving the game whenever we want to. Adding components has gained some steps: we have to register them in `main`, tag them for `Serialize, Deserialize`, and remember to add them to our component type lists in `saveload_system.rs`. That could be easier - but it's a very solid foundation.

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-11-loadsave)**

---

Copyright (C) 2019, Herbert Wolverson.

---