pub mod components;
pub mod helpers;
pub mod render;
pub mod resources;
pub mod ui;

use bevy::{asset::AssetMetaCheck, prelude::*, render::view::RenderLayers};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::prelude::*;

use components::{
    ants::{
        goal::{update_ant_goal, update_metrics, Metrics},
        movement::update_ant_direction,
        position::{update_ant_position, update_ant_position_kinds},
        zombants::{
            spawn_zombant_queen, update_zombants_deposit, update_zombqueen_source,
            ZombAntQueenSpawnPoint,
        },
        *,
    },
    clues::place_clues,
    cocoons::CocoonBundle,
    nav_mesh::*,
    object::ObjectBundle,
    pheromones::{init_pheromons, init_sources, PheromoneConfig},
    player::*,
    tiles::*,
};
use helpers::pause_if_not_focused;
use render::{
    player_animation::PlayerAnimationPlugin, render_ant::AntMaterialPlugin,
    render_cocoon::CocoonMaterialPlugin,
};
use resources::{
    clues::{clues_receive_events, ClueEvent, Clues},
    nav_mesh_lut::NavMeshLUT,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_entity::<CocoonBundle>("Shedding")
            .register_ldtk_entity::<ObjectBundle>("Source")
            // .register_ldtk_entity::<AntBundle>("Ant")
            .register_ldtk_entity::<ZombAntQueenSpawnPoint>("Zombant_Queen_Spawn_Point")
            .register_ldtk_int_cell::<TileGroundBundle>(TILE_INT_GROUND)
            .register_ldtk_int_cell::<TileEmptyUndergroundBundle>(TILE_INT_EMPTY)
            .register_ldtk_int_cell::<TileEmptyOvergroundBundle>(TILE_INT_OVERGROUND)
            .register_type::<components::nav_mesh::NavNode>()
            .register_type::<Clues>()
            .insert_resource(AssetMetaCheck::Never)
            .init_resource::<NavMeshLUT>()
            .add_event::<ClueEvent>()
            .init_resource::<PheromoneConfig>()
            .init_resource::<Metrics>()
            .add_plugins((
                DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites? (TODO: test)
                LdtkPlugin,
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
                AntMaterialPlugin,
                CocoonMaterialPlugin,
                PlayerAnimationPlugin,
            ))
            .add_plugins((
                ResourceInspectorPlugin::<PheromoneConfig>::default(),
                ResourceInspectorPlugin::<Metrics>::default(),
            ))
            .add_state::<AppState>()
            .add_loading_state(
                LoadingState::new(AppState::Loading)
                    .continue_to_state(AppState::ProcessingNavNodes),
            )
            .add_collection_to_loading_state::<_, AllAssets>(AppState::Loading)
            .insert_resource(LevelSelection::index(0))
            .add_systems(OnExit(AppState::Loading), spawn_ldtk_level)
            .add_systems(
                Update,
                (spawn_nav_mesh).run_if(in_state(AppState::ProcessingNavNodes)),
            )
            .add_systems(
                OnEnter(AppState::ProcessingOthers),
                (
                    // One-shot systems that need nav nodes
                    spawn_player_sensor,
                    spawn_zombant_queen,
                    place_clues,
                    (init_pheromons, apply_deferred, init_sources).chain(),
                ),
            )
            .add_systems(
                Update,
                start_playing.run_if(in_state(AppState::ProcessingOthers)),
            )
            .add_systems(
                Update,
                (
                    pause_if_not_focused,
                    update_player_sensor,
                    clues_receive_events,
                    ant_explosion_collision,
                    (
                        update_ant_position_kinds,
                        // assert_ants, // TODO: disable in release?
                        update_ant_direction,
                        // update_ant_direction_randomly,
                        update_ant_position,
                        update_zombants_deposit,
                        update_zombqueen_source,
                        update_ant_goal,
                        update_metrics,
                    )
                        .chain(),
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    ProcessingNavNodes,
    ProcessingOthers,
    Playing,
}

#[derive(AssetCollection, Resource)]
pub struct AllAssets {
    #[asset(path = "textures/explosion_spritesheet.png")]
    pub explosion: Handle<Image>,
    #[asset(path = "textures/owlet_spritesheet.png")]
    pub player: Handle<Image>,
    #[asset(path = "Tiles_64x64.png")]
    pub tilemap: Handle<Image>,
    #[asset(path = "Ant nest.ldtk")]
    pub map: Handle<LdtkProject>,
}

pub fn spawn_ldtk_level(mut commands: Commands, assets: Res<AllAssets>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.map.clone(),
        ..Default::default()
    });
}

pub fn start_playing(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
}

pub const PIXELS_PER_METER: f32 = 16.;
pub const PLAYER_SIZE: Vec2 = Vec2::new(32., 32.);
pub const ANT_SIZE: Vec2 = Vec2::new(16., 16.);
/// Vertical and horizontal edges will have their [NavNode] placed at `tile_size * WALL_Z_FACTOR / 2.` in Z
pub const WALL_Z_FACTOR: f32 = 1.;
/// Number of pixels the ants should be clipping in a wall when moving on it
pub const ANT_WALL_CLIPPING: f32 = 0.5;

pub const TILE_INT_GROUND: i32 = 1;
pub const TILE_INT_EMPTY: i32 = 2;
pub const TILE_INT_OVERGROUND: i32 = 3;
pub const TILE_SIZE: f32 = 16.;

pub const COCOON_ROOMS: &[u8] = &[0, 1, 2];
/// Should be less than ROOMS.len()
pub const CLUES_NUMBER: usize = 2;

pub const COLLISION_GROUP_WALLS: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_SENSOR: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ANTS: Group = Group::GROUP_4;
pub const COLLISION_GROUP_DEAD_ANTS: Group = Group::GROUP_5;
pub const COLLISION_GROUP_CLUE: Group = Group::GROUP_6;
pub const COLLISION_GROUP_EXPLOSION: Group = Group::GROUP_7;

pub const RENDERLAYER_ANTS: RenderLayers = RenderLayers::layer(1);
pub const RENDERLAYER_PLAYER: RenderLayers = RenderLayers::layer(2);
pub const RENDERLAYER_CLUE_ANT: RenderLayers = RenderLayers::layer(3);

pub const CLUE_COLOR: Color = Color::rgb_linear(1., 0.6, 0.);
