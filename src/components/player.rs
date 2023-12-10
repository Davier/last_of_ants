use std::f32::consts::PI;

use bevy::{prelude::*, render::view::RenderLayers, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    resources::clues::ClueEvent, COLLISION_GROUP_ANTS, COLLISION_GROUP_CLUE,
    COLLISION_GROUP_DEAD_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PLAYER_SENSOR,
    COLLISION_GROUP_WALLS, PLAYER_SIZE, RENDERLAYER_PLAYER,
};

use super::{
    ants::{AntMovement, AntPositionKind, AntStyle},
    cocoons::Clue,
    dead_ants::DeadAntBundle,
    nav_mesh::NavNode,
};

#[derive(Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,
    pub sprite: SpriteBundle,
    pub controller: KinematicCharacterController,
    pub collider: Collider,
    pub collider_mass: ColliderMassProperties,
    // The velocity is only used by us since it's not a RigidBody
    pub velocity: Velocity,
    pub render_layers: RenderLayers,
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
            collider_mass: ColliderMassProperties::Density(1.),
            player: Default::default(),
            controller: KinematicCharacterController {
                min_slope_slide_angle: PI / 5.,
                filter_groups: Some(CollisionGroups::new(
                    COLLISION_GROUP_PLAYER,
                    COLLISION_GROUP_WALLS | COLLISION_GROUP_DEAD_ANTS,
                )),
                apply_impulse_to_dynamic_bodies: false, // FIXME: I couldn't get it to work
                autostep: Some(CharacterAutostep {
                    include_dynamic_bodies: true,
                    max_height: CharacterLength::Relative(0.4),
                    ..default()
                }),
                ..Default::default()
            },
            velocity: Default::default(),
            render_layers: RENDERLAYER_PLAYER,
        }
    }
}

pub fn update_player_sensor(
    mut commands: Commands,
    player_sensors: Query<(&PlayerWallSensor, &CollidingEntities)>,
    mut players: Query<&mut Player>,
    nav_nodes: Query<&NavNode>,
    ants: Query<(&AntMovement, &Transform, &Parent, &AntStyle)>,
    clues: Query<(Entity, &Parent), With<Clue>>,
    mut clue_events: EventWriter<ClueEvent>,
) {
    for (sensor, colliding_entities) in player_sensors.iter() {
        let Ok(mut player) = players.get_mut(sensor.player) else {
            warn!("Unattached player sensor");
            continue;
        };
        player.is_on_left_wall = false;
        player.is_on_right_wall = false;
        // player.is_on_ceiling = false;
        player.on_ground.clear();
        player.on_wall.clear();
        for colliding_entity in colliding_entities.iter() {
            // Collision with ant
            if let Ok((ant_movement, ant_transform, ant_parent, ant_style)) =
                ants.get(colliding_entity)
            {
                if !matches!(ant_movement.position_kind, AntPositionKind::Background) {
                    // Spawn a dead ant
                    commands
                        .spawn(DeadAntBundle::new(*ant_transform, *ant_style))
                        .set_parent(ant_parent.get());
                    // Despawn the alive ant
                    commands
                        .entity(ant_parent.get())
                        .remove_children(&[colliding_entity]);
                    commands.entity(colliding_entity).despawn();
                }
            // Collision with wall
            } else if let Ok(nav_node) = nav_nodes.get(colliding_entity) {
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
            // Collision with a clue
            } else if let Ok((entity, parent)) = clues.get(colliding_entity) {
                // TODO: SFX
                commands.entity(parent.get()).remove_children(&[entity]);
                commands.entity(entity).despawn();
                clue_events.send(ClueEvent::Found);
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
                CollisionGroups::new(
                    COLLISION_GROUP_PLAYER_SENSOR,
                    COLLISION_GROUP_WALLS | COLLISION_GROUP_ANTS | COLLISION_GROUP_CLUE,
                ),
            ))
            .set_parent(entity);
    }
}
