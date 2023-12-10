use crate::{render::render_ant::AntMaterialBundle, resources::nav_mesh_lut::NavMeshLUT};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::LdtkEntity;
use bevy_rapier2d::prelude::*;
use rand::{seq::IteratorRandom, thread_rng, Rng};

use super::{
    ants::{goal::AntGoal, *, movement::AntMovement},
    nav_mesh::NavNode,
};

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

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct ZombAntQueen;

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
            zombant_queen: ZombAntQueen,
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
