# Magic Spells - or Finally A Use For That Blue Mana Bar

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

The last few chapters have been building up to making this one possible: spellcasting. We've had a blue mana bar onscreen for quite a while, now we make it do something useful!

## Knowing Spells

Spellcasting is an optional way to play the game - you can do quite well bashing things if you prefer. It's a common feature in roleplaying games that you can't cast a spell until you know it; you study hard, learn the gestures and incantations and can now unleash your mighty magical powers on the world.

The first implication of this is that an entity needs to be able to *know* spells. A nice side-effect of this is that it gives a convenient way for us to adds special attacks to monsters - we'll cover that later. For now, we'll add a new component to `components.rs` (and register in `main.rs` and `saveload_system.rs`):

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub display_name : String,
    pub mana_cost : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub spells : Vec<KnownSpell>
}
```

We'll also add it to `spawner.rs`'s `player` function. Eventually, we'll blank the spells list (just set it to `Vec::new()`, but for now we're going to add *Zap* as a placeholder):

```rust
.with(KnownSpells{ spells : vec![ KnownSpell{ display_name : "Zap".to_string(), mana_cost: 1 } ] })
```

If you'll remember back in [section 4.9](chapter_52.html), we specified that the user interface should list spells you can cast. Our intended interface looked like this:

![Screenshot](./c52-s1.jpg)

Now we have the data required to fill this out! Open up `gui.rs`, and find the part of `draw_ui` that renders consumables. Right underneath it, insert the following code:

```rust
// Spells
y += 1;
let blue = RGB::named(rltk::CYAN);
let known_spells_storage = ecs.read_storage::<KnownSpells>();
let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
let mut index = 1;
for spell in known_spells.iter() {
    ctx.print_color(50, y, blue, black, &format!("^{}", index));
    ctx.print_color(53, y, blue, black, &format!("{} ({})", spell.display_name, spell.mana_cost));
    index += 1;
    y += 1;
}
```

This reads the `KnownSpells` component (the player *must* have one), extracts the list and uses it to render spells with hotkey listings. We've made the blue into a cyan for readability, but it looks about right:

![Screenshot](./c66-s1.jpg)

## Casting Spells

Displaying the spells is a good start, but we need to be able to actually *cast* (or try to cast) them! You may remember in `player.rs` we handled consumable hotkeys. We'll use a very similar system to handle spell hotkeys. In `player_input`, add the following:

```rust
if ctx.control && ctx.key.is_some() {
    let key : Option<i32> =
        match ctx.key.unwrap() {
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None
        };
    if let Some(key) = key {
        return use_spell_hotkey(gs, key-1);
    }
}
```

That's *just* like the consumable hotkey code (a wise user would refactor some of this into a function, but we'll keep it separated for clarity in the tutorial). It calls `use_spell_hotkey` - which we haven't written yet! Let's go ahead and make a start:

```rust
fn use_spell_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::KnownSpells;

    let player_entity = gs.ecs.fetch::<Entity>();
    let known_spells_storage = gs.ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;

    if (key as usize) < known_spells.len() {
        let pools = gs.ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.mana.current >= known_spells[key as usize].mana_cost {
            // TODO: Cast the Spell
        } else {
            let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
            gamelog.entries.insert(0, "You don't have enough mana to cast that!".to_string());
        }
    }

    RunState::Ticking
}
```

Notice the big `TODO` in there! We need to put some infrastructure in place before we can actually make the spell-casting happen.

### Defining our Zap Spell

The primary reason we hit a bit of a wall there is that we haven't actually told the engine what a `Zap` spell *does*. We've defined everything else in our raw `spawns.json` file, so lets go ahead and make a new `spells` section:

```json
"spells" : [
    {
        "name" : "Zap",
        "effects" : { 
            "ranged" : "6",
            "damage" : "5",
            "particle_line" : "▓;#00FFFF;200.0"
        }
    }
]
```

Let's extend our `raws` system to be able to read this, and make it available for use in-game. We'll start with a new file, `raws/spell_structs.rs` which will define what a spell looks like to the JSON system:

```rust
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Spell {
    pub name : String,
    pub effects : HashMap<String, String>
}
```

Now we'll add `mod spells; pub use spells::Spell;` to `raws/mod.rs` and extend the `Raws` struct to include it:

```rust
mod spell_structs;
pub use spell_structs::Spell;
...
#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>,
    pub mobs : Vec<Mob>,
    pub props : Vec<Prop>,
    pub spawn_table : Vec<SpawnTableEntry>,
    pub loot_tables : Vec<LootTable>,
    pub faction_table : Vec<FactionInfo>,
    pub spells : Vec<Spell>
}
```

Now that we've made the field, we should add it to the `empty()` system in `raws/rawmaster.rs`. We'll also add an index, just like the other raw types:

```rust
pub struct RawMaster {
    raws : Raws,
    item_index : HashMap<String, usize>,
    mob_index : HashMap<String, usize>,
    prop_index : HashMap<String, usize>,
    loot_index : HashMap<String, usize>,
    faction_index : HashMap<String, HashMap<String, Reaction>>,
    spell_index : HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws : Raws{
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table : Vec::new(),
                spells : Vec::new()
            },
            item_index : HashMap::new(),
            mob_index : HashMap::new(),
            prop_index : HashMap::new(),
            loot_index : HashMap::new(),
            faction_index : HashMap::new(),
            spell_index : HashMap::new()
        }
    }
```

And in `load`, we need to populate the index:

```rust
for (i,spell) in self.raws.spells.iter().enumerate() {
    self.spell_index.insert(spell.name.clone(), i);
}
```

We're tying the spell design very heavily to the existing item effects system, but now we hit another minor issue: we're not actually spawning spells as entities - in some cases, they just go straight into the effects system. However, it would be nice to keep using all of the effect code we've written. So we're going to spawn *template* entities for spells. This allows us to find the spell template, and use the existing code to spawn its results. First, in `components.rs` (and registered in `main.rs` and `saveload_system.rs`), we'll make a new `SpellTemplate` component:

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SpellTemplate {
    pub mana_cost : i32
}
```

In `raws/rawmaster.rs` we'll need a new function: `spawn_named_spell`:

```rust
pub fn spawn_named_spell(raws: &RawMaster, ecs : &mut World, key : &str) -> Option<Entity> {
    if raws.spell_index.contains_key(key) {
        let spell_template = &raws.raws.spells[raws.spell_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = eb.with(SpellTemplate{ mana_cost : spell_template.mana_cost });
        eb = eb.with(Name{ name : spell_template.name.clone() });
        apply_effects!(spell_template.effects, eb);

        return Some(eb.build());
    }
    None
}
```

This is simple: we create a new entity, mark it for serialization and as a spell template, give it a name, and use our existing `effects!` macro to fill out the blanks. Then we return the entity.

We want to do this for *all* spells when a new game starts. We'll start by adding a function to `raws/rawmaster.rs` to call it for all spells:

```rust
pub fn spawn_all_spells(ecs : &mut World) {
    let raws = &super::RAWS.lock().unwrap();
    for spell in raws.raws.spells.iter() {
        spawn_named_spell(raws, ecs, &spell.name);
    }
}
```

Since the player only spawns once, we'll call it at the beginning of `spawner.rs`'s `player` function. That guarantees that it will be present, since not having a player is a fatal bug (and a sad thing!):

```rust
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    spawn_all_spells(ecs);
    ...
```

Finally, we're going to add a utility function (to `raws/rawmaster.rs`) to help us find a spell entity. It's pretty straightforward:

```rust
pub fn find_spell_entity(ecs : &World, name : &str) -> Option<Entity> {
    let names = ecs.read_storage::<Name>();
    let spell_templates = ecs.read_storage::<SpellTemplate>();
    let entities = ecs.entities();

    for (entity, sname, _template) in (&entities, &names, &spell_templates).join() {
        if name == sname.name {
            return Some(entity);
        }
    }
    None
}
```

### Enqueueing Zap

Now that we have Zap defined as a spell template, we can finish up the `spell_hotkeys` system we started earlier. First, we'll need a component to indicate a desire to cast a spell. In `components.rs` (and registered in `main.rs` and `saveload_system.rs`):

```rust
// See wrapper below for serialization
#[derive(Component, Debug)]
pub struct WantsToCastSpell {
    pub spell : Entity,
    pub target : Option<rltk::Point>
}
```

Because it's a component with `Entity` data, we also need the helper:

```rust
// WantsToCastSpell wrapper
#[derive(Serialize, Deserialize, Clone)]
pub struct WantsToCastSpellData<M>(M, Option<rltk::Point>);

impl<M: Marker + Serialize> ConvertSaveload<M> for WantsToCastSpell
where
    for<'de> M: Deserialize<'de>,
{
    type Data = WantsToCastSpellData<M>;
    type Error = NoError;

    fn convert_into<F>(&self, mut ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>,
    {
        let marker = ids(self.spell).unwrap();
        Ok(WantsToCastSpellData(marker, self.target))
    }

    fn convert_from<F>(data: Self::Data, mut ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>,
    {
        let spell = ids(data.0).unwrap();
        let target = data.1;
        Ok(WantsToCastSpell{spell, target})
    }
}
```

This gives us enough to finish up spellcasting in `player.rs`:

```rust
fn use_spell_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::KnownSpells;
    use super::raws::find_spell_entity;

    let player_entity = gs.ecs.fetch::<Entity>();
    let known_spells_storage = gs.ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;

    if (key as usize) < known_spells.len() {
        let pools = gs.ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.mana.current >= known_spells[key as usize].mana_cost {
            if let Some(spell_entity) = find_spell_entity(&gs.ecs, &known_spells[key as usize].display_name) {
                use crate::components::Ranged;
                if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(spell_entity) {
                    return RunState::ShowTargeting{ range: ranged.range, item: spell_entity };
                };
                let mut intent = gs.ecs.write_storage::<WantsToCastSpell>();
                intent.insert(
                    *player_entity,
                    WantsToCastSpell{ spell: spell_entity, target: None }
                ).expect("Unable to insert intent");
                return RunState::Ticking;
            }
        } else {
            let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
            gamelog.entries.insert(0, "You don't have enough mana to cast that!".to_string());
        }
    }

    RunState::Ticking
}
```

You'll notice that we're re-using `ShowTargeting` - but with a spell entity instead of an item. We need to adjust the conditions in `main.rs` to handle this:

```rust
RunState::ShowTargeting{range, item} => {
    let result = gui::ranged_target(self, ctx, range);
    match result.0 {
        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
        gui::ItemMenuResult::NoResponse => {}
        gui::ItemMenuResult::Selected => {
            if self.ecs.read_storage::<SpellTemplate>().get(item).is_some() {
                let mut intent = self.ecs.write_storage::<WantsToCastSpell>();
                intent.insert(*self.ecs.fetch::<Entity>(), WantsToCastSpell{ spell: item, target: result.1 }).expect("Unable to insert intent");
                newrunstate = RunState::Ticking;
            } else {
                let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1 }).expect("Unable to insert intent");
                newrunstate = RunState::Ticking;
            }
        }
    }
}
```

So when a target is selected, it looks at the `item` entity - if it has a spell component, it launches a `WantsToCastSpell` - otherwise it sticks with `WantsToUseItem`.

You've hopefully noticed that we're not actually using `WantsToCastSpell` anywhere! We'll need another system to handle it. It's basically the same as using an item, so we'll add it in next to it. In `inventory_system/use_system.rs`, we'll add a second system:

```rust
use specs::prelude::*;
use super::{Name, WantsToUseItem,Map, AreaOfEffect, EquipmentChanged, IdentifiedItem, WantsToCastSpell};
use crate::effects::*;
...
// The ItemUseSystem goes here
...
pub struct SpellUseSystem {}

impl<'a> System<'a> for SpellUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToCastSpell>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, EquipmentChanged>,
                        WriteStorage<'a, IdentifiedItem>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, map, entities, mut wants_use, names,
            aoe, mut dirty, mut identified_item) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged{}).expect("Unable to insert");

            // Identify
            if entity == *player_entity {
                identified_item.insert(entity, IdentifiedItem{ name: names.get(useitem.spell).unwrap().name.clone() })
                    .expect("Unable to insert");
            }

            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::SpellUse{ spell : useitem.spell },
                match useitem.target {
                    None => Targets::Single{ target: *player_entity },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(useitem.spell) {
                            Targets::Tiles{ tiles: aoe_tiles(&*map, target, aoe.radius) }
                        } else {
                            Targets::Tile{ tile_idx : map.xy_idx(target.x, target.y) as i32 }
                        }
                    }
                }
            );

        }

        wants_use.clear();
    }
}
```

This is *very* similar to the `ItemUseSystem`, but takes `WantsToCastSpell` as input. It then sends an `EffectType::SpellUse` to the effects system. We haven't written that yet - so let's do that. We'll start by adding it to the `EffectType` enumeration:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    SpellUse { spell: Entity },
    WellFed,
    Healing { amount : i32 },
    Confusion { turns : i32 },
    TriggerFire { trigger: Entity },
    TeleportTo { x:i32, y:i32, depth: i32, player_only : bool },
    AttributeEffect { bonus : AttributeBonus, name : String, duration : i32 }
}
```

Then we need to add it into the `spell_applicator` function:

```rust
fn target_applicator(ecs : &mut World, effect : &EffectSpawner) {
    if let EffectType::ItemUse{item} = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
    } else if let EffectType::SpellUse{spell} = effect.effect_type {
        triggers::spell_trigger(effect.creator, spell, &effect.targets, ecs);
    } else if let EffectType::TriggerFire{trigger} = effect.effect_type {
        triggers::trigger(effect.creator, trigger, &effect.targets, ecs);
    } else {
        match &effect.targets {
            Targets::Tile{tile_idx} => affect_tile(ecs, effect, *tile_idx),
            Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
            Targets::Single{target} => affect_entity(ecs, effect, *target),
            Targets::TargetList{targets} => targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity)),
        }
    }
}
```

This is sending spell-casting to a new trigger function, `spell_trigger`. This is defined in `triggers.rs`:

```rust
pub fn spell_trigger(creator : Option<Entity>, spell: Entity, targets : &Targets, ecs: &mut World) {
    if let Some(template) = ecs.read_storage::<SpellTemplate>().get(spell) {
        let mut pools = ecs.write_storage::<Pools>();
        if let Some(caster) = creator {
            if let Some(pool) = pools.get_mut(caster) {
                if template.mana_cost <= pool.mana.current {
                    pool.mana.current -= template.mana_cost;
                }
            }
        }
    }
    event_trigger(creator, spell, targets, ecs);
}
```

This is relatively simple. It:

* Checks that there is a spell template attached to the input.
* Obtains the caster's pools, to gain access to their mana.
* Reduces the caster's mana by the cost of the spell.
* Sends the spell over to the effects system - which we've already written.

We'll also want to fix a visual issue. Previously, `find_item_position` (in `effects/targeting.rs`) has always sufficed for figuring out where to start some visual effects. Since the item is now a spell template - and has no position - visual effects aren't going to work. We'll add an additional parameter - owner - to the function and it can fall back to the owner's position:

```rust
pub fn find_item_position(ecs: &World, target: Entity, creator: Option<Entity>) -> Option<i32> {
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();

    // Easy - it has a position
    if let Some(pos) = positions.get(target) {
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }

    // Maybe it is carried?
    if let Some(carried) = ecs.read_storage::<InBackpack>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // Maybe it is equipped?
    if let Some(equipped) = ecs.read_storage::<Equipped>().get(target) {
        if let Some(pos) = positions.get(equipped.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // Maybe the creator has a position?
    if let Some(creator) = creator {
        if let Some(pos) = positions.get(creator) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }

    // No idea - give up
    None
}
```

Then we just need to make a small change in `event_trigger` (in `effects/triggers.rs`):

```rust
// Line particle spawn
if let Some(part) = ecs.read_storage::<SpawnParticleLine>().get(entity) {
    ...
```

And there you have it. If you `cargo run` now, you can press `ctrl+1` to zap people!

## Restoring Mana

You may notice that you never actually get your mana back, right now. You get to zap a few times (4 by default), and then you're done. While that's very 1st Edition D&D-like, it's not a lot of fun for a video game. On the other hand, spells are *powerful* - so we don't want it to be too easy to be the Energizer Bunny of magic!

### Mana Potions

A good start would be to provide *Mana Potions* to restore your magical thirst. In `spawns.json`:

```json
{
    "name" : "Mana Potion",
    "renderable": {
        "glyph" : "!",
        "fg" : "#FF00FF",
        "bg" : "#000000",
        "order" : 2
    },
    "consumable" : {
        "effects" : { "provides_mana" : "4" }
    },
    "weight_lbs" : 0.5,
    "base_value" : 50.0,
    "vendor_category" : "alchemy",
    "magic" : { "class" : "common", "naming" : "potion" }
},
```

And make them a plentiful spawn:

```json
{ "name" : "Mana Potion", "weight" : 7, "min_depth" : 0, "max_depth" : 100 },
```

We don't have support for `provides_mana` yet, so we'll need to make a component for it. In `components.rs` (and `main.rs` and `saveload_system.rs`):

```rust
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesMana {
    pub mana_amount : i32
}
```

And in `raws/rawmaster.rs` we add it as a spawn effect:

```rust
macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
        let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => $eb = $eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() }),
                "provides_mana" => $eb = $eb.with(ProvidesMana{ mana_amount: effect.1.parse::<i32>().unwrap() }),
                "ranged" => $eb = $eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() }),
                "damage" => $eb = $eb.with(InflictsDamage{ damage : effect.1.parse::<i32>().unwrap() }),
                "area_of_effect" => $eb = $eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }),
                "confusion" => {
                    $eb = $eb.with(Confusion{});
                    $eb = $eb.with(Duration{ turns: effect.1.parse::<i32>().unwrap() });
                }
                "magic_mapping" => $eb = $eb.with(MagicMapper{}),
                "town_portal" => $eb = $eb.with(TownPortal{}),
                "food" => $eb = $eb.with(ProvidesFood{}),
                "single_activation" => $eb = $eb.with(SingleActivation{}),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                "remove_curse" => $eb = $eb.with(ProvidesRemoveCurse{}),
                "identify" => $eb = $eb.with(ProvidesIdentification{}),
                _ => println!("Warning: consumable effect {} not implemented.", effect_name)
            }
        }
    };
}
```

That creates the component (you should be used to this by now!), so we also need to handle the *effect* of using it. We'll start by making a new `EffectType` in `effects/mod.rs`:

```rust
#[derive(Debug)]
pub enum EffectType { 
    Damage { amount : i32 },
    Bloodstain,
    Particle { glyph: u8, fg : rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    SpellUse { spell: Entity },
    WellFed,
    Healing { amount : i32 },
    Mana { amount : i32 },
    Confusion { turns : i32 },
    TriggerFire { trigger: Entity },
    TeleportTo { x:i32, y:i32, depth: i32, player_only : bool },
    AttributeEffect { bonus : AttributeBonus, name : String, duration : i32 }
}
```

We'll mark it as affecting entities:

```rust
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::WellFed => true,
        EffectType::Healing{..} => true,
        EffectType::Mana{..} => true,
        EffectType::Confusion{..} => true,
        EffectType::TeleportTo{..} => true,
        EffectType::AttributeEffect{..} => true,
        _ => false
    }
}
```

And include it in our `affect_entities` function:

```rust
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain{..} => if let Some(pos) = entity_position(ecs, target) { damage::bloodstain(ecs, pos) },
        EffectType::Particle{..} => if let Some(pos) = entity_position(ecs, target) { particles::particle_to_tile(ecs, pos, &effect) },
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing{..} => damage::heal_damage(ecs, effect, target),
        EffectType::Mana{..} => damage::restore_mana(ecs, effect, target),
        EffectType::Confusion{..} => damage::add_confusion(ecs, effect, target),
        EffectType::TeleportTo{..} => movement::apply_teleport(ecs, effect, target),
        EffectType::AttributeEffect{..} => damage::attribute_effect(ecs, effect, target),
        _ => {}
    }
}
```

Add the following to the triggers list in `effects/triggers.rs` (right underneath Healing):

```rust
// Mana
if let Some(mana) = ecs.read_storage::<ProvidesMana>().get(entity) {
    add_effect(creator, EffectType::Mana{amount: mana.mana_amount}, targets.clone());
    did_something = true;
}
```

Finally, we need to implement `restore_mana` in `effects/damage.rs`:

```rust
pub fn restore_mana(ecs: &mut World, mana: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Mana{amount} = mana.effect_type {
            pool.mana.current = i32::min(pool.mana.max, pool.mana.current + amount);
            add_effect(None, 
                EffectType::Particle{ 
                    glyph: rltk::to_cp437('‼'),
                    fg : rltk::RGB::named(rltk::BLUE),
                    bg : rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0
                }, 
                Targets::Single{target}
            );
        }
    }
}
```

This is pretty much the same as our healing effect - but with Mana instead of Hit Points.

So if you `cargo run` now, you have a decent chance of fining potions that restore your mana.

### Mana Over Time

We already support giving the player health over time, if they rest away from enemies. It makes sense to do the same for mana, but we want it to be *much slower*. Mana is powerful - with a ranged *zap*, you can inflict a bunch of damage with relatively little risk (although positioning is still key, since a wounded enemy can still hurt you). So we want to restore the player's mana when they rest - but *very slowly*. In `player.rs`, the `skip_turn` function handles restoring health. Let's expand the healing portion to sometimes restore a bit of Mana as well:

```rust
if can_heal {
    let mut health_components = ecs.write_storage::<Pools>();
    let pools = health_components.get_mut(*player_entity).unwrap();
    pools.hit_points.current = i32::min(pools.hit_points.current + 1, pools.hit_points.max);
    let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
    if rng.roll_dice(1,6)==1 {
        pools.mana.current = i32::min(pools.mana.current + 1, pools.mana.max);
    }
}
```

This gives a 1 in 6 chance of restoring some mana when you rest, if you are eligible for healing.

## Enemy Spell Use - Special Attacks

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-66-spells)**


[Run this chapter's example with web assembly, in your browser (WebGL2 required)](http://bfnightly.bracketproductions.com/rustbook/wasm/chapter-66-spells)
---

Copyright (C) 2019, Herbert Wolverson.

---