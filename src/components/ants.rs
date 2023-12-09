use std::f32::consts::PI;

use bevy::{ecs::system::EntityCommands, prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    render::render_ant::{
        AntMaterial, AntMaterialBundle, ANT_MATERIAL_SIDE, ANT_MATERIAL_TOP, ANT_MESH2D,
    },
    ANT_SIZE, ANT_WALL_CLIPPING, COLLISION_GROUP_ANTS, COLLISION_GROUP_PLAYER_SENSOR,
    COLLISION_GROUP_WALLS, RENDERLAYER_ANTS, TILE_SIZE, WALL_Z_FACTOR,
};

use super::{
    nav_mesh::{NavMeshLUT, NavNode},
    pheromons::{PheromonGradients, PH1},
};

#[derive(Bundle)]
pub struct AntBundle {
    pub ant: Ant,
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
pub struct Ant {
    pub position_kind: AntPositionKind,
    pub speed: f32,
    /// TODO: Scaled is only used for rendering for now
    pub scale: f32,
    pub color_primary: Color,
    pub color_primary_kind: AntColorKind,
    pub color_secondary: Color,
    pub color_secondary_kind: AntColorKind,
    pub direction: Vec3,
    pub current_wall: (Entity, GlobalTransform), // FIXME: use relative transforms
    pub animation_phase: f32,
    pub goal: usize, // TODO turn into enum
}

/// Kind of color, used to give the player clues
#[derive(Debug, Clone, Copy, Reflect)]
pub enum AntColorKind {
    BLACK,
    RED,
    BROWN,
    GREEN,
    YELLOW,
}

impl AntColorKind {
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        *[
            Self::BLACK,
            Self::RED,
            Self::BROWN,
            Self::GREEN,
            Self::YELLOW,
        ]
        .choose(rng)
        .unwrap()
    }

    /// Not all colors are a good match
    pub fn new_random_from_primary(rng: &mut ThreadRng, primary: &Self) -> Self {
        *match primary {
            AntColorKind::BLACK => [Self::RED, Self::BROWN, Self::GREEN].as_slice(),
            AntColorKind::RED => [Self::BLACK, Self::RED, Self::BROWN].as_slice(),
            AntColorKind::BROWN => [Self::BLACK, Self::RED, Self::BROWN].as_slice(),
            AntColorKind::GREEN => [Self::BLACK, Self::GREEN, Self::YELLOW].as_slice(),
            AntColorKind::YELLOW => [Self::BROWN, Self::GREEN, Self::YELLOW].as_slice(),
        }
        .choose(rng)
        .unwrap()
    }

    pub fn generate_color(&self, rng: &mut ThreadRng) -> Color {
        let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.005; // TODO:cleanup
        match self {
            AntColorKind::BLACK => Color::rgb(shade.x, shade.x, shade.x),
            AntColorKind::RED => Color::rgb(0.1 - shade.x, 0.001 + shade.y, shade.y),
            AntColorKind::BROWN => {
                let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.03;
                Color::rgb(0.15 + shade.x, 0.04 + shade.y, 0.02 + shade.z)
            }
            AntColorKind::GREEN => Color::rgb(shade.x, 0.06 - shade.y, shade.x),
            AntColorKind::YELLOW => {
                let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.2;
                Color::rgb(1. - shade.x, 0.6 - shade.y, 0.)
            }
        }
    }
}

impl AntBundle {
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
            ant: Ant {
                position_kind,
                speed,
                scale,
                direction,
                current_wall,
                animation_phase: rng.gen::<f32>() * 2. * PI,
                color_primary,
                color_primary_kind,
                color_secondary,
                color_secondary_kind,
                goal: PH1,
            },
            material,
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
    ) -> EntityCommands<'w, 's, 'c> {
        let mut command = commands.spawn(AntBundle::new_on_nav_node(
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
        ));
        command.set_parent(entities_holder);
        command
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
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
        &mut Handle<AntMaterial>,
    )>,
    nav_nodes: Query<(Entity, &NavNode, &GlobalTransform)>,
    nav_mesh_lut: Res<NavMeshLUT>,
) {
    for (mut ant, colliding_entities, ant_transform_global, mut ant_transform, mut ant_material) in
        ants.iter_mut()
    {
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
                if ant_transform.translation.z <= ANT_WALL_CLIPPING {
                    // Offset from the wall
                    // TODO: offset from all colliding walls?
                    ant_transform.translation.y += if is_up_side {
                        -2. * ANT_WALL_CLIPPING
                    } else {
                        2. * ANT_WALL_CLIPPING
                    };
                    place_ant_on_background(&mut ant, &mut ant_transform);
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
                    let (wall_entity, wall_node, wall_transform_global) = {
                        let NavNode::HorizontalEdge { left, right, .. } = current_wall.1 else {
                            dbg!(current_wall);
                            panic!(); // FIXME
                        };
                        let neighbor = if new_wall_is_left_side { right } else { left };
                        nav_nodes.get(*neighbor).unwrap()
                    };
                    if !matches!(wall_node, NavNode::VerticalEdge { is_left_side, .. } if *is_left_side == new_wall_is_left_side)
                    {
                        dbg!((
                            *ant,
                            *ant_transform_global,
                            current_wall,
                            wall_entity,
                            wall_node,
                            wall_transform_global
                        ));
                        // A collision was missed, try to fix it
                        assert!(matches!(wall_node, NavNode::HorizontalEdge { .. }));
                        ant.current_wall = (wall_entity, *wall_transform_global);
                    } else {
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
            }
            AntPositionKind::VerticalWall { is_left_side } => {
                // Check if the ant has gone into the background
                if ant_transform.translation.z <= ANT_WALL_CLIPPING {
                    ant_transform.translation.x += if is_left_side {
                        2. * ANT_WALL_CLIPPING
                    } else {
                        -2. * ANT_WALL_CLIPPING
                    };
                    place_ant_on_background(&mut ant, &mut ant_transform);
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
                    let (wall_entity, wall_node, wall_transform_global) = {
                        let NavNode::VerticalEdge { up, down, .. } = current_wall.1 else {
                            dbg!(current_wall);
                            panic!(); // FIXME: triggers if spawning ants too early?
                        };
                        let neighbor = if new_wall_is_up_side { down } else { up };
                        nav_nodes.get(*neighbor).unwrap()
                    };
                    if !matches!(wall_node, NavNode::HorizontalEdge{ is_up_side, .. } if *is_up_side == new_wall_is_up_side)
                    {
                        dbg!((
                            *ant,
                            *ant_transform_global,
                            current_wall,
                            wall_entity,
                            wall_node,
                            wall_transform_global
                        ));
                        // A collision was missed, try to fix it
                        assert!(matches!(wall_node, NavNode::VerticalEdge { .. }));
                        ant.current_wall = (wall_entity, *wall_transform_global);
                    } else {
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
        }
        // Update the current tile if in the background
        if matches!(ant.position_kind, AntPositionKind::Background) {
            let Some((background_entity, _)) =
                nav_mesh_lut.get_tile_entity(ant_transform.translation.xy())
            else {
                continue;
            };
            let (_, nav_node, background_entity_transform) =
                nav_nodes.get(background_entity).unwrap();
            assert!(matches!(nav_node, NavNode::Background { .. }));
            ant.current_wall = (background_entity, *background_entity_transform);
        }
        // Update material
        *ant_material = match ant.position_kind {
            AntPositionKind::Background => ANT_MATERIAL_TOP.clone(),
            AntPositionKind::VerticalWall { .. } | AntPositionKind::HorizontalWall { .. } => {
                ANT_MATERIAL_SIDE.clone()
            }
        }
    }
}

pub fn assert_ants(
    ants: Query<(Entity, &Ant, &GlobalTransform)>,
    nav_nodes: Query<&NavNode>,
    // mut time: ResMut<Time<Virtual>>,
    mut commands: Commands,
) {
    let mut all_ok = true;
    for (entity, ant, ant_transform_global) in ants.iter() {
        let current_nav_node = nav_nodes.get(ant.current_wall.0).unwrap();
        let ok = match ant.position_kind {
            AntPositionKind::Background => matches!(current_nav_node, NavNode::Background { .. }),
            AntPositionKind::VerticalWall { .. } => {
                matches!(current_nav_node, NavNode::VerticalEdge { .. })
            }
            AntPositionKind::HorizontalWall { .. } => {
                matches!(current_nav_node, NavNode::HorizontalEdge { .. })
            }
        };
        if !ok {
            all_ok = false;
            error!("Entity has incorrect current wall assigned");
            dbg!((entity, ant, ant_transform_global, current_nav_node));
            commands.entity(entity).despawn(); // FIXME: parent?
        }
    }
    if !all_ok {
        // time.pause();
    }
}

/// Calculate desired direction of ants according to the navigation mesh
pub fn update_ant_direction(mut ants: Query<&mut Ant>, gradients: Query<&PheromonGradients>) {
    let mut rng = rand::thread_rng();

    for mut ant in ants.iter_mut() {
        let closest_gradient = gradients.get(ant.current_wall.0).unwrap();

        // the gradient for the pheromon the ant follows is not null: follow it
        let goal_gradient = closest_gradient.gradients[ant.goal].extend(0.);
        if goal_gradient != Vec3::ZERO {
            ant.direction = goal_gradient;
        } else {
            let random = rng.gen_range(0.0..1.0);
            if 0.01 > random {
                ant.direction =
                    Quat::from_rotation_z(rng.gen_range(-(PI / 2.)..(PI / 2.))) * ant.direction;
            } else if 0.1 > random {
                ant.direction =
                    Quat::from_rotation_z(rng.gen_range(-(PI / 6.)..(PI / 6.))) * ant.direction;
            }
        }
    }
}

pub fn update_ant_direction_randomly(mut ants: Query<&mut Ant>, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_seconds_f64();
    for mut ant in ants.iter_mut() {
        if rng.gen_bool(dt) {
            ant.direction = Vec3::new(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
            )
            .normalize();
        }
    }
}

/// Move ants according to their direction and the constraints of [AntPositionKind]
pub fn update_ant_position(mut ants: Query<(&Ant, &mut Transform)>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for (ant, mut ant_transform) in ants.iter_mut() {
        let dt = dt.min(TILE_SIZE / 4. / ant.speed); // Clamp to avoid going through walls when lagging
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
    let offset = (ANT_SIZE.y / 2. - ANT_WALL_CLIPPING) * if is_up_side { -1. } else { 1. };
    ant_transform.translation.y += wall_transform_relative.translation.y + offset;
    if ant_transform.translation.z < ANT_WALL_CLIPPING {
        ant_transform.translation.z = ANT_WALL_CLIPPING;
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
    let offset = (ANT_SIZE.x / 2. - ANT_WALL_CLIPPING) * if is_left_side { 1. } else { -1. };
    ant_transform.translation.x += wall_transform_relative.translation.x + offset;
    if ant_transform.translation.z < ANT_WALL_CLIPPING {
        ant_transform.translation.z = ANT_WALL_CLIPPING;
    }
}

fn place_ant_on_background(ant: &mut Ant, ant_transform: &mut Transform) {
    ant.position_kind = AntPositionKind::Background;
    ant_transform.translation.z = 0.;
    // TODO: place away from walls?
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
