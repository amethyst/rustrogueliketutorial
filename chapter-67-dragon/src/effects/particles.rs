use specs::prelude::*;
use super::*;
use crate::particle_system::ParticleBuilder;
use crate::map::Map;

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
