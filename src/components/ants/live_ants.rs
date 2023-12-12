use std::f32::consts::PI;

use bevy::{ecs::system::EntityCommands, prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;
use rand::{rngs::ThreadRng, Rng};

use crate::{
    components::nav_mesh::NavNode,
    render::render_ant::{AntMaterialBundle, ANT_MATERIAL_SIDE, ANT_MATERIAL_TOP, ANT_MESH2D},
    ANT_SIZE, ANT_WALL_CLIPPING, COLLISION_GROUP_ANTS, COLLISION_GROUP_EXPLOSION,
    COLLISION_GROUP_PLAYER_SENSOR, COLLISION_GROUP_WALLS, RENDERLAYER_ANTS, TILE_SIZE,
    WALL_Z_FACTOR,
};

use super::{
    goal::AntGoal, movement::position::AntPositionKind, movement::AntMovement, AntColorKind,
    AntStyle,
};

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct LiveAnt {}

#[derive(Bundle)]
pub struct LiveAntBundle {
    pub live_ant: LiveAnt,
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

impl LiveAntBundle {
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
            live_ant: LiveAnt {},
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
        let mut command = commands.spawn(LiveAntBundle::new_on_nav_node(
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
