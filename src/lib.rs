pub mod components;
pub mod helpers;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_ldtk::{prelude::*, systems::process_ldtk_levels};
use bevy_rapier2d::prelude::*;

use components::{entities::*, nav_mesh::*, tiles::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            // .register_ldtk_entity::<AntBundle>("Ant")
            .register_ldtk_int_cell::<TileGround>(1)
            .register_ldtk_int_cell::<TileEmptyUnderground>(2)
            .register_type::<components::nav_mesh::NavNode>()
            .insert_resource(AssetMetaCheck::Never)
            .add_plugins((
                DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites? (TODO: test)
                LdtkPlugin,
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
            ))
            .add_systems(PreUpdate, spawn_nav_mesh.after(process_ldtk_levels))
            .add_systems(Update, (update_player_sensor, spawn_player_sensor));
    }
}

pub const PLAYER_SIZE: Vec2 = Vec2::new(8., 16.);
pub const PIXELS_PER_METER: f32 = 16.;
pub const ANT_SIZE: Vec2 = Vec2::new(8., 8.);