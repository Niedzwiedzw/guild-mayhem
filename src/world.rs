use std::ops::Mul;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use heron::prelude::*;
use itertools::Itertools;
use mc_core::world::anvil::source::AnvilLevelSource;
use mc_core::world::chunk::{Chunk, ChunkHeight, SubChunk};
use mc_core::world::level::{Level, LevelEnv};
use mc_vanilla::ext::WithVanilla;
use rand::Rng;
use rand_seeder::SipHasher;

use crate::constants::CENTER_Y;

pub struct WorldPlugin;

const LEVEL_SAVE: &str = "/home/niedzwiedz/.minecraft/saves/test-117";

const WORLD_POSITION_SCALE: f32 = 1.05;

fn block_color_from_name(name: &str) -> Color {
    let hasher = SipHasher::from(name);
    let mut rng = hasher.into_rng();
    let r = rng.gen();
    let g = rng.gen();
    let b = rng.gen();
    return Color::rgb(r, g, b);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let env = Arc::new(LevelEnv::with_vanilla());
    let source = AnvilLevelSource::new(LEVEL_SAVE);
    let height = ChunkHeight { min: 0, max: 15 };
    let mut level = Level::new("overworld".to_string(), env, height, source);

    let size = || 0..16;

    let chunk_iter = |cx: usize, cz: usize| {
        (cx..(cx + 16))
            .flat_map(move |x| (cz..(cz + 16)).flat_map(move |y| (0..384).map(move |z| (x, y, z))))
            .map(|(x, y, z)| (x as i32, y as i32, z as i32))
    };
    let chunk_size = |cx: usize, cz: usize| {
        chunk_iter(cx, cz)
            .map(|(x, y, z)| (x / 16, z / 16))
            .unique()
    };
    for (cx, cz) in chunk_size(0, 0) {
        level.request_chunk_load(cx, cz);
    }

    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        level.load_chunks_with_callback(|cx, cz, res| {
            match res {
                Err(e) => eprintln!("ERR: [{}/{}] {:?}", cx, cz, e),
                Ok(c) => {
                    // eprintln!("success :: {:?}", c.read().unwrap().get_position());
                }
            }
        });
        let count = level.get_loading_chunks_count();
        eprintln!("chunk count: {}", count);
        if count == 0 && i != 0 {
            break;
        }
    }

    let plane_size = 20.0;
    chunk_iter(0, 0)
        .filter_map(|(x, y, z)| match level.chunks.get_block_at(x, y, z) {
            Ok(block) => {
                let block = &block.get_block();
                let name = block.get_name();
                let color = block_color_from_name(name);

                if [&mc_vanilla::block::AIR].contains(block) {
                    eprintln!("found air");
                    return None;
                }
                eprintln!(
                    "spawning \"{}\" at ({},{},{}) with color ({:?})",
                    name, x, y, z, color
                );
                Some((color, (x, y, z)))
            }
            Err(e) => {
                // eprintln!("{:?}", e);
                None
            }
        })
        .for_each(|(color, (x, y, z))| {
            // eprintln!("loading block :: [{}, {}, {}]", x, y, z);
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(color.into()),
                    transform: Transform::from_xyz(
                        WORLD_POSITION_SCALE * x as f32,
                        WORLD_POSITION_SCALE * z as f32,
                        WORLD_POSITION_SCALE * y as f32,
                    ),
                    ..Default::default()
                })
                .insert(RigidBody::Static)
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
                    border_radius: None,
                });
        });
    // plane
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Plane { size: plane_size })),
    //         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //         ..Default::default()
    //     })
    //     .insert(RigidBody::Static)
    //     .insert(CollisionShape::Cuboid {
    //         half_extends: Vec3::new(plane_size, 0.001, plane_size) / 2.0,
    //         border_radius: None,
    //     });
    // cube
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //         transform: Transform::from_xyz(0.0, CENTER_Y as f32, 0.0),
    //         ..Default::default()
    //     })
    //     .insert(RigidBody::Dynamic)
    //     .insert(CollisionShape::Cuboid {
    //         half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
    //         border_radius: None,
    //     });
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, (CENTER_Y as f32) * 1.2, 4.0),
        ..Default::default()
    });
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
    }
}
