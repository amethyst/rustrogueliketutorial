use super::{BuilderChain, XStart, YStart, AreaStartingPosition, 
    CullUnreachable, VoronoiSpawning,
    AreaEndingPosition, XEnd, YEnd, BspInteriorBuilder };

pub fn dark_elf_city(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    println!("Dark elf builder");
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elven City");
    chain.start_with(BspInteriorBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::RIGHT, YStart::CENTER));
    chain.with(AreaEndingPosition::new(XEnd::LEFT, YEnd::CENTER));
    chain.with(VoronoiSpawning::new());
    chain
}

pub fn dark_elf_plaza(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    println!("Dark elf plaza builder");
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elven Plaza");
    chain.start_with(PlazaMapBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain
}

// Plaza Builder
use super::{InitialMapBuilder, BuilderMap, TileType };

pub struct PlazaMapBuilder {}

impl InitialMapBuilder for PlazaMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, build_data : &mut BuilderMap) {
        self.empty_map(build_data);
        self.spawn_zones(build_data);
    }
}

impl PlazaMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<PlazaMapBuilder> {
        Box::new(PlazaMapBuilder{})
    }

    fn empty_map(&mut self, build_data : &mut BuilderMap) {
        build_data.map.tiles.iter_mut().for_each(|t| *t = TileType::Floor);
    }

    fn spawn_zones(&mut self, build_data : &mut BuilderMap) {
        let mut voronoi_seeds : Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < 32 {
            let vx = crate::rng::roll_dice(1, build_data.map.width-1);
            let vy = crate::rng::roll_dice(1, build_data.map.height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32) ; 32];
        let mut voronoi_membership : Vec<i32> = vec![0 ; build_data.map.width as usize * build_data.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % build_data.map.width;
            let y = i as i32 / build_data.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                    rltk::Point::new(x, y),
                    pos.1
                );
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        // Make a list of zone sizes and cull empty ones
        let mut zone_sizes : Vec<(i32, usize)> = Vec::with_capacity(32);
        for zone in 0..32 {
            let num_tiles = voronoi_membership.iter().filter(|z| **z == zone).count();
            if num_tiles > 0 {
                zone_sizes.push((zone, num_tiles));
            }
        }
        zone_sizes.sort_by(|a,b| b.1.cmp(&a.1));

        // Start making zonal terrain
        zone_sizes.iter().enumerate().for_each(|(i, (zone, _))| {
            match i {
                0 => self.portal_park(build_data, &voronoi_membership, *zone, &voronoi_seeds),
                1 | 2 => self.park(build_data, &voronoi_membership, *zone, &voronoi_seeds),
                i if i > 20 => self.fill_zone(build_data, &voronoi_membership, *zone, TileType::Wall),
                _ => {
                    let roll = crate::rng::roll_dice(1, 6);
                    match roll {
                        1 => self.fill_zone(build_data, &voronoi_membership, *zone, TileType::DeepWater),
                        2 => self.fill_zone(build_data, &voronoi_membership, *zone, TileType::ShallowWater),
                        3 => self.stalactite_display(build_data, &voronoi_membership, *zone),
                        _ => {}
                    }
                }
            }
        });

        // Clear the path
        self.make_roads(build_data, &voronoi_membership);
    }

    fn portal_park(&mut self, build_data : &mut BuilderMap, voronoi_membership: &[i32], zone: i32, seeds: &[(usize, rltk::Point)]) {
        let zone_tiles : Vec<usize> = voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .map(|(idx, _)| idx)
            .collect();

        // Start all gravel
        zone_tiles.iter().for_each(|idx| build_data.map.tiles[*idx] = TileType::Gravel);

        // Add the exit
        let center = seeds[zone as usize].1;
        let idx = build_data.map.xy_idx(center.x, center.y);
        build_data.map.tiles[idx] = TileType::DownStairs;

        // Add some altars around the exit
        let altars = [
            build_data.map.xy_idx(center.x - 2, center.y),
            build_data.map.xy_idx(center.x + 2, center.y),
            build_data.map.xy_idx(center.x, center.y - 2),
            build_data.map.xy_idx(center.x, center.y + 2),
        ];
        altars.iter().for_each(|idx| build_data.spawn_list.push((*idx, "Altar".to_string())));

        let demon_spawn = build_data.map.xy_idx(center.x+1, center.y+1);
        build_data.spawn_list.push((demon_spawn, "Vokoth".to_string()));
    }

    fn fill_zone(&mut self, build_data : &mut BuilderMap, voronoi_membership: &[i32], zone: i32, tile_type: TileType) {
        voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .for_each(|(idx, _)| build_data.map.tiles[idx] = tile_type);
    }

    fn stalactite_display(&mut self, build_data : &mut BuilderMap, voronoi_membership: &[i32], zone: i32) {
        voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .for_each(|(idx, _)| {
                build_data.map.tiles[idx] = match crate::rng::roll_dice(1,10) {
                    1 => TileType::Stalactite,
                    2 => TileType::Stalagmite,
                    _ => TileType::Grass,
                };
            });
    }

    fn park(&mut self, build_data : &mut BuilderMap, voronoi_membership: &[i32], zone: i32, seeds: &[(usize, rltk::Point)]) {
        let zone_tiles : Vec<usize> = voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .map(|(idx, _)| idx)
            .collect();

        // Start all grass
        zone_tiles.iter().for_each(|idx| build_data.map.tiles[*idx] = TileType::Grass);

        // Add a stone area in the middle
        let center = seeds[zone as usize].1;
        for y in center.y-2 ..= center.y+2 {
            for x in center.x-2 ..= center.x+2 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Road;
                if crate::rng::roll_dice(1,6) > 2 {
                    build_data.map.bloodstains.insert(idx);
                }
            }
        }

        // With an altar at the center
        build_data.spawn_list.push((
            build_data.map.xy_idx(center.x, center.y),
            "Altar".to_string()
        ));

        // And chairs for spectators, and the spectators themselves
        let available_enemies = match crate::rng::roll_dice(1, 3) {
            1 => vec![
                "Arbat Dark Elf",
                "Arbat Dark Elf Leader",
                "Arbat Orc Slave",
            ],
            2 => vec![
                "Barbo Dark Elf",
                "Barbo Goblin Archer",
            ],
            _ => vec![
                "Cirro Dark Elf",
                "Cirro Dark Priestess",
                "Cirro Spider",
            ]
        };

        zone_tiles.iter().for_each(|idx| {
            if build_data.map.tiles[*idx] == TileType::Grass {
                    match crate::rng::roll_dice(1, 10) {
                    1 => build_data.spawn_list.push((
                            *idx,
                            "Chair".to_string()
                        )),
                    2 => {
                        let to_spawn = crate::rng::range(0, available_enemies.len() as i32);
                        build_data.spawn_list.push((
                            *idx,
                            available_enemies[to_spawn as usize].to_string()
                        ));
                    }
                    _ => {}
                }
            }
        });
    }

    fn make_roads(&mut self, build_data : &mut BuilderMap, voronoi_membership: &[i32]) {
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed { neighbors += 1; }

                if neighbors > 1 {
                    build_data.map.tiles[my_idx] = TileType::Road;
                }
            }
        }
    }
}
