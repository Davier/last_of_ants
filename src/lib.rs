pub mod components;
pub mod helpers;
pub mod render;
pub mod resources;
pub mod ui;

use bevy::{asset::AssetMetaCheck, prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::{prelude::*, systems::process_ldtk_levels};
use bevy_rapier2d::prelude::*;

use components::{
    ants::*,
    clues::place_clues,
    cocoons::CocoonBundle,
    nav_mesh::*,
    pheromons::{init_pheromons, PheromonSourceBundle},
    player::*,
    tiles::*,
    zombants::ZombAntQueenSpawnPoint,
};
use helpers::pause_if_not_focused;
use render::{render_ant::AntMaterialPlugin, render_cocoon::CocoonMaterialPlugin};
use resources::{
    clues::{clues_receive_events, ClueEvent, Clues},
    nav_mesh_lut::NavMeshLUT,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_entity::<PheromonSourceBundle>("Source")
            .register_ldtk_entity::<CocoonBundle>("Shedding")
            // .register_ldtk_entity::<AntBundle>("Ant")
            .register_ldtk_entity::<ZombAntQueenSpawnPoint>("Zombant_Queen_Spawn_Point")
            .register_ldtk_int_cell::<TileGroundBundle>(TILE_INT_GROUND)
            .register_ldtk_int_cell::<TileEmptyUndergroundBundle>(TILE_INT_EMPTY)
            .register_type::<components::nav_mesh::NavNode>()
            .register_type::<Clues>()
            .insert_resource(AssetMetaCheck::Never)
            .init_resource::<NavMeshLUT>()
            .add_event::<ClueEvent>()
            .add_plugins((
                DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites? (TODO: test)
                LdtkPlugin,
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
                AntMaterialPlugin,
                CocoonMaterialPlugin,
            ))
            .add_systems(
                PreUpdate,
                (
                    spawn_nav_mesh,
                    init_pheromons.after(spawn_nav_mesh),
                    place_clues,
                    pause_if_not_focused,
                ),
            )
            .add_systems(
                Update,
                (
                    update_player_sensor,
                    spawn_player_sensor,
                    clues_receive_events,
                    (
                        update_ant_position_kinds,
                        assert_ants, // TODO: disable in release?
                        update_ant_direction,
                        // update_ant_direction_randomly,
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
/// Number of pixels the ants should be clipping in a wall when moving on it
pub const ANT_WALL_CLIPPING: f32 = 0.5;

pub const TILE_INT_GROUND: i32 = 1;
pub const TILE_INT_EMPTY: i32 = 2;
pub const TILE_SIZE: f32 = 16.;

pub const COCOON_ROOMS: &[u8] = &[0, 1, 2];
/// Should be less than ROOMS.len()
pub const CLUES_NUMBER: usize = 2;

pub const COLLISION_GROUP_WALLS: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_SENSOR: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ANTS: Group = Group::GROUP_4;
pub const COLLISION_GROUP_DEAD_ANTS: Group = Group::GROUP_4;
pub const COLLISION_GROUP_CLUE: Group = Group::GROUP_5;

pub const RENDERLAYER_ANTS: RenderLayers = RenderLayers::layer(1);
pub const RENDERLAYER_PLAYER: RenderLayers = RenderLayers::layer(2);
pub const RENDERLAYER_CLUE_ANT: RenderLayers = RenderLayers::layer(3);

pub const CLUE_COLOR: Color = Color::rgb_linear(1., 0.6, 0.);
