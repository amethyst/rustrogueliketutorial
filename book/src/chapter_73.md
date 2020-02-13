# Scanning The Systems

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

Specs provides a really nice dispatcher system: it can automatically employ concurrency, making your game really zoom. So why aren't we using it? Web Assembly! WASM doesn't support threading in the same way as every other platform, and a Specs application compiled with a dispatcher to WASM dies hard on the first attempt to dispatch the systems. It isn't really fair on desktop applications to suffer from this. Also, the current `run_systems` isn't at all nice to look at:

```rust
fn run_systems(&mut self) {
    let mut mapindex = MapIndexingSystem{};
    mapindex.run_now(&self.ecs);
    let mut vis = VisibilitySystem{};
    vis.run_now(&self.ecs);
    ... // MANY more
```

So the goal of this chapter is to build an interface that detects WASM, and falls back to a single-threaded dispatcher. If WASM isn't around, we'd like to use the Specs dispatcher. We'd also like a nicer interface to our systems - and not have to specify systems more than once.

## Starting the Systems Module

To get started, we'll make a new directory: `src/systems`. This will hold the self-contained *systems* setup, but for now we're going to use it to start building a setup that can handle switching between Specs dispatch for native use, and a single-threaded invoker in WASM. In the new `src/systems` directory, make a file: `mod.rs`. You can leave it empty for now.

> Warning: there's some moderately advanced macros and configuration here. Feel free to use it, and learn how it works later if needs-be. We're 73 chapters in, so my hope is that we're ready!

Make another new directory: `src/systems/dispatcher`. In that folder, place another empty `mod.rs` file.

Now go to `main.rs` and add a `mod systems;` line: this is designed to include it in the compilation (we'll worry about tidy usage later). Modify `src/systems/mod.rs` to include the line `mod dispatcher` - again, this is just to ensure that it gets compiled.

## Generalizing Dispatch

With our specifications/idea, we know that we want a generic way to run the systems - and not care about which underlying setup is active (from the programming perspective). This sounds like a job for a *trait* - traits are (amongst other things) Rust's answer to polymorphism, inheritance (kinda) and interfaces. Add the following to `src/systems/dispatcher/mod.rs`:

```rust
use specs::prelude::World;
use super::*;

pub trait UnifiedDispatcher {
    fn run_now(&mut self, ecs : *mut World);
}
```

This specifies that our `UnifiedDispatcher` trait will offer a method called `run_now`, which takes itself (for state) and the ECS as a mutable parameter.

## Single threaded dispatch

We'll start with the easy case. Add a new file, `src/systems/dispatcher/single_thread.rs`. In `dispatcher/mod.rs`, add the lines `mod single_thread; pub use single_thread::*;`.

Inside `single_thread.rs`, we're going to need some library support - so start with the following imports:

```rust
use super::super::*;
use super::UnifiedDispatcher;
use specs::prelude::*;
```

Next, we need a place to store our systems. We're going to be passing the runnable targets in Specs-style (see below), but for single-threaded execution our goal is to run them in the order in which they were passed. Unlike the previous `run_now`, we're going to make the systems ahead of time and just iterate/execute them - rather than making them afresh each time. Let's start with the structure definition:

```rust
pub struct SingleThreadedDispatcher<'a> {
    pub systems : Vec<Box<dyn RunNow<'a>>>
}
```

There's a bit of added complexity for lifetimes here (we put the `RunNow` trait from Specs on the same lifetime as the structure), but it's simple enough: every system using Specs' systems functionality implements the `RunNow` trait. So we simply store a vector of boxed (since they vary by size, we have to use pointer indirection) `RunNow` traits.

Actually executing them is a bit more difficult. The following method works:

```rust
impl<'a> UnifiedDispatcher for SingleThreadedDispatcher<'a> {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            for sys in self.systems.iter_mut() {
                sys.run_now(&*ecs);
            }
            crate::effects::run_effects_queue(&mut *ecs);
        }
    }
}
```

There may be a better way to write this, but I kept running into lifetime problems. The `World` and the systems both tend to be effectively `'static` - they live for the life of the program. Persuading Rust that this was the case gave me a day-long headache, until I finally decided to just use `unsafe` and trust myself to do the right thing!

Notice that we're taking `World` as a *mutable pointer*, not a regular mutable reference. De-referencing mutable pointers is inherently unsafe: Rust can't be sure that you aren't stomping all over lifetime guarantees. So the `unsafe` block is there to allow us to do just that. Since we add systems and never remove them, and calling systems without a working `World` is going to blow up anyway - we can get away with it. (If anyone wants to give me a safe implementation, I'll gladly use it!). The function simply iterates all the systems, and executes them - and runs the effects queue at the end.

So that's the relatively easy part. The *hard* part is that we want to take Specs' style dispatcher invocation - and turn it into useful systems data. We also want to do it in a way that will work for *either* dispatching type, AND we don't want to declare our systems more than once. After scratching my head for a while, I came up with a *macro* that generates a function:

```rust
macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        fn new_dispatch() -> Box<dyn UnifiedDispatcher + 'static> {
            let mut dispatch = SingleThreadedDispatcher{
                systems : Vec::new()
            };

            $(
                dispatch.systems.push( Box::new( $type {} ));
            )*

            return Box::new(dispatch);
        }
    };
}
```

Macros are always hard to teach; if you aren't careful, they start to look like Perl. They aren't so much generating *code*, as they are generating *syntax* - that will then "cook" into code during compilation. Looking at how Specs builds systems, each system gets a line like this:

```rust
.with(HelloWorld, "hello_world", &[])
```

So we are specifying three pieces of data per system: the system *type*, a name, and an array of strings specifying dependencies. For single-threaded use, we're actually going to ignore the last two (and trust the user to enter systems in the right order). Mapping this to the parameters portion of the macro, we have:

```rust
macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
```

The `$(...),*` means "repeat the contents of this block, 0..*n* times. Then the three parameters are inside parentheses - making them a *tuple*. `$type` is the system's type - and is an *identifier* (rather than a pure type). `$name` and `$deps` are just expressions.

In the body of the macro:

```rust
) => {
    fn new_dispatch() -> Box<dyn UnifiedDispatcher + 'static> {
        let mut dispatch = SingleThreadedDispatcher{
            systems : Vec::new()
        };

        $(
            dispatch.systems.push( Box::new( $type {} ));
        )*

        return Box::new(dispatch);
    }
};
```

We define a new function, called `new_dispatch`. It returns a boxed, dynamic and `'static` `UnifiedDispatcher`. (The macro doesn't define the function until you run it!). It starts by making a new instance of the `SingleThreadedDispatcher` with an empty systems vector. Then it iterates through each *tuple*, pushing an empty system into the vector. Finally, it returns the structure - with a box around it.

We aren't actually *making* the function yet - we just taught Rust how to do it. So in `src/systems/dispatch/mod.rs` we need to define it, along with the systems it needs to use:

```rust
#[macro_use]
mod single_thread;
pub use single_thread::*;

construct_dispatcher!(
    (MapIndexingSystem, "map_index", &[]),
    (VisibilitySystem, "visibility", &[]),
    (EncumbranceSystem, "encumbrance", &[]),
    (InitiativeSystem, "initiative", &[]),
    (TurnStatusSystem, "turnstatus", &[]),
    (QuipSystem, "quips", &[]),
    (AdjacentAI, "adjacent", &[]),
    (VisibleAI, "visible", &[]),
    (ApproachAI, "approach", &[]),
    (FleeAI, "flee", &[]),
    (ChaseAI, "chase", &[]),
    (DefaultMoveAI, "default_move", &[]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee", &[]),
    (RangedCombatSystem, "ranged", &[]),
    (ItemCollectionSystem, "pickup", &[]),
    (ItemEquipOnUse, "equip", &[]),
    (ItemUseSystem, "use", &[]),
    (SpellUseSystem, "spells", &[]),
    (ItemIdentificationSystem, "itemid", &[]),
    (ItemDropSystem, "drop", &[]),
    (ItemRemoveSystem, "remove", &[]),
    (HungerSystem, "hunger", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
```

This defines a `new()` function that simply passes the results of calling `new_dispatch`. The macro call to `construct_dispatcher!` *makes* this function - with all of the system definitions (I included all of them).

## Moving our systems

For easy access, I've moved *all* of our systems (just Specs systems) into the new `systems` module. You can see the implementation details in the [source code](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-73-systems). Moving them is actually pretty simple:

1. Move the system (or system folder) into `systems`.
2. Remove the `mod` and `use` statements from `main.rs` that were looking for it.
3. In the system, replace `use super::` with `use crate::`.
4. Adjust `src/systems/mod.rs` to compile (`mod`) and `use` the system.

The completed `src/systems/mod.rs` looks like this. Notice we've added an easy-access `new` function to obtain a new systems dispatcher:

```rust
mod dispatcher;
pub use dispatcher::UnifiedDispatcher;

// System imports
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod ai;
use ai::*;
mod movement_system;
use movement_system::MovementSystem;
mod trigger_system;
use trigger_system::TriggerSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod ranged_combat_system;
use ranged_combat_system::RangedCombatSystem;
mod inventory_system;
use inventory_system::*;
mod hunger_system;
use hunger_system::HungerSystem;
pub mod particle_system;
use particle_system::ParticleSpawnSystem;
mod lighting_system;
use lighting_system::LightingSystem;

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    dispatcher::new()
}
```

The `particle_system` is a bit different in that it has *other* functions that are used elsewhere. You'll want to find these, and adjust their path to `crate::systems::particle_system::`.

Now open `main.rs`, and add the new dispatcher to the `State`:

```rust
pub struct State {
    pub ecs: World,
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32,
    dispatcher : Box<dyn systems::UnifiedDispatcher + 'static>
}
```

State's initializer changes to (in `fn main()`):

```rust
let mut gs = State {
    ecs: World::new(),
    mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
    mapgen_index : 0,
    mapgen_history: Vec::new(),
    mapgen_timer: 0.0,
    dispatcher: systems::build()
};
```

We can now *massively* simplify our `run_systems` function:

```rust
impl State {
    fn run_systems(&mut self) {
        self.dispatcher.run_now(&mut self.ecs);
        self.ecs.maintain();
    }
}
```

If you `cargo run` the project now, it will behave just as it did before: but the systems execution is now a bit more straightforward. It may even be a a little faster, since we're not remaking systems on every execution.

## Multi-threaded dispatch

We've gained a bit of clarity and organization with the single-threaded dispatcher, but we're not yet unleashing Specs' power! Make a new file, `src/systems/dispatcher/multi_thread.rs`. 

We'll start by making a new structure to hold a Specs dispatcher, and including some references:

```rust
use super::UnifiedDispatcher;
use specs::prelude::*;

pub struct MultiThreadedDispatcher {
    pub dispatcher: specs::Dispatcher<'static, 'static>
}
```

We also need to implement `run_now` (with the `UnifiedDispatcher` trait we created):

```rust
impl<'a> UnifiedDispatcher for MultiThreadedDispatcher {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            self.dispatcher.dispatch(&mut *ecs);
            crate::effects::run_effects_queue(&mut *ecs);
        }
    }
}
```

This is quite simple: it simply tells Specs to "dispatch" the dispatcher we are storing, and then executes the effects queue.

Once again, we need a macro to handle input:

```rust
macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        fn new_dispatch() -> Box<dyn UnifiedDispatcher + 'static> {
            use specs::DispatcherBuilder;

            let dispatcher = DispatcherBuilder::new()
                $(
                    .with($type{}, $name, $deps)
                )*
                .build();

            let dispatch = MultiThreadedDispatcher{
                dispatcher : dispatcher
            };

            return Box::new(dispatch);
        }
    };
}
```

This takes *exactly* the same input as the single-threaded version. That's deliberate: they are designed to be interchangeable. It also makes a `new_dispatch` function, with the same return type. If calls `DispatchBuilder::new` from Specs, and then iterates the macro parameters to add a `.with(...)` line for each set of system data. Finally, it calls `.build` and stores it in the `MultiThreadedDispatcher` struct - and returns itself in a box.

That's actually pretty simple, but leaves one big question: how do we know which one we are going to use? We pretty much only want to use the single-threaded version in WASM32, otherwise we'd like to take advantage of Specs' threading and efficiency. So we modify `src/systems/dispatcher/mod.rs` to include *conditional compilation*:

```rust
#[cfg(target_arch = "wasm32")]
#[macro_use]
mod single_thread;

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
mod multi_thread;

#[cfg(target_arch = "wasm32")]
pub use single_thread::*;

#[cfg(not(target_arch = "wasm32"))]
pub use multi_thread::*;

use specs::prelude::World;
use super::*;

pub trait UnifiedDispatcher {
    fn run_now(&mut self, ecs : *mut World);
}

construct_dispatcher!(
    (MapIndexingSystem, "map_index", &[]),
    (VisibilitySystem, "visibility", &[]),
    (EncumbranceSystem, "encumbrance", &[]),
    (InitiativeSystem, "initiative", &[]),
    (TurnStatusSystem, "turnstatus", &[]),
    (QuipSystem, "quips", &[]),
    (AdjacentAI, "adjacent", &[]),
    (VisibleAI, "visible", &[]),
    (ApproachAI, "approach", &[]),
    (FleeAI, "flee", &[]),
    (ChaseAI, "chase", &[]),
    (DefaultMoveAI, "default_move", &[]),
    (MovementSystem, "movement", &[]),
    (TriggerSystem, "triggers", &[]),
    (MeleeCombatSystem, "melee", &[]),
    (RangedCombatSystem, "ranged", &[]),
    (ItemCollectionSystem, "pickup", &[]),
    (ItemEquipOnUse, "equip", &[]),
    (ItemUseSystem, "use", &[]),
    (SpellUseSystem, "spells", &[]),
    (ItemIdentificationSystem, "itemid", &[]),
    (ItemDropSystem, "drop", &[]),
    (ItemRemoveSystem, "remove", &[]),
    (HungerSystem, "hunger", &[]),
    (ParticleSpawnSystem, "particle_spawn", &[]),
    (LightingSystem, "lighting", &[])
);

pub fn new() -> Box<dyn UnifiedDispatcher + 'static> {
    new_dispatch()
}
```

The single-threaded imports are preceded by a conditional compilation marker:

```rust
#[cfg(target_arch = "wasm32")]
```

This says "only compile the accompanying line IF the target architecture is `wasm32`".

Likewise, the multi-threaded version has the opposite:

```rust
#[cfg(not(target_arch = "wasm32"))]
```

We repeat these for both the `mod` and the `use` statements in the dispatcher. So if you are running WASM, you get `#[macro_use] mod single_thread; use single_thread::*`. If you are running natively, you get `#[macro_use] mod multi_thread; use multi_thread::*`. Rust won't compile a module that isn't included with a `mod` statement: so we only ever build *one* of the dispatcher strategies. Since we are then using it, the macro `construct_dispatcher!` is placed into our local (`systems::dispatcher`) namespace - so our call to the macro runs whichever version we connected to.

This is *compile time dispatch*, and is a very powerful setup. RLTK uses it internally a *lot* to customize itself for the various hardware back-ends.

So if you `cargo run` your project now - the game runs using a Specs dispatcher. If you fire up a system monitor, you can see that it is using multiple threads!

## So why didn't it explode, when we added threads?

It's a common idiom that "I had a bug. I added 8 threads, and now I have 8 bugs." This can be very true, but Rust tries really hard to promote "fearless concurrency". Rust itself protects against *data races* - not allowing two systems to access/change the same data at the same time, which is a common source of bugs in some other languages. It doesn't protect against logical problems, however - such as a system requiring information from a previous system, only that other system hasn't run yet.

Specs takes the safety another step forward on your behalf. Here is the `SystemData` definition from our map indexing system:

```rust
WriteExpect<'a, Map>,
ReadStorage<'a, Position>,
ReadStorage<'a, BlocksTile>,
ReadStorage<'a, TileSize>,
Entities<'a>
```

Remember how we are painstakingly specifying whether we want `Write` or `Read` access to resources and components? Specs actually uses this for scheduling. When it builds the dispatcher, it looks for `Write` access - and ensures that no two systems can have write access to the same data at once. It also ensures that reading data won't happen while it is locked for writing. *However*, systems can concurrently *read* data. So in this case, Specs guarantees that anything that needs to *read* the map will wait until the `MapIndexingSystem` is done *writing* to it.

This has the effect of building a dependency chain - and ordering systems logically. As a shortened example:

* `MapIndexingSystem` writes to the map, and reads `Position`, `BlocksTile` and `TileSize`.
* `VisibilitySystem` writes to the map, `Viewshed`, `Hidden` and `RandomNumberGenerator`. It reads `Position`, `Name`, and `BlocksVisibility`.
* `EncumbranceSystem` writes to `EquipmentChanged`, `Pools`, `Attributes` and reads from `Item`, `InBackpack`, `Equipped`, `Entity`, `AttributeBonus`, `StatusEffect` and `Slow`.
* `InitiativeSystem` writes to `Initiative`, `MyTurn`, `RandomNumberGenerator`, `RunState`, `Duration`, `EquipmentChanged` and reads from `Position`, `Attributes`, `Entity`, `Point`, `Pools`, `StatusEffect`, `DamageOverTime`.
* `TurnStatusSystem` writes to `MyTurn`, and reads from `Confusion`, `RunState`, `StatusEffect`.

We could keep enumerating through all of them, but that's a good illustration. From this, we can determine:

1. `MapIndexingSystem` locks the map, so it can't run concurrently with `VisibilitySystem`. Since `MapIndexingSystem` is defined first, it will run first.
2. `VisibilitySystem` locks the map, viewshed, hidden and RNG. So it has to wait for Visibility system.
3. `EncumbranceSystem` locks the RNG, so it has to wait until `VisibilitySystem` is done.
4. `InitiativeSystem` also locks the RNG, so it has to wait until Encumbrance is done.
5. `TurnStatusSystem` locks `MyTurn` - and so does `InitiativeSystem`. So it will have to wait until that system is done.

In other words: we aren't really multi-threading all that much yet! We are benefitting from efficiency gains by using Specs' dispatcher - so we've gained *some* benefit.


---

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-73-systems)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-73-systems)
---

Copyright (C) 2019, Herbert Wolverson.

---