use bevy::{
    app::{Events, ManualEventReader},
    input::{keyboard, mouse::MouseMotion},
    prelude::*,
    render::camera::Camera,
};
use heron::prelude::*;
use std::ops::Add;

use crate::constants::CENTER_Y;

pub struct PlayerPlugin;

pub struct Player;

pub struct Orientation(Quat);

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let position = Vec3::new(-2.0, CENTER_Y as f32, 5.0);
    let orientation = Quat::IDENTITY;
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Player)
        .insert(Transform::from_translation(position))
        .insert(Acceleration::from_linear(Vec3::ZERO))
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(Orientation(orientation))
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(3.2, 2.0, 3.2) / 2.0,
            border_radius: None,
        });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

pub fn camera_follow_system(
    player: Query<(&Player, &Transform, &Orientation), Without<Camera>>,
    mut camera: Query<(&Camera, &mut Transform), Without<Player>>,
) {
    if let Ok((_, player_position, player_orientation)) = player.single() {
        if let Ok((_, mut camera_position)) = camera.single_mut() {
            *camera_position = *player_position;
            camera_position.rotation = player_orientation.0
        }
    }
}

pub fn player_controls_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Acceleration, &Orientation, &mut Velocity), With<Player>>,
) {
    if let Ok((mut acceleration, orientation, mut velocity)) = query.single_mut() {
        let mut acc = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            acc.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            acc.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            acc.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            acc.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            *velocity = Velocity::from_linear(velocity.linear + Vec3::Y * 1.0);
        }

        *acceleration = Acceleration::from_linear(orientation.0.mul_vec3(acc * 10.0));
    }
}

#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

fn cursor_grab_system(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }
}

fn player_look_system(
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&Player, &mut Orientation)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut orientation) in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            if window.cursor_locked() {
                // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                let window_scale = window.height().min(window.width());

                state.pitch -= (0.0001 * ev.delta.y * window_scale).to_radians();
                state.yaw -= (0.0001 * ev.delta.x * window_scale).to_radians();
            }

            state.pitch = state.pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            orientation.0 = Quat::from_axis_angle(Vec3::Y, state.yaw)
                * Quat::from_axis_angle(Vec3::X, state.pitch);
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<InputState>()
            .add_startup_system(spawn_player.system())
            .add_startup_system(initial_grab_cursor.system())
            .add_system(player_controls_system.system())
            .add_system(player_look_system.system())
            .add_system(cursor_grab_system.system())
            .add_system(camera_follow_system.system());
    }
}
