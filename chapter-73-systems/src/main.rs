extern crate serde;
use rltk::{Console, GameState, Rltk, Point};
extern crate specs;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
#[macro_use]
extern crate specs_derive;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;
pub mod saveload_system;
pub mod random_table;
pub mod rex_assets;
pub mod map_builders;
pub mod raws;
mod gamesystem;
pub use gamesystem::*;
pub mod effects;
#[macro_use]
extern crate lazy_static;
mod systems;
pub mod rng;

const SHOW_MAPGEN_VISUALIZER : bool = false;
const SHOW_FPS : bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode { Buy, Sell }

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range : i32, item : Entity},
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode : VendorMode },
    TeleportingToOtherLevel { x: i32, y: i32, depth: i32 },
    ShowRemoveCurse,
    ShowIdentify
}

pub struct State {
    pub ecs: World,
    mapgen_next_state : Option<RunState>,
    mapgen_history : Vec<Map>,
    mapgen_index : usize,
    mapgen_timer : f32,
    dispatcher : Box<dyn systems::UnifiedDispatcher + 'static>
}

impl State {
    fn run_systems(&mut self) {
        self.dispatcher.run_now(&mut self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    #[allow(clippy::cognitive_complexity)]
    fn tick(&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        systems::particle_system::update_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver{..} => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        match newrunstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                } else {
                    ctx.cls();
                    if self.mapgen_index < self.mapgen_history.len() && self.mapgen_index < self.mapgen_history.len() { camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx); }

                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > 250.0 {
                        self.mapgen_timer = 0.0;
                        self.mapgen_index += 1;
                        if self.mapgen_index >= self.mapgen_history.len() {
                            //self.mapgen_index -= 1;
                            newrunstate = self.mapgen_next_state.unwrap();
                        }
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
                if newrunstate != RunState::AwaitingInput {
                    crate::gamelog::record_event("Turn", 1);
                }
            }
            RunState::Ticking => {
                let mut should_change_target = false;
                while newrunstate == RunState::Ticking {
                    self.run_systems();
                    self.ecs.maintain();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            newrunstate = RunState::AwaitingInput;
                            should_change_target = true;
                        }
                        RunState::MagicMapReveal{ .. } => newrunstate = RunState::MagicMapReveal{ row: 0 },
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel{ x, y, depth } => newrunstate = RunState::TeleportingToOtherLevel{ x, y, depth },
                        RunState::ShowRemoveCurse => newrunstate = RunState::ShowRemoveCurse,
                        RunState::ShowIdentify => newrunstate = RunState::ShowIdentify,
                        _ => newrunstate = RunState::Ticking
                    }
                }
                if should_change_target {
                    player::end_turn_targeting(&mut self.ecs);
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting{ range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item: item_entity, target: None }).expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::Reveal => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToRemoveItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveCurse => {
                let result = gui::remove_curse_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        self.ecs.write_storage::<CursedItem>().remove(item_entity);
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowIdentify => {
                let result = gui::identify_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        if let Some(name) = self.ecs.read_storage::<Name>().get(item_entity) {
                            let mut dm = self.ecs.fetch_mut::<MasterDungeonMap>();
                            dm.identified_items.insert(name.name.clone());
                        }
                        newrunstate = RunState::Ticking;
                    }
                }
            }
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
            RunState::ShowVendor{vendor, mode} => {
                use crate::raws::*;
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        let price = self.ecs.read_storage::<Item>().get(result.1.unwrap()).unwrap().base_value * 0.8;
                        self.ecs.write_storage::<Pools>().get_mut(*self.ecs.fetch::<Entity>()).unwrap().gold += price;
                        self.ecs.delete_entity(result.1.unwrap()).expect("Unable to delete");
                    }
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        let mut identified = self.ecs.write_storage::<IdentifiedItem>();
                        identified.insert(*player_entity, IdentifiedItem{ name : tag.clone() }).expect("Unable to insert");
                        std::mem::drop(identified);
                        let player_pools = pools.get_mut(*player_entity).unwrap();
                        std::mem::drop(player_entity);
                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            crate::raws::spawn_named_item(&RAWS.lock().unwrap(), &mut self.ecs, &tag, SpawnType::Carried{ by: player_entity });
                        }
                    }
                    gui::VendorResult::BuyMode => newrunstate = RunState::ShowVendor{ vendor, mode: VendorMode::Buy },
                    gui::VendorResult::SellMode => newrunstate = RunState::ShowVendor{ vendor, mode: VendorMode::Sell }
                }
            }
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
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MapGeneration;
                        self.mapgen_next_state = Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame });
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::LoadGame };
            }
            RunState::NextLevel => {
                self.goto_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TownPortal => {
                // Spawn the portal
                spawner::spawn_town_portal(&mut self.ecs);

                // Transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - (map_depth-1);
                self.goto_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel{x, y, depth} => {
                self.goto_level(depth-1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<rltk::Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::MagicMapReveal{row} => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32,row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height-1 {
                    newrunstate = RunState::Ticking;
                } else {
                    newrunstate = RunState::MagicMapReveal{ row: row+1 };
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        rltk::render_draw_buffer(ctx);
        if SHOW_FPS {
            ctx.print(1, 59, &format!("FPS: {}", ctx.fps));
        }
    }
}

impl State {
    fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // Build a new map and place the player
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        // Notify the player
        gamelog::Logger::new().append("You change level.").log();
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Replace the world maps
        self.ecs.insert(map::MasterDungeonMap::new());

        // Build a new map and place the player
        self.generate_world_map(1, 0);
    }

    fn generate_world_map(&mut self, new_depth : i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }

        gamelog::clear_log();
        gamelog::Logger::new()
            .append("Welcome to")
            .color(rltk::CYAN)
            .append("Rusty Roguelike")
            .log();

        gamelog::clear_events();
    }
}

fn main() {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple(80, 60)
        .with_title("Roguelike Tutorial")
        .with_font("vga8x16.png", 8, 16)
        .with_sparse_console(80, 30, "vga8x16.png")
        .with_vsync(false)
        .build();
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state : Some(RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index : 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
        dispatcher: systems::build()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<Weapon>();
    gs.ecs.register::<Wearable>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<MagicItem>();
    gs.ecs.register::<ObfuscatedName>();
    gs.ecs.register::<IdentifiedItem>();
    gs.ecs.register::<SpawnParticleBurst>();
    gs.ecs.register::<SpawnParticleLine>();
    gs.ecs.register::<CursedItem>();
    gs.ecs.register::<ProvidesRemoveCurse>();
    gs.ecs.register::<ProvidesIdentification>();
    gs.ecs.register::<AttributeBonus>();
    gs.ecs.register::<Duration>();
    gs.ecs.register::<StatusEffect>();
    gs.ecs.register::<KnownSpells>();
    gs.ecs.register::<SpellTemplate>();
    gs.ecs.register::<WantsToCastSpell>();
    gs.ecs.register::<TeachesSpell>();
    gs.ecs.register::<ProvidesMana>();
    gs.ecs.register::<Slow>();
    gs.ecs.register::<DamageOverTime>();
    gs.ecs.register::<SpecialAbilities>();
    gs.ecs.register::<TileSize>();
    gs.ecs.register::<OnDeath>();
    gs.ecs.register::<AlwaysTargetsSelf>();
    gs.ecs.register::<Target>();
    gs.ecs.register::<WantsToShoot>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration{} );
    gs.ecs.insert(systems::particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs);
}
