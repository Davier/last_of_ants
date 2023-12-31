use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    components::ants::AntStyle,
    components::pheromones::{
        concentrations::PheromoneConcentrations, PheromoneConfig, PheromoneKind,
    },
    render::render_ant::{AntMaterialBundle, ANT_MATERIAL_DEAD, ANT_MESH2D},
    resources::nav_mesh_lut::NavMeshLUT,
    ANT_SIZE, COLLISION_GROUP_DEAD_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALLS,
    RENDERLAYER_ANTS,
};

#[derive(Debug, Component, Reflect)]
pub struct DeadAnt;

#[derive(Bundle)]
pub struct DeadAntBundle {
    pub dead_ant: DeadAnt,
    pub ant_style: AntStyle,
    pub material: AntMaterialBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_mass: ColliderMassProperties,
    pub active_events: ActiveEvents,
    pub active_collisions: ActiveCollisionTypes,
    pub collision_groups: CollisionGroups,
    pub render_layers: RenderLayers,
    pub ccd: Ccd,
}

impl DeadAntBundle {
    pub fn new(ant_transform: Transform, ant_style: AntStyle) -> Self {
        Self {
            dead_ant: DeadAnt,
            ant_style,
            material: AntMaterialBundle {
                mesh: ANT_MESH2D,
                material: ANT_MATERIAL_DEAD,
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
            ccd: Ccd::enabled(),
            active_events: ActiveEvents::all(),
            active_collisions: ActiveCollisionTypes::all(),
        }
    }
}

pub fn update_dead_ants_deposit(
    dead_ants: Query<&GlobalTransform, With<DeadAnt>>,
    mut nodes: Query<&mut PheromoneConcentrations>,
    navmesh_lut: Res<NavMeshLUT>,
    phcfg: Res<PheromoneConfig>,
) {
    for dead_ant_transform in dead_ants.iter() {
        let (node, _) = navmesh_lut
            .get_tile_entity(dead_ant_transform.translation().xy())
            .unwrap();
        let mut pheromones = nodes.get_mut(node).unwrap();
        pheromones.concentrations[PheromoneKind::DeadAnt as usize] += phcfg.dead_ant_deposit;
    }
}
