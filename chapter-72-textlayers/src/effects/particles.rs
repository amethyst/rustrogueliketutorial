use specs::prelude::*;
use super::*;
use crate::particle_system::ParticleBuilder;
use crate::map::Map;
use crate::components::{ParticleAnimation, ParticleLifetime, Renderable, Position};
use rltk::{Point, to_cp437};

pub fn particle_to_tile(ecs: &mut World, tile_idx : i32, effect: &EffectSpawner) {
    if let EffectType::Particle{ glyph, fg, bg, lifespan } = effect.effect_type {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        particle_builder.request(
            tile_idx % map.width, 
            tile_idx / map.width, 
            fg, 
            bg, 
            glyph, 
            lifespan
        );
    }
}

pub fn projectile(ecs: &mut World, tile_idx : i32, effect: &EffectSpawner) {
    if let EffectType::ParticleProjectile{ glyph, fg, bg, 
        lifespan, speed, path } = &effect.effect_type 
    {
        let map = ecs.fetch::<Map>();
        let x = tile_idx % map.width;
        let y = tile_idx / map.width;
        std::mem::drop(map);
        ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{ fg: *fg, bg: *bg, glyph: *glyph, render_order: 0 })
            .with(ParticleLifetime{
                lifetime_ms: path.len() as f32 * speed,
                animation: Some(ParticleAnimation{
                    step_time: *speed,
                    path: path.to_vec(),
                    current_step: 0,
                    timer: 0.0
                })
            })
            .build();
    }
}
