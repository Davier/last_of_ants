use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    ANT_SIZE, COLLISION_GROUP_DEAD_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALLS,
    RENDERLAYER_ANTS,
};

#[derive(Debug, Component, Reflect)]
pub struct DeadAnt;

#[derive(Bundle)]
pub struct DeadAntBundle {
    pub dead_ant: DeadAnt,
    pub sprite: SpriteBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_mass: ColliderMassProperties,
    pub render_layers: RenderLayers,
    pub collision_groups: CollisionGroups,
}

impl DeadAntBundle {
    pub fn new(ant_transform: Transform) -> Self {
        Self {
            dead_ant: DeadAnt,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(ANT_SIZE * 2. / 3.),
                    ..default()
                },
                transform: ant_transform,
                ..default()
            },
            rigid_body: RigidBody::Dynamic,
            // Dead ants need to be smaller than ants, otherwise they are clipping with the player
            // when they spawn and are flung away
            collider: Collider::cuboid(ANT_SIZE.x / 3., ANT_SIZE.y / 3.),
            collider_mass: ColliderMassProperties::Density(1.),
            render_layers: RENDERLAYER_ANTS,
            collision_groups: CollisionGroups::new(
                COLLISION_GROUP_DEAD_ANTS,
                COLLISION_GROUP_PLAYER | COLLISION_GROUP_WALLS,
            ),
        }
    }
}
