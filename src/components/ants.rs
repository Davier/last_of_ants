use bevy::{ecs::system::EntityCommands, prelude::*, render::view::{RenderLayers, NoFrustumCulling}};
use bevy_rapier2d::prelude::*;

use crate::{
    ANT_SIZE, COLLISION_GROUP_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PLAYER_SENSOR,
    COLLISION_GROUP_WALLS, TILE_SIZE, WALL_Z_FACTOR, RENDERLAYER_ANTS,
};

use super::nav_mesh::{NavMeshLUT, NavNode};

#[derive(Bundle)]
pub struct AntBundle {
    pub ant: Ant,
    pub sprite: SpriteBundle,
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub active_collisions: ActiveCollisionTypes,
    pub colliding_entities: CollidingEntities,
    pub collision_groups: CollisionGroups,
    pub render_layers: RenderLayers,
}

#[derive(Clone, Copy, Component, Reflect)]
pub struct Ant {
    pub position_kind: AntPositionKind,
    pub speed: f32,
    pub direction: Vec3,
    pub current_wall: (Entity, GlobalTransform), // FIXME: use relative transforms
}

impl AntBundle {
    pub fn new_on_nav_node(
        direction: Vec3,
        speed: f32,
        nav_node_entity: Entity,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder_pos: &GlobalTransform,
    ) -> Self {
        // FIXME: use common fn to place on walls?
        let mut transform = nav_node_pos.reparented_to(entities_holder_pos);
        let position_kind = match nav_node {
            NavNode::Background { .. } => {
                transform.translation.z = 0.;
                AntPositionKind::Background
            }
            NavNode::VerticalEdge { is_left_side, .. } => {
                transform.translation.z = TILE_SIZE * WALL_Z_FACTOR;
                transform.translation.x +=
                    (ANT_SIZE.x / 2. - 0.01) * if *is_left_side { 1. } else { -1. };
                AntPositionKind::VerticalWall {
                    is_left_side: *is_left_side,
                }
            }
            NavNode::HorizontalEdge { is_up_side, .. } => {
                transform.translation.z = TILE_SIZE * WALL_Z_FACTOR;
                transform.translation.y +=
                    (ANT_SIZE.y / 2. - 0.01) * if *is_up_side { -1. } else { 1. };
                AntPositionKind::HorizontalWall {
                    is_up_side: *is_up_side,
                }
            }
        };
        let current_wall = (nav_node_entity, *nav_node_pos);
        Self {
            ant: Ant {
                position_kind,
                speed,
                direction,
                current_wall,
            },
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(ANT_SIZE),
                    ..default()
                },
                transform,
                ..default()
            },
            collider: Collider::cuboid(ANT_SIZE.x / 2., ANT_SIZE.y / 2.),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collisions: ActiveCollisionTypes::STATIC_STATIC,
            colliding_entities: Default::default(),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_ANTS,
                COLLISION_GROUP_PLAYER_SENSOR | COLLISION_GROUP_WALLS,
            ),
            render_layers: RENDERLAYER_ANTS,
        }
    }
    pub fn spawn_on_nav_node<'c, 'w, 's>(
        commands: &'c mut Commands<'w, 's>,
        direction: Vec3,
        speed: f32,
        nav_node_entity: Entity,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder: Entity,
        entities_holder_pos: &GlobalTransform,
    ) -> EntityCommands<'w, 's, 'c> {
        let mut command = commands.spawn(AntBundle::new_on_nav_node(
            direction,
            speed,
            nav_node_entity,
            nav_node,
            nav_node_pos,
            entities_holder_pos,
        ));
        command.set_parent(entities_holder);
        command
    }
}

#[derive(Clone, Copy, Reflect)]
pub enum AntPositionKind {
    Background,
    VerticalWall { is_left_side: bool },
    HorizontalWall { is_up_side: bool },
}

pub fn update_ant_position_kinds(
    mut ants: Query<(
        &mut Ant,
        &CollidingEntities,
        &GlobalTransform,
        &mut Transform,
    )>,
    nav_nodes: Query<(Entity, &NavNode, &GlobalTransform)>,
    nav_mesh_lut: Res<NavMeshLUT>,
) {
    for (mut ant, colliding_entities, ant_transform_global, mut ant_transform) in ants.iter_mut() {
        // Detect walls and update [AntPosition]
        match ant.position_kind {
            AntPositionKind::Background => {
                // Find the closest wall that the ant is clipping with
                if let Some((nav_node_entity, nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes)
                {
                    match nav_node {
                        NavNode::Background { .. } => unreachable!(),
                        NavNode::HorizontalEdge { is_up_side, .. } => {
                            let wall_transform_relative =
                                wall_transform_global.reparented_to(ant_transform_global);
                            place_ant_on_horizontal_wall(
                                *is_up_side,
                                &mut ant,
                                &mut ant_transform,
                                &wall_transform_relative,
                            );
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                        NavNode::VerticalEdge { is_left_side, .. } => {
                            let wall_transform_relative =
                                wall_transform_global.reparented_to(ant_transform_global);
                            place_ant_on_vertical_wall(
                                *is_left_side,
                                &mut ant,
                                &mut ant_transform,
                                &wall_transform_relative,
                            );
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                    }
                }
            }
            AntPositionKind::HorizontalWall { is_up_side } => {
                // Check if the ant has gone into the background
                if ant_transform.translation.z <= 0.01 {
                    ant.position_kind = AntPositionKind::Background;
                    ant_transform.translation.z = 0.;
                    ant_transform.translation.y += if is_up_side { -0.02 } else { 0.02 };
                }
                // Check the closest colliding wall
                else if let Some((nav_node_entity, nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes)
                {
                    match nav_node {
                        NavNode::Background { .. } => unreachable!(),
                        // If it's a vertical wall, then we place the ant on it
                        NavNode::VerticalEdge { is_left_side, .. } => {
                            let wall_transform_relative =
                                wall_transform_global.reparented_to(ant_transform_global);
                            place_ant_on_vertical_wall(
                                *is_left_side,
                                &mut ant,
                                &mut ant_transform,
                                &wall_transform_relative,
                            );
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                        // Otherwise update the transform of wall the ant is currently on
                        NavNode::HorizontalEdge { .. } => {
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                    };
                }
                // If the ant is no longer colliding with any wall, it means that it went past an outward turn
                else if colliding_entities.is_empty() {
                    let new_wall_is_left_side = ant.direction.x > 0.;

                    let current_wall = nav_nodes.get(ant.current_wall.0).unwrap();
                    let (wall_entity, _wall_node, wall_transform_global) = {
                        let NavNode::HorizontalEdge { left, right, .. } = current_wall.1 else {
                            dbg!(current_wall);
                            panic!();
                        };
                        let neighbor = if new_wall_is_left_side { right } else { left };
                        nav_nodes.get(*neighbor).unwrap()
                    };
                    let wall_transform_relative =
                        wall_transform_global.reparented_to(ant_transform_global);
                    place_ant_on_vertical_wall(
                        new_wall_is_left_side,
                        &mut ant,
                        &mut ant_transform,
                        &wall_transform_relative,
                    );
                    ant.current_wall = (wall_entity, *wall_transform_global);
                }
            }
            AntPositionKind::VerticalWall { is_left_side } => {
                // Check if the ant has gone into the background
                if ant_transform.translation.z <= 0.01 {
                    ant.position_kind = AntPositionKind::Background;
                    ant_transform.translation.z = 0.;
                    ant_transform.translation.x += if is_left_side { 0.02 } else { -0.02 };
                }
                // Check the closest colliding wall
                else if let Some((nav_node_entity, nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes)
                {
                    match nav_node {
                        NavNode::Background { .. } => unreachable!(),
                        // If it's a horizontal wall, then we place the ant on it
                        NavNode::HorizontalEdge { is_up_side, .. } => {
                            let wall_transform_relative =
                                wall_transform_global.reparented_to(ant_transform_global);
                            place_ant_on_horizontal_wall(
                                *is_up_side,
                                &mut ant,
                                &mut ant_transform,
                                &wall_transform_relative,
                            );
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                        // Otherwise update the transform of wall the ant is currently on
                        NavNode::VerticalEdge { .. } => {
                            ant.current_wall = (nav_node_entity, *wall_transform_global);
                        }
                    };
                }
                // If the ant is no longer colliding with any wall, it means that it went past an outward turn
                else if colliding_entities.is_empty() {
                    let new_wall_is_up_side = ant.direction.y < 0.;

                    let current_wall = nav_nodes.get(ant.current_wall.0).unwrap();
                    let (wall_entity, _wall_node, wall_transform_global) = {
                        let NavNode::VerticalEdge { up, down, .. } = current_wall.1 else {
                            dbg!(current_wall);
                            panic!(); // FIXME: triggers if spawning ants too early?
                        };
                        let neighbor = if new_wall_is_up_side { down } else { up };
                        nav_nodes.get(*neighbor).unwrap()
                    };
                    let wall_transform_relative =
                        wall_transform_global.reparented_to(ant_transform_global);
                    place_ant_on_horizontal_wall(
                        new_wall_is_up_side,
                        &mut ant,
                        &mut ant_transform,
                        &wall_transform_relative,
                    );
                    ant.current_wall = (wall_entity, *wall_transform_global);
                }
            }
        }
        // Update the current tile if in the background
        if matches!(ant.position_kind, AntPositionKind::Background) {
            let Some((background_entity, _)) =
                nav_mesh_lut.get_tile_entity(ant_transform.translation.xy())
            else {
                continue;
            };
            let background_entity_transform = *nav_nodes
                .get_component::<GlobalTransform>(background_entity)
                .unwrap();
            ant.current_wall = (background_entity, background_entity_transform);
        }
    }
}

/// Calculate desired direction of ants according to the navigation mesh
pub fn update_ant_direction() {}

// Move ants according to their direction and the constraints of [AntPositionKind]
pub fn update_ant_position(mut ants: Query<(&Ant, &mut Transform)>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for (ant, mut ant_transform) in ants.iter_mut() {
        match ant.position_kind {
            AntPositionKind::Background => {
                let delta_xy = ant.direction.xy().normalize_or_zero() * ant.speed * dt;
                ant_transform.translation.x += delta_xy.x;
                ant_transform.translation.y += delta_xy.y;
            }
            AntPositionKind::VerticalWall { .. } => {
                // Vertical speed is the ant's full speed, while speed in Z axis is a projection of the desired direction on YZ
                let delta_yz = ant.direction.yz().normalize_or_zero() * ant.speed * dt;
                let delta_y = ant.direction.y.signum() * ant.speed * dt;
                ant_transform.translation.y += delta_y;
                ant_transform.translation.z += delta_yz[1];
                if ant_transform.translation.z > WALL_Z_FACTOR * TILE_SIZE {
                    ant_transform.translation.z = WALL_Z_FACTOR * TILE_SIZE;
                }
            }
            AntPositionKind::HorizontalWall { .. } => {
                // Horizontal speed is the ant's full speed, while speed in Z axis is a projection of the desired direction on XZ
                let delta_y = ant.direction.xz().normalize_or_zero() * ant.speed * dt;
                let delta_x = ant.direction.x.signum() * ant.speed * dt;
                ant_transform.translation.x += delta_x;
                ant_transform.translation.z += delta_y[1];
                if ant_transform.translation.z > WALL_Z_FACTOR * TILE_SIZE {
                    ant_transform.translation.z = WALL_Z_FACTOR * TILE_SIZE;
                }
            }
        }
    }
}

fn find_closest_wall<'a>(
    ant_transform_global: &GlobalTransform,
    colliding_entities: &CollidingEntities,
    nav_nodes: &'a Query<(Entity, &NavNode, &GlobalTransform)>,
) -> Option<(Entity, &'a NavNode, &'a GlobalTransform)> {
    colliding_entities
        .iter()
        .filter_map(|entity| nav_nodes.get(entity).ok())
        .fold(
            None,
            |closest_node: Option<(f32, Entity, &NavNode, &GlobalTransform)>,
             (nav_node_entity, nav_node, wall_transform)| {
                if matches!(nav_node, NavNode::Background { .. }) {
                    unreachable!();
                }
                let distance = ant_transform_global
                    .translation()
                    .xy()
                    .distance(wall_transform.translation().xy());
                if let Some(closest_node) = closest_node {
                    if distance < closest_node.0 {
                        return Some((distance, nav_node_entity, nav_node, wall_transform));
                    }
                } else {
                    return Some((distance, nav_node_entity, nav_node, wall_transform));
                }
                closest_node
            },
        )
        .map(|(_, nav_node_entity, nav_node, wall_transform)| {
            (nav_node_entity, nav_node, wall_transform)
        })
}

fn place_ant_on_horizontal_wall(
    is_up_side: bool,
    ant: &mut Ant,
    ant_transform: &mut Transform,
    wall_transform_relative: &Transform,
) {
    // Change position kind
    ant.position_kind = AntPositionKind::HorizontalWall { is_up_side };
    // Re-place ant on the wall
    let offset = (ANT_SIZE.y / 2. - 0.01) * if is_up_side { -1. } else { 1. };
    ant_transform.translation.y += wall_transform_relative.translation.y + offset;
    if ant_transform.translation.z < 0.01 {
        ant_transform.translation.z = 0.01;
    }
}

fn place_ant_on_vertical_wall(
    is_left_side: bool,
    ant: &mut Ant,
    ant_transform: &mut Transform,
    wall_transform_relative: &Transform,
) {
    // Change position kind
    ant.position_kind = AntPositionKind::VerticalWall { is_left_side };
    // Re-place ant on the wall
    let offset = (ANT_SIZE.x / 2. - 0.01) * if is_left_side { 1. } else { -1. };
    ant_transform.translation.x += wall_transform_relative.translation.x + offset;
    if ant_transform.translation.z < 0.01 {
        ant_transform.translation.z = 0.01;
    }
}

pub fn debug_ants(ants: Query<(&Ant, &GlobalTransform)>, mut gizmos: Gizmos) {
    for (ant, pos) in ants.iter() {
        let color = match ant.position_kind {
            AntPositionKind::Background => Color::BLUE,
            AntPositionKind::VerticalWall { is_left_side: true } => Color::RED,
            AntPositionKind::VerticalWall {
                is_left_side: false,
            } => Color::ORANGE_RED,
            AntPositionKind::HorizontalWall { is_up_side: true } => Color::GREEN,
            AntPositionKind::HorizontalWall { is_up_side: false } => Color::YELLOW_GREEN,
        };
        gizmos.circle_2d(pos.translation().xy(), 4., color);
        gizmos.line_2d(
            pos.translation().xy(),
            ant.current_wall.1.translation().xy(),
            Color::WHITE,
        );
    }
}
