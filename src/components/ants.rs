use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    ANT_SIZE, COLLISION_GROUP_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALLS, WALL_Z_FACTOR,
};

use super::nav_mesh::NavNode;

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
}

#[derive(Clone, Copy, Component, Reflect)]
pub struct Ant {
    pub position_kind: AntPositionKind,
    pub speed: f32,
    pub direction: Vec3,
    pub current_wall: GlobalTransform,
}

impl AntBundle {
    pub fn new_on_nav_node(
        direction: Vec3,
        speed: f32,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder_pos: &GlobalTransform,
    ) -> Self {
        let mut transform = nav_node_pos.reparented_to(entities_holder_pos);
        transform.translation.z = 0.01; // FIXME
        let position_kind = match nav_node {
            NavNode::Background { .. } => AntPositionKind::Background,
            NavNode::VerticalEdge { is_left_side, .. } => AntPositionKind::VerticalWall {
                is_left_side: *is_left_side,
            },
            NavNode::HorizontalEdge { is_up_side, .. } => AntPositionKind::HorizontalWall {
                is_up_side: *is_up_side,
            },
        };
        let current_wall = *nav_node_pos;
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
                COLLISION_GROUP_PLAYER | COLLISION_GROUP_WALLS,
            ),
        }
    }
    pub fn spawn_on_nav_node<'c, 'w, 's>(
        commands: &'c mut Commands<'w, 's>,
        direction: Vec3,
        speed: f32,
        nav_node: &NavNode,
        nav_node_pos: &GlobalTransform,
        entities_holder: Entity,
        entities_holder_pos: &GlobalTransform,
    ) -> EntityCommands<'w, 's, 'c> {
        let mut command = commands.spawn(AntBundle::new_on_nav_node(
            direction,
            speed,
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
    nav_nodes: Query<(&NavNode, &GlobalTransform)>,
) {
    for (mut ant, colliding_entities, ant_transform_global, mut ant_transform) in ants.iter_mut() {
        // Detect walls and update [AntPosition]
        match ant.position_kind {
            AntPositionKind::Background => {
                // Find the closest wall that the ant is clipping with
                if let Some((nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes, None)
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
                            ant.current_wall = *wall_transform_global;
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
                            ant.current_wall = *wall_transform_global;
                        }
                    }
                }
            }
            AntPositionKind::HorizontalWall { is_up_side } => {
                // Check if the ant has gone into the background
                if ant_transform.translation.z <= 0.01 {
                    ant.position_kind = AntPositionKind::Background;
                    ant_transform.translation.z = 0.;
                    ant_transform.translation.y += if is_up_side { -0.01 } else { 0.01 };
                }
                // Check the closest colliding wall
                else if let Some((nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes, None)
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
                            ant.current_wall = *wall_transform_global;
                        }
                        // Otherwise update the transform of wall the ant is currently on
                        NavNode::HorizontalEdge { .. } => {
                            ant.current_wall = *wall_transform_global;
                        }
                    };
                }
                // If the ant is no longer colliding with any wall, it means that it went past an outward turn
                else if colliding_entities.is_empty() {
                    let new_wall_is_left_side = ant.direction.x > 0.;
                    let mut wall_transform_relative =
                        ant.current_wall.reparented_to(ant_transform_global);
                    // FIXME: hardcoded tile sizes
                    wall_transform_relative.translation.y +=
                        16. / 2. * if is_up_side { 1. } else { -1. };
                    wall_transform_relative.translation.x +=
                        16. / 2. * if new_wall_is_left_side { 1. } else { -1. };

                    place_ant_on_vertical_wall(
                        new_wall_is_left_side,
                        &mut ant,
                        &mut ant_transform,
                        &wall_transform_relative,
                    );
                    // FIXME: how to update ant.current_wall?
                }
            }
            AntPositionKind::VerticalWall { is_left_side } => {
                // Check if the ant has gone into the background
                if ant_transform.translation.z <= 0.01 {
                    ant.position_kind = AntPositionKind::Background;
                    ant_transform.translation.z = 0.;
                    ant_transform.translation.x += if is_left_side { 0.01 } else { -0.01 };
                }
                // Check the closest colliding wall
                else if let Some((nav_node, wall_transform_global)) =
                    find_closest_wall(ant_transform_global, colliding_entities, &nav_nodes, None)
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
                            ant.current_wall = *wall_transform_global;
                        }
                        // Otherwise update the transform of wall the ant is currently on
                        NavNode::VerticalEdge { .. } => {
                            ant.current_wall = *wall_transform_global;
                        }
                    };
                }
                // If the ant is no longer colliding with any wall, it means that it went past an outward turn
                else if colliding_entities.is_empty() {
                    let new_wall_is_up_side = ant.direction.y < 0.;
                    let mut wall_transform_relative =
                        ant.current_wall.reparented_to(ant_transform_global);
                    // FIXME: hardcoded tile sizes
                    wall_transform_relative.translation.x +=
                        16. / 2. * if is_left_side { -1. } else { 1. };
                    wall_transform_relative.translation.y +=
                        16. / 2. * if new_wall_is_up_side { -1. } else { 1. };

                    place_ant_on_horizontal_wall(
                        new_wall_is_up_side,
                        &mut ant,
                        &mut ant_transform,
                        &wall_transform_relative,
                    );
                    // FIXME: how to update ant.current_wall?
                }
            }
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
                let desired_direction = ant.direction.xy().normalize_or_zero() * ant.speed * dt;
                ant_transform.translation.x += desired_direction.x;
                ant_transform.translation.y += desired_direction.y;
            }
            AntPositionKind::VerticalWall { .. } => {
                // Vertical speed is the ant's full speed, while speed in Z axis is a projection of the desired direction on YZ
                let desired_direction_yz = ant.direction.yz().normalize_or_zero() * ant.speed * dt;
                let desired_direction_y = ant.direction.y.signum();
                ant_transform.translation.y += desired_direction_y;
                ant_transform.translation.z += desired_direction_yz[1];
                // FIXME tile size
                if ant_transform.translation.z > WALL_Z_FACTOR * 16. {
                    ant_transform.translation.z = WALL_Z_FACTOR * 16.;
                }
            }
            AntPositionKind::HorizontalWall { .. } => {
                // Horizontal speed is the ant's full speed, while speed in Z axis is a projection of the desired direction on XZ
                let desired_direction_xz = ant.direction.xz().normalize_or_zero() * ant.speed * dt;
                let desired_direction_x = ant.direction.x.signum();
                ant_transform.translation.x += desired_direction_x;
                ant_transform.translation.z += desired_direction_xz[1];
                // FIXME tile size
                if ant_transform.translation.z > WALL_Z_FACTOR * 16. {
                    ant_transform.translation.z = WALL_Z_FACTOR * 16.;
                }
            }
        }
    }
}

fn find_closest_wall<'a>(
    ant_transform_global: &GlobalTransform,
    colliding_entities: &CollidingEntities,
    nav_nodes: &'a Query<(&NavNode, &GlobalTransform)>,
    filter: Option<fn(&NavNode) -> bool>,
) -> Option<(&'a NavNode, &'a GlobalTransform)> {
    colliding_entities
        .iter()
        .filter_map(|entity| nav_nodes.get(entity).ok())
        .fold(
            None,
            |closest_node: Option<(f32, &NavNode, &GlobalTransform)>,
             (nav_node, wall_transform)| {
                if matches!(nav_node, NavNode::Background { .. }) {
                    unreachable!();
                }
                // Skip nodes that the filter rejects
                if let Some(filter) = filter {
                    if !filter(nav_node) {
                        return closest_node;
                    }
                }
                let distance = ant_transform_global
                    .translation()
                    .xy()
                    .distance(wall_transform.translation().xy());
                if let Some(closest_node) = closest_node {
                    if distance < closest_node.0 {
                        return Some((distance, nav_node, wall_transform));
                    }
                } else {
                    return Some((distance, nav_node, wall_transform));
                }
                closest_node
            },
        )
        .map(|(_, nav_node, wall_transform)| (nav_node, wall_transform))
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
    let offset = (ANT_SIZE.y / 2. - 0.1) * if is_up_side { -1. } else { 1. };
    ant_transform.translation.y += wall_transform_relative.translation.y + offset;
    if ant_transform.translation.z < 0.01 {
        ant_transform.translation.z = 0.1; // FIXME
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
    let offset = (ANT_SIZE.y / 2. - 0.1) * if is_left_side { 1. } else { -1. };
    ant_transform.translation.x += wall_transform_relative.translation.x + offset;
    if ant_transform.translation.z < 0.01 {
        ant_transform.translation.z = 0.1; // FIXME
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
            ant.current_wall.translation().xy(),
            Color::WHITE,
        );
    }
}
