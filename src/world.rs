use std::any::Any;
use std::io::Read;
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

use crate::constants::CENTER_Y;

pub struct WorldPlugin;

const LEVEL_SAVE: &str = "/home/niedzwiedz/.minecraft/saves/test-117";

const WORLD_POSITION_SCALE: f32 = 1.05;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let env = Arc::new(LevelEnv::with_vanilla());
    let source = AnvilLevelSource::new(LEVEL_SAVE);
    let height = ChunkHeight { min: 0, max: 15 };
    let mut level = Level::new("overworld".to_string(), env, height, source);

    let size = || 0..32;
    let chunk_size = || size().map(|i| i / 16).unique();
    for cx in chunk_size() {
        for cz in chunk_size() {
            eprintln!("requesting chunk {}/{}", cx, cz);
            level.request_chunk_load(cx, cz);
        }
    }

    level.load_chunks_with_callback(|cx, cz, res| {
        eprintln!("chunk loaded");
        if let Ok(chunk) = res {
            if let Ok(chunk) = chunk.read() {
                chunk
                    .iter_sub_chunks()
                    .filter_map(|(i, subchunk)| subchunk.map(|s| (i, s)))
                    .flat_map(|(i, subchunk)| {
                        subchunk
                            .iter_blocks()
                            .enumerate()
                            .map(move |(offset, block_state)| (i, offset, block_state))
                    })
                    .for_each(|(i, offset, block_state)| {
                        let x = offset & 15;
                        let y = (offset >> 8) & 15;
                        let z = (offset >> 4) & 15;
                        let block = block_state.get_block();
                        use mc_vanilla::block as block_type;

                        if block == &block_type::AIR {
                            eprintln!("air at [{},{},{}]", x, y, z);
                            return;
                        }
                        let name = block.get_name();
                        let mut bytes = name.bytes();
                        let scale = |v: u8| v as f32 / u8::MAX as f32;
                        let r = scale(bytes.next().unwrap());
                        let g = scale(bytes.next().unwrap());
                        let b = scale(bytes.next().unwrap());
                        println!("spawning {} at [{},{},{}] with color [{},{},{}]", name, x, y, z, r, g, b);
                        commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                material: materials.add(Color::rgb(r, g, b).into()),
                                transform: Transform::from_xyz(
                                    WORLD_POSITION_SCALE * x as f32,
                                    WORLD_POSITION_SCALE * y as f32,
                                    WORLD_POSITION_SCALE * z as f32,
                                ),
                                ..Default::default()
                            })
                            .insert(RigidBody::Static)
                            .insert(CollisionShape::Cuboid {
                                half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
                                border_radius: None,
                            });
                    });
            }
        }
    });
    // for i in 0..5 {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     level.load_chunks_with_callback(|cx, cz, res| {
    //         match res {
    //             Err(e) => eprintln!("ERR: [{}/{}] {:?}", cx, cz, e),
    //             Ok(c) => {
    //                 // eprintln!("success :: {:?}", c.read().unwrap().get_position());
    //             }
    //         }
    //     });
    //     let count = level.get_loading_chunks_count();
    //     eprintln!("chunk count: {}", count);
    //     // if count == 0 {
    //     //     break;
    //     // }
    // }

    let plane_size = 20.0;
    // size()
    //     .flat_map(move |x| size().flat_map(move |y| size().map(move |z| (x, y, z))))
    //     .filter(|(x, y, z)| match level.chunks.get_block_at(*x, *y, *z) {
    //         Ok(_) => {
    //             // eprint!(".");
    //             true
    //         }
    //         Err(e) => {
    //             // eprintln!("{:?}", e);
    //             false
    //         }
    //     })
    //     .for_each(|(x, y, z)| {
    //         // eprintln!("loading block :: [{}, {}, {}]", x, y, z);
    //         commands
    //             .spawn_bundle(PbrBundle {
    //                 mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //                 material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
    //                 transform: Transform::from_xyz(
    //                     WORLD_POSITION_SCALE * x as f32,
    //                     WORLD_POSITION_SCALE * y as f32,
    //                     WORLD_POSITION_SCALE * z as f32,
    //                 ),
    //                 ..Default::default()
    //             })
    //             .insert(RigidBody::Static)
    //             .insert(CollisionShape::Cuboid {
    //                 half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
    //                 border_radius: None,
    //             });
    //     });
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
