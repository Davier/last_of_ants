use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{PLAYER_SIZE, ANT_SIZE};

use super::nav_mesh::NavNode;

#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,
    pub sprite: SpriteBundle,
    pub controller: KinematicCharacterController,
    pub collider: Collider,
    // The velocity is only used by us since it's not a RigidBody
    pub velocity: Velocity,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(PLAYER_SIZE),
                    ..default()
                },
                ..default()
            },
            collider: Collider::cuboid(PLAYER_SIZE.x / 2., PLAYER_SIZE.y / 2.),
            player: Default::default(),
            controller: KinematicCharacterController {
                min_slope_slide_angle: PI / 5.,
                filter_groups: Some(CollisionGroups::new(
                    COLLISION_GROUP_PLAYER,
                    COLLISION_GROUP_WALLS,
                )),
                ..Default::default()
            },
            velocity: Default::default(),
        }
    }
}

pub fn update_player_sensor(
    player_sensors: Query<(&PlayerWallSensor, &CollidingEntities)>,
    mut players: Query<&mut Player>,
    nav_nodes: Query<&NavNode>,
) {
    for (player, colliding_entities) in player_sensors.iter() {
        let Ok(mut player) = players.get_mut(player.player) else {
            warn!("Unattached player sensor");
            continue;
        };
        player.is_on_left_wall = false;
        player.is_on_right_wall = false;
        // player.is_on_ceiling = false;
        player.on_ground.clear();
        player.on_wall.clear();
        for colliding_entity in colliding_entities.iter() {
            let Ok(nav_node) = nav_nodes.get(colliding_entity) else {
                continue;
            };
            match nav_node {
                NavNode::VerticalEdge { is_left_side, .. } => {
                    player.on_wall.insert(colliding_entity);
                    if *is_left_side {
                        player.is_on_left_wall = true;
                    } else {
                        player.is_on_right_wall = true;
                    }
                }
                NavNode::HorizontalEdge {
                    is_up_side: false, ..
                } => {
                    player.on_ground.insert(colliding_entity);
                }
                NavNode::HorizontalEdge {
                    is_up_side: true, ..
                } => {
                    // The sensor is now only at the player's feets, collisions
                    // with the ceiling are detected with the character controller
                    // player.is_on_ceiling = true;
                }
                _ => (),
            }
        }
    }
}

#[derive(Clone, Default, Component, Reflect)]
pub struct Player {
    pub on_wall: HashSet<Entity>,
    pub on_ground: HashSet<Entity>,
    // pub is_grounded: bool,
    pub is_on_left_wall: bool,
    pub is_on_right_wall: bool,
    // pub is_on_ceiling: bool,
}

#[derive(Debug, Copy, Clone, Reflect, Component)]
pub struct PlayerWallSensor {
    player: Entity,
}

pub fn spawn_player_sensor(
    mut commands: Commands,
    added_players: Query<(Entity, &Collider), Added<Player>>,
) {
    for (entity, collider) in added_players.iter() {
        let size = collider.as_cuboid().unwrap().raw.half_extents;
        let offset = 0.5;
        commands
            .spawn((
                PlayerWallSensor { player: entity },
                // The sensor is smaller and at the feets of the player, otherwise it detects
                // vertical walls that are only on top of the player
                Collider::cuboid(size[0] + offset, size[1] / 2. + offset),
                Sensor,
                TransformBundle::from_transform(Transform::from_xyz(0., -size[1] / 2., 0.)),
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::STATIC_STATIC,
                CollidingEntities::default(),
                CollisionGroups::new(COLLISION_GROUP_PLAYER_SENSOR, COLLISION_GROUP_WALLS),
            ))
            .set_parent(entity);
    }
}

#[derive(Bundle, Clone, LdtkEntity)]
pub struct AntBundle {
    pub ant: Ant,
    pub sprite: SpriteBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

impl Default for AntBundle {
    fn default() -> Self {
        Self {
            ant: Default::default(),
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(ANT_SIZE),
                    ..default()
                },
                ..default()
            },
            collider: Collider::cuboid(ANT_SIZE.x / 2., ANT_SIZE.y / 2.),
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_ANTS,
                COLLISION_GROUP_PLAYER | COLLISION_GROUP_WALLS,
            ),
        }
    }
}

#[derive(Clone, Copy, Default, Component, Reflect)]
pub struct Ant;

pub const COLLISION_GROUP_WALLS: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_SENSOR: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ANTS: Group = Group::GROUP_4;
