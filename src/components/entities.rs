use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

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
                    custom_size: Some(Vec2::new(8., 16.)),
                    ..default()
                },
                ..default()
            },
            collider: Collider::cuboid(4., 8.),
            player: Default::default(),
            controller: KinematicCharacterController {
                min_slope_slide_angle: PI / 5.,
                filter_groups: Some(CollisionGroups::new(
                    COLLISION_GROUP_PLAYER,
                    COLLISION_GROUP_WALLS | COLLISION_GROUP_ANTS,
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
        player.is_on_ceiling = false;
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
                NavNode::HorizontalEdge { is_up_side: false, .. } => {
                    player.on_ground.insert(colliding_entity);
                }
                NavNode::HorizontalEdge { is_up_side: true, .. } => {
                    player.is_on_ceiling = true;
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
    pub is_on_ceiling: bool,
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
        let offset = 2.;
        commands
            .spawn((
                PlayerWallSensor { player: entity },
                Collider::cuboid(size[0] + offset, size[1] + offset),
                Sensor,
                TransformBundle::default(),
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
                    custom_size: Some(Vec2::new(8., 8.)),
                    ..default()
                },
                ..default()
            },
            collider: Collider::cuboid(4., 4.),
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
