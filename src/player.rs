use std::ops::Add;

use bevy::{app::{Events, ManualEventReader}, input::{keyboard, mouse::MouseMotion}, prelude::*, render::camera::Camera};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Position(Vec3);

#[derive(Component)]
pub struct Orientation(Quat);

pub fn spawn_player(mut commands: Commands) {
    let position = Vec3::new(-2.0, 2.5, 5.0);
    let orientation = Quat::IDENTITY;
    commands.spawn()
        .insert(Player)
        .insert(Position(position))
        .insert(Orientation(orientation));

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

pub fn camera_follow_system(
    player: Query<(&Player, &Position, &Orientation)>,
    mut camera: Query<(&Camera, &mut Transform)>
) {
    let (_, player_position, player_orientation) = player.single();
    let (_, mut camera_position) = camera.single_mut();
    *camera_position = Transform::from_translation(player_position.0);
    camera_position.rotation = player_orientation.0
}

pub fn player_controls_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Position, &Orientation)>
) {
    let (_, mut position, orientation) = query.single_mut();
    let mut velocity = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::W) {
        velocity.z -= 0.4;
    }
    if keyboard_input.pressed(KeyCode::S) {
        velocity.z += 0.4;
    }
    if keyboard_input.pressed(KeyCode::D) {
        velocity.x += 0.4;
    }
    if keyboard_input.pressed(KeyCode::A) {
        velocity.x -= 0.4;
    }
    position.0 += orientation.0.mul_vec3(velocity);
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
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputState>()
            .add_startup_system(spawn_player.system())
            .add_startup_system(initial_grab_cursor.system())
            .add_system(player_controls_system.system())
            .add_system(player_look_system.system())
            .add_system(cursor_grab_system.system())
            .add_system(camera_follow_system.system());
    }
}
