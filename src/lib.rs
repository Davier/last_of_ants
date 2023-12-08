pub mod components;
pub mod helpers;
pub mod render;

use bevy::{asset::AssetMetaCheck, prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::{prelude::*, systems::process_ldtk_levels};
use bevy_rapier2d::prelude::*;

use components::{ants::*, nav_mesh::*, player::*, tiles::*};
use helpers::pause_if_not_focused;
use render::render_ant::AntMaterialPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            // .register_ldtk_entity::<AntBundle>("Ant")
            .register_ldtk_int_cell::<TileGroundBundle>(TILE_INT_GROUND)
            .register_ldtk_int_cell::<TileEmptyUndergroundBundle>(TILE_INT_EMPTY)
            .register_type::<components::nav_mesh::NavNode>()
            .insert_resource(AssetMetaCheck::Never)
            .init_resource::<NavMeshLUT>()
            .add_plugins((
                DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites? (TODO: test)
                LdtkPlugin,
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
                AntMaterialPlugin,
            ))
            .add_systems(
                PreUpdate,
                (
                    spawn_nav_mesh.after(process_ldtk_levels),
                    pause_if_not_focused,
                ),
            )
            .add_systems(
                Update,
                (
                    update_player_sensor,
                    spawn_player_sensor,
                    (
                        update_ant_position_kinds,
                        assert_ants, // TODO: disable in release?
                        // update_ant_direction,
                        update_ant_direction_randomly,
                        update_ant_position,
                    )
                        .chain(),
                ),
            );
    }
}

pub const PIXELS_PER_METER: f32 = 16.;
pub const PLAYER_SIZE: Vec2 = Vec2::new(8., 16.);
pub const ANT_SIZE: Vec2 = Vec2::new(16., 16.);
/// Vertical and horizontal edges will have their [NavNode] placed at `tile_size * WALL_Z_FACTOR / 2.` in Z
pub const WALL_Z_FACTOR: f32 = 1.;

pub const TILE_INT_GROUND: i32 = 1;
pub const TILE_INT_EMPTY: i32 = 2;
pub const TILE_SIZE: f32 = 16.;

pub const COLLISION_GROUP_WALLS: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_SENSOR: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ANTS: Group = Group::GROUP_4;
pub const COLLISION_GROUP_DEAD_ANTS: Group = Group::GROUP_4;

pub const RENDERLAYER_ANTS: RenderLayers = RenderLayers::layer(1);
pub const RENDERLAYER_PLAYER: RenderLayers = RenderLayers::layer(2);
