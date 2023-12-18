use std::f32::consts::PI;

use crate::{
    components::{
        ants::{
            goal::AntGoal, live_ants::LiveAntBundle, movement::position::AntPositionKind,
            movement::AntMovement, AntColorKind, AntStyle,
        },
        nav_mesh::NavNode,
        pheromones::{concentrations::PheromoneConcentrations, PheromoneConfig, PheromoneKind},
    },
    render::render_ant::{AntMaterialBundle, ANT_MATERIAL_SIDE, ANT_MATERIAL_TOP, ANT_MESH2D},
    resources::nav_mesh_lut::NavMeshLUT,
    AppState, ANT_SIZE, ANT_WALL_CLIPPING, COLLISION_GROUP_ANTS, COLLISION_GROUP_EXPLOSION,
    COLLISION_GROUP_PLAYER_SENSOR, COLLISION_GROUP_WALLS, RENDERLAYER_ANTS, TILE_SIZE,
    WALL_Z_FACTOR,
};
use bevy::{ecs::system::EntityCommands, prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::LdtkEntity;
use bevy_rapier2d::prelude::*;
use rand::{rngs::ThreadRng, seq::IteratorRandom, thread_rng, Rng};

#[derive(Bundle)]
pub struct ZombAntBundle {
    pub zombant: ZombAnt,
    pub ant_movement: AntMovement,
    pub ant_style: AntStyle,
    pub material: AntMaterialBundle,
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub active_collisions: ActiveCollisionTypes,
    pub colliding_entities: CollidingEntities,
    pub collision_groups: CollisionGroups,
    pub render_layers: RenderLayers,
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct ZombAnt {}

impl ZombAntBundle {
    #[allow(clippy::too_many_arguments)]
    pub fn new_on_nav_node(
        direction: Vec3,
        speed: f32,
        scale: f32,
        color_primary_kind: AntColorKind,
        color_secondary_kind: AntColorKind,
        nav_node_entity: Entity,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder_pos: &GlobalTransform,
        rng: &mut ThreadRng,
        goal: AntGoal,
    ) -> Self {
        // FIXME: use common fn to place on walls?
        let mut transform = nav_node_pos.reparented_to(entities_holder_pos);
        let mut is_side = true;
        let position_kind = match nav_node {
            NavNode::Background { .. } => {
                transform.translation.z = 0.;
                is_side = false;
                AntPositionKind::Background
            }
            NavNode::VerticalEdge { is_left_side, .. } => {
                transform.translation.z = TILE_SIZE * WALL_Z_FACTOR;
                transform.translation.x +=
                    (ANT_SIZE.x / 2. - ANT_WALL_CLIPPING) * if *is_left_side { 1. } else { -1. };
                AntPositionKind::VerticalWall {
                    is_left_side: *is_left_side,
                }
            }
            NavNode::HorizontalEdge { is_up_side, .. } => {
                transform.translation.z = TILE_SIZE * WALL_Z_FACTOR;
                transform.translation.y +=
                    (ANT_SIZE.y / 2. - ANT_WALL_CLIPPING) * if *is_up_side { -1. } else { 1. };
                AntPositionKind::HorizontalWall {
                    is_up_side: *is_up_side,
                }
            }
        };
        let current_wall = (nav_node_entity, *nav_node_pos);
        let material = AntMaterialBundle {
            mesh: ANT_MESH2D,
            material: if is_side {
                ANT_MATERIAL_SIDE
            } else {
                ANT_MATERIAL_TOP
            },
            transform,
            ..default()
        };
        let color_primary = color_primary_kind.generate_color(rng);
        let color_secondary = color_secondary_kind.generate_color(rng);
        Self {
            zombant: ZombAnt {},
            ant_movement: AntMovement {
                position_kind,
                speed,
                direction,
                current_node: current_wall,
                goal,
                last_direction_update: 0.0,
            },
            ant_style: AntStyle {
                scale,
                color_primary,
                color_primary_kind,
                color_secondary,
                color_secondary_kind,
                animation_phase: rng.gen::<f32>() * 2. * PI,
            },
            material,
            collider: Collider::cuboid(ANT_SIZE.x / 2., ANT_SIZE.y / 2.),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collisions: ActiveCollisionTypes::STATIC_STATIC,
            colliding_entities: Default::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_ANTS,
                COLLISION_GROUP_PLAYER_SENSOR | COLLISION_GROUP_WALLS | COLLISION_GROUP_EXPLOSION,
            ),
            render_layers: RENDERLAYER_ANTS,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn_on_nav_node<'c, 'w, 's>(
        commands: &'c mut Commands<'w, 's>,
        direction: Vec3,
        speed: f32,
        scale: f32,
        color_primary_kind: AntColorKind,
        color_secondary_kind: AntColorKind,
        nav_node_entity: Entity,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder: Entity,
        entities_holder_pos: &GlobalTransform,
        rng: &mut ThreadRng,
        goal: AntGoal,
    ) -> EntityCommands<'w, 's, 'c> {
        let mut command = commands.spawn(ZombAntBundle::new_on_nav_node(
            direction,
            speed,
            scale,
            color_primary_kind,
            color_secondary_kind,
            nav_node_entity,
            nav_node,
            nav_node_pos,
            entities_holder_pos,
            rng,
            goal,
        ));
        command.set_parent(entities_holder);
        command
    }
}

#[derive(Bundle)]
pub struct ZombAntQueenBundle {
    pub zombant_queen: ZombAntQueen,
    pub ant_movement: AntMovement,
    pub ant_style: AntStyle,
    pub material: AntMaterialBundle,
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub active_collisions: ActiveCollisionTypes,
    pub colliding_entities: CollidingEntities,
    pub collision_groups: CollisionGroups,
    pub render_layers: RenderLayers,
}

#[derive(Default, Debug, Clone, Copy, Component, Reflect)]
pub struct ZombAntQueen {
    pub holds: f32,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, LdtkEntity)]
pub struct ZombAntQueenSpawnPoint {}

pub fn spawn_zombant_queen(
    mut commands: Commands,
    spawn_points: Query<(&Transform, &Parent), With<ZombAntQueenSpawnPoint>>,
    nav_nodes: Query<(&NavNode, &GlobalTransform)>,
    global_transforms: Query<&GlobalTransform>,
    nav_mesh_lut: Res<NavMeshLUT>,
) {
    let mut rng = thread_rng();
    let Some((spawn_point_pos, entities_holder)) = spawn_points.iter().choose(&mut rng) else {
        error!("There are no spawn points for the zombant queen on the map");
        return;
    };

    let nav_node_entity = nav_mesh_lut
        .get_tile_entity(spawn_point_pos.translation.xy())
        .unwrap()
        .0;
    let (nav_node, nav_node_pos) = nav_nodes.get(nav_node_entity).unwrap();
    let entities_holder_pos = global_transforms.get(entities_holder.get()).unwrap();
    let direction = Vec3::new(
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5,
    )
    .normalize();
    let color_primary_kind = AntColorKind::new_random(&mut rng);
    let color_secondary_kind = AntColorKind::new_random_from_primary(&mut rng, &color_primary_kind);
    let speed = 40.;
    let scale = 1.; // TODO
    let ant_bundle = LiveAntBundle::new_on_nav_node(
        direction,
        speed,
        scale,
        color_primary_kind,
        color_secondary_kind,
        nav_node_entity,
        nav_node,
        nav_node_pos,
        entities_holder_pos,
        &mut rng,
        AntGoal::default(),
    );
    commands
        .spawn(ZombAntQueenBundle {
            zombant_queen: ZombAntQueen::default(),
            ant_movement: ant_bundle.ant_movement,
            ant_style: ant_bundle.ant_style,
            material: ant_bundle.material,
            collider: ant_bundle.collider,
            sensor: Sensor,
            active_events: ant_bundle.active_events,
            active_collisions: ant_bundle.active_collisions,
            colliding_entities: ant_bundle.colliding_entities,
            collision_groups: ant_bundle.collision_groups,
            render_layers: ant_bundle.render_layers,
        })
        .set_parent(entities_holder.get());
}

pub fn update_zombants_deposit(
    zombants: Query<&AntMovement, With<ZombAnt>>,
    mut nodes: Query<&mut PheromoneConcentrations>,
    phcfg: Res<PheromoneConfig>,
) {
    for ant_movement in zombants.iter() {
        let mut pheromones = nodes.get_mut(ant_movement.current_node.0).unwrap();
        pheromones.concentrations[PheromoneKind::Zombant as usize] += phcfg.zombant_deposit;
    }
}

pub fn update_zombqueen_source(
    queen: Query<&AntMovement, With<ZombAntQueen>>,
    mut nodes: Query<&mut PheromoneConcentrations>,
    phcfg: Res<PheromoneConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Ok(queen_movement) = queen.get_single() {
        let mut pheromones = nodes.get_mut(queen_movement.current_node.0).unwrap();
        pheromones.concentrations[PheromoneKind::Zombqueen as usize] += phcfg.zombqueen_source;
    } else {
        next_state.set(AppState::Win);
    }
}
