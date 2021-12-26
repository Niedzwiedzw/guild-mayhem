pub mod player;
pub mod world;
pub mod constants;
pub mod debug_screen;

use bevy::prelude::*;
use heron::prelude::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(world::WorldPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .run();
}

