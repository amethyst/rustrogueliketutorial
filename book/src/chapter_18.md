# Particle Effects in ASCII

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

---

There's no real visual feedback for your actions - you hit something, and it either goes away, or it doesn't. Bloodstains give a good impression of what *previously* happened in a location - but it would be nice to give some sort of instant reaction to your actions. These need to be fast, non-blocking (so you don't have to wait for the animation to finish to keep playing), and not too intrusive. Particles are a good fit for this, so we'll implement a simple ASCII/CP437 particle system.

## Particle component

As usual, we'll start out by thinking about what a particle *is*. Typically it has a position, something to render, and a lifetime (so it goes away). We've already written two out of three of those, so lets go ahead and create a `ParticleLifetime` component. In `components.rs`:

```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms : f32
}
```

We have to register this in all the usual places: `main.rs` and `saveload_system.rs` (twice).

## Spawning particles

We'll start by spawning a particle whenever someone attacks. In `melee_combat_system.rs`, we'll expand the list of resources required for melee:

```rust
type SystemData = ( Entities<'a>,
    WriteExpect<'a, GameLog>,
    WriteStorage<'a, WantsToMelee>,
    ReadStorage<'a, Name>,
    ReadStorage<'a, CombatStats>,
    WriteStorage<'a, SufferDamage>,
    ReadStorage<'a, MeleePowerBonus>,
    ReadStorage<'a, DefenseBonus>,
    ReadStorage<'a, Equipped>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Renderable>,
    WriteStorage<'a, ParticleLifetime>
);
```

Then we'll add some logic to spawn a particle on impact:

```rust
let pos = positions.get(wants_melee.target);
if let Some(pos) = pos {
    let particle = entities.create();
    positions.insert(particle, Position{ x:pos.x, y:pos.y }).expect("Unable to insert position");
    renderables.insert(particle, 
        Renderable{ glyph: rltk::to_cp437('░'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0 }).expect("Unable to insert renderable");
    particles.insert(particle, ParticleLifetime{ lifetime_ms : 100.0 }).expect("Unable to insert particle lifetime");
}
```

This gives a borrow warning (FIXME), but works (I'll fix later, promise). If you `cargo run` your project now, when one entity attacks another a cyan ░ pattern renders in the attack space. Unfortunately, it persists forever!

## Vanishing particles

We want to age each particle by the time since the last frame, each tick. We can do this by modifying the `tick` function in `main.rs`:

```rust
match newrunstate {
    RunState::MainMenu{..} => {}
    RunState::GameOver{..} => {}
    _ => {
        draw_map(&self.ecs, ctx);

        let mut dead_particles : Vec<Entity> = Vec::new();
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

            // Age out particles
            let mut particles = self.ecs.write_storage::<ParticleLifetime>();
            let entities = self.ecs.entities();
            for (entity, mut particle) in (&entities, &mut particles).join() {
                particle.lifetime_ms -= ctx.frame_time_ms;
                if particle.lifetime_ms < 0.0 {
                    dead_particles.push(entity);
                }
            }                    
        }
        for dead in dead_particles.iter() {
            self.ecs.delete_entity(*dead).expect("Particle will not die");
        } 
    }
}
```

You can `cargo run` now, and see the hit effect quickly appear and then vanish after you bash a poor goblin (or it bashes you!).
FIXME: SCREENSHOT

## Adding more effects

It would be nice if the various magical items in the game provide some visual feedback. We'll try and add this in a relatively generic fashion to the item use system in `inventory_system.rs`. First, we'll make a new structure to indicate that we'd like a particle:

```rust
struct ItemParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: u8,
    lifetime: f32
}
```

Then at the top of the system, we'll request a few more containers and initialize a list of particle requests:

```rust
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, ProvidesHealing>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, Position>,
                        ReadExpect<'a, Point>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, ParticleLifetime>
                      );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data : Self::SystemData) {
        let mut particle_requests : Vec<ItemParticleRequest> = Vec::new();
```

At the *bottom* of the system, we'll instantiate our particles in one go:

```rust
wants_use.clear();

for part in particle_requests.iter() {
    let particle = entities.create();
    positions.insert(particle, Position{ x : part.x, y: part.y }).expect("Unable to insert position");
    renderables.insert(particle, Renderable{ fg: part.fg, bg: part.bg, glyph: part.glyph, render_order: 0 }).expect("Unable to insert renderable");
    particle_life.insert(particle, ParticleLifetime{ lifetime_ms: part.lifetime }).expect("Unable to insert particle lifetime");
}
```

Now we need to add some effects. In the *healing* section:

```rust
used_item = true;                            
particle_requests.push(ItemParticleRequest{
    x: player_pos.x,
    y: player_pos.y,
    fg: RGB::from_f32(0., 0.75, 0.),
    bg: RGB::from_f32(0., 0., 0.),
    glyph: rltk::to_cp437('♥'),
    lifetime: 200.0
});
```

And in the *damage* section:

```rust
gamelog.entries.insert(0, format!("You use {} on {}, inflicting {} hp.", item_name.name, mob_name.name, damage.damage));

let mob_pos = positions.get(*mob);
if let Some(mob_pos) = mob_pos {
    particle_requests.push(ItemParticleRequest{
        x: mob_pos.x,
        y: mob_pos.y,
        fg: RGB::named(rltk::ORANGE),
        bg: RGB::from_f32(0., 0., 0.),
        glyph: rltk::to_cp437('▒'),
        lifetime: 200.0
    });
}
```

And in the *confusion* section:

```rust
let mob_pos = positions.get(*mob);
if let Some(mob_pos) = mob_pos {
    particle_requests.push(ItemParticleRequest{
        x: mob_pos.x,
        y: mob_pos.y,
        fg: RGB::from_f32(0., 0., 0.75),
        bg: RGB::from_f32(0., 0., 0.),
        glyph: rltk::to_cp437('?'),
        lifetime: 200.0
    });
}
```

If you `cargo run` the project now, you'll find that healing generates a nice heart over yourself, confusion spams a question mark, and damage-dealing flashes an orange haze. It's subtle, but gives a lot more visceral feel to the game.

FIXME: SCREENSHOT

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-18-particles)**

---

Copyright (C) 2019, Herbert Wolverson.

---