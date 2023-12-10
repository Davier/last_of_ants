use crate::render::render_ant::AntMaterialBundle;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use super::ants::*;

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

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct ZombAntQueen;
