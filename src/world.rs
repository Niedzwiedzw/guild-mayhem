use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use heron::prelude::*;
use mc_core::world::anvil::source::AnvilLevelSource;
use mc_core::world::chunk::{Chunk, ChunkHeight, SubChunk};
use mc_core::world::level::{Level, LevelEnv};
use mc_vanilla::ext::WithVanilla;

use crate::constants::CENTER_Y;

pub struct WorldPlugin;

const LEVEL_SAVE: &str = "/home/niedzwiedz/.minecraft/saves/New World";

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let env = Arc::new(LevelEnv::with_vanilla());
    let source = AnvilLevelSource::new(LEVEL_SAVE);
    let height = ChunkHeight { min: 0, max: 126 };
    let mut level = Level::new("overworld".to_string(), env, height, source);

    let size = || 0..32;

    for cx in size() {
        for cz in size() {
            level.request_chunk_load(cx, cz);
        }
    }
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        level.load_chunks();
        let count = level.get_loading_chunks_count();
        eprintln!("chunk count: {}", count);
        if count == 0 {
            break;
        }
    }

    let plane_size = 20.0;
    size().flat_map(move |x| size().flat_map(move |y| size().map(move |z| (x, y, z))))
        .filter(|(x, y, z)| match level.chunks.get_block_at(*x, *y, *z) {
            Ok(_) => {
                // eprint!(".");
                true
            },
            Err(_) => {
                // eprint!("x");
                false
            },
        })
        .for_each(|(x, y, z)| {
            eprintln!("loading block :: [{}, {}, {}]", x, y, z);
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..Default::default()
                })
                .insert(RigidBody::Static)
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
                    border_radius: None,
                });
        });
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: plane_size })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    })
        .insert(
            RigidBody::Static
        )
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(plane_size, 0.001, plane_size) / 2.0,
            border_radius: None,
        });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, CENTER_Y as f32, 0.0),
        ..Default::default()
    })
        .insert(
            RigidBody::Dynamic
        )
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 1.0, 1.0) / 2.0,
            border_radius: None,
        });
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
