use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_rapier2d::geometry::{Collider, CollisionGroups};
use itertools::Itertools;

use crate::{
    resources::nav_mesh_lut::NavMeshLUT, AppState, ANT_WALL_CLIPPING, COLLISION_GROUP_ANTS,
    COLLISION_GROUP_DEAD_ANTS, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PLAYER_SENSOR,
    COLLISION_GROUP_WALLS, TILE_INT_EMPTY, TILE_INT_GROUND, TILE_INT_OVERGROUND, WALL_Z_FACTOR,
};

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub enum NavNode {
    Background {
        up: Entity,
        left: Entity,
        down: Entity,
        right: Entity,
    },
    VerticalEdge {
        up: Entity,
        up_kind: EdgeNeighborKind,
        down: Entity,
        down_kind: EdgeNeighborKind,
        back: Entity,
        is_left_side: bool,
    },
    HorizontalEdge {
        left: EdgeNeighbor,
        right: EdgeNeighbor,
        back: Option<Entity>,
        is_up_side: bool,
    },
}

// This is only to have reflection
impl Default for NavNode {
    fn default() -> Self {
        Self::Background {
            up: Entity::PLACEHOLDER,
            left: Entity::PLACEHOLDER,
            down: Entity::PLACEHOLDER,
            right: Entity::PLACEHOLDER,
        }
    }
}

impl NavNode {
    pub fn neighbors(&self) -> Vec<Entity> {
        match self {
            NavNode::Background {
                up,
                left,
                down,
                right,
            } => vec![*up, *left, *down, *right],
            NavNode::VerticalEdge { up, down, back, .. } => vec![*up, *down, *back],
            NavNode::HorizontalEdge {
                left, right, back, ..
            } => [left.get(), right.get(), *back]
                .iter()
                .flatten()
                .copied()
                .collect_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum EdgeNeighborKind {
    Straight,
    Inward,
    Outward,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum EdgeNeighbor {
    Straight(Entity),
    Inward(Entity),
    Outward(Entity),
    None,
}

impl EdgeNeighbor {
    pub fn get(&self) -> Option<Entity> {
        match self {
            EdgeNeighbor::Straight(e) | EdgeNeighbor::Inward(e) | EdgeNeighbor::Outward(e) => {
                Some(*e)
            }
            EdgeNeighbor::None => None,
        }
    }
}
pub fn spawn_nav_mesh(
    mut commands: Commands,
    mut level_events: EventReader<LevelEvent>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    tile_storage: Query<(&TileStorage, &Name)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for level_event in level_events.read() {
        let LevelEvent::Transformed(level_iid) = level_event else {
            continue;
        };
        // Ants can be in empty background tile or at the edge between an empty tile and a wall
        // The entities for tiles are spawned by LDtk, we spawn the entities for edges
        let ldtk_project = ldtk_project_assets.get(ldtk_projects.single()).unwrap();

        let level = ldtk_project
            .as_standalone()
            .get_loaded_level_by_iid(&level_iid.to_string())
            .unwrap();

        let LayerInstance {
            c_hei: grid_height,
            c_wid: grid_width,
            int_grid_csv: grid_int,
            grid_size: tile_size,
            ..
        } = &level
            .layer_instances()
            .iter()
            .find(|layer| layer.identifier == "Structure")
            .unwrap();
        let grid_width = *grid_width;
        let grid_height = *grid_height;
        let half_tile_size = *tile_size as f32 / 2.;

        let grid_is_empty_vec = grid_int.iter().map(|i| *i == TILE_INT_EMPTY).collect_vec();
        let grid_is_empty: &[bool] = grid_is_empty_vec.as_ref();

        let grid_entity_vec = tile_storage
            .iter()
            .find(|(_, name)| name.as_str() == "Structure")
            .unwrap()
            .0
            .iter()
            .map(|entity| entity.unwrap())
            .collect_vec();
        let grid_entity_vec = grid_entity_vec
            .chunks(grid_width as usize)
            .rev()
            .flatten()
            .copied()
            .collect_vec();
        let grid_entity: &[Entity] = grid_entity_vec.as_ref();

        let grid_iter = (0..grid_entity.len()).map(|i| Index2d::new(i, grid_width, grid_height));

        let mut entities_bundle = HashMap::new();

        // Spawn an entity for every edge
        let mut grid_edges = grid_iter
            .clone()
            .map(|tile| {
                // Non empty tiles have no edges
                if !grid_is_empty[tile.i()] {
                    return TileEdges::default();
                }
                // Spawn an edge if a neighbor tile exists and is not empty
                let mut spawn_edge = |neighbor: Option<Index2d>| {
                    exists_and_is_empty(neighbor, grid_entity, grid_is_empty)
                        .is_none()
                        .then(|| {
                            let mut id = Entity::PLACEHOLDER;
                            commands
                                .entity(grid_entity[tile.i()])
                                .with_children(|parent| {
                                    id = parent.spawn_empty().id();
                                });
                            id
                        })
                };
                TileEdges {
                    up: spawn_edge(tile.up()),
                    left: spawn_edge(tile.left()),
                    down: spawn_edge(tile.down()),
                    right: spawn_edge(tile.right()),
                }
            })
            .collect::<Vec<TileEdges>>();

        // Make a Node for each tile and edge, linking them to their neighbors' entities
        let wall_collision_group = CollisionGroups::new(
            COLLISION_GROUP_WALLS,
            COLLISION_GROUP_PLAYER
                | COLLISION_GROUP_PLAYER_SENSOR
                | COLLISION_GROUP_ANTS
                | COLLISION_GROUP_DEAD_ANTS,
        );
        for tile in grid_iter.clone() {
            // Non empty tiles have no edges
            if !grid_is_empty[tile.i()] {
                continue;
            }
            let tile_edges = grid_edges[tile.i()];
            // Upside edge
            let up = tile_edges
                .up
                .unwrap_or_else(|| grid_entity[tile.up().unwrap().i()]);
            if let Some(up_edge) = tile_edges.up {
                let right = if let Some(edge_right) = tile_edges.right {
                    EdgeNeighbor::Inward(edge_right)
                } else {
                    let right = tile.right().unwrap();
                    if exists_and_is_empty(right.up(), grid_entity, grid_is_empty).is_some() {
                        EdgeNeighbor::Outward(grid_edges[right.up().unwrap().i()].left.unwrap())
                    } else {
                        EdgeNeighbor::Straight(grid_edges[right.i()].up.unwrap())
                    }
                };
                let left = if let Some(edge_left) = tile_edges.left {
                    EdgeNeighbor::Inward(edge_left)
                } else {
                    let left = tile.left().unwrap();
                    if exists_and_is_empty(left.up(), grid_entity, grid_is_empty).is_some() {
                        EdgeNeighbor::Outward(grid_edges[left.up().unwrap().i()].right.unwrap())
                    } else {
                        EdgeNeighbor::Straight(grid_edges[left.i()].up.unwrap())
                    }
                };
                let back = Some(grid_entity[tile.i()]);
                entities_bundle.insert(
                    up_edge,
                    (
                        NavNode::HorizontalEdge {
                            left,
                            right,
                            back,
                            is_up_side: true,
                        },
                        TransformBundle::from_transform(Transform::from_xyz(
                            0.,
                            half_tile_size,
                            half_tile_size * WALL_Z_FACTOR,
                        )),
                        Collider::polyline(
                            vec![
                                Vec2::new(-half_tile_size, 0.),
                                Vec2::new(half_tile_size, 0.),
                            ],
                            None,
                        ),
                        wall_collision_group,
                    ),
                );
            }
            // Downside edge
            let down = tile_edges
                .down
                .unwrap_or_else(|| grid_entity[tile.down().unwrap().i()]);
            if let Some(down_edge) = tile_edges.down {
                let right = if let Some(edge_right) = tile_edges.right {
                    EdgeNeighbor::Inward(edge_right)
                } else {
                    let right = tile.right().unwrap();
                    if exists_and_is_empty(right.down(), grid_entity, grid_is_empty).is_some() {
                        EdgeNeighbor::Outward(grid_edges[right.down().unwrap().i()].left.unwrap())
                    } else {
                        EdgeNeighbor::Straight(grid_edges[right.i()].down.unwrap())
                    }
                };
                let left = if let Some(edge_left) = tile_edges.left {
                    EdgeNeighbor::Inward(edge_left)
                } else {
                    let left = tile.left().unwrap();
                    if exists_and_is_empty(left.down(), grid_entity, grid_is_empty).is_some() {
                        EdgeNeighbor::Outward(grid_edges[left.down().unwrap().i()].right.unwrap())
                    } else {
                        EdgeNeighbor::Straight(grid_edges[left.i()].down.unwrap())
                    }
                };
                let back = Some(grid_entity[tile.i()]);
                entities_bundle.insert(
                    down_edge,
                    (
                        NavNode::HorizontalEdge {
                            left,
                            right,
                            back,
                            is_up_side: false,
                        },
                        TransformBundle::from_transform(Transform::from_xyz(
                            0.,
                            -half_tile_size,
                            half_tile_size * WALL_Z_FACTOR,
                        )),
                        Collider::polyline(
                            vec![
                                Vec2::new(-half_tile_size, 0.),
                                Vec2::new(half_tile_size, 0.),
                            ],
                            None,
                        ),
                        wall_collision_group,
                    ),
                );
            }
            // Left-side edge
            let left = tile_edges
                .left
                .unwrap_or_else(|| grid_entity[tile.left().unwrap().i()]);
            if let Some(left_edge) = tile_edges.left {
                let (down_kind, down) = if let Some(edge_down) = tile_edges.down {
                    (EdgeNeighborKind::Inward, edge_down)
                } else {
                    let down = tile.down().unwrap();
                    if exists_and_is_empty(down.left(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Outward,
                            grid_edges[down.left().unwrap().i()].up.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[down.i()].left.unwrap(),
                        )
                    }
                };
                let (up_kind, up) = if let Some(edge_up) = tile_edges.up {
                    (EdgeNeighborKind::Inward, edge_up)
                } else {
                    let up = tile.up().unwrap();
                    if exists_and_is_empty(up.left(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Outward,
                            grid_edges[up.left().unwrap().i()].down.unwrap(),
                        )
                    } else {
                        (EdgeNeighborKind::Straight, grid_edges[up.i()].left.unwrap())
                    }
                };
                let back = grid_entity[tile.i()];
                entities_bundle.insert(
                    left_edge,
                    (
                        NavNode::VerticalEdge {
                            up,
                            up_kind,
                            down,
                            down_kind,
                            back,
                            is_left_side: true,
                        },
                        TransformBundle::from_transform(Transform::from_xyz(
                            -half_tile_size,
                            0.,
                            half_tile_size * WALL_Z_FACTOR,
                        )),
                        Collider::polyline(
                            vec![
                                Vec2::new(0., half_tile_size),
                                Vec2::new(0., -half_tile_size),
                            ],
                            None,
                        ),
                        wall_collision_group,
                    ),
                );
            }
            // Right-side edge
            let right = tile_edges
                .right
                .unwrap_or_else(|| grid_entity[tile.right().unwrap().i()]);
            if let Some(right_edge) = tile_edges.right {
                let (down_kind, down) = if let Some(edge_down) = tile_edges.down {
                    (EdgeNeighborKind::Inward, edge_down)
                } else {
                    let down = tile.down().unwrap();
                    if exists_and_is_empty(down.right(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Outward,
                            grid_edges[down.right().unwrap().i()].up.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[down.i()].right.unwrap(),
                        )
                    }
                };
                let (up_kind, up) = if let Some(edge_up) = tile_edges.up {
                    (EdgeNeighborKind::Inward, edge_up)
                } else {
                    let up = tile.up().unwrap();
                    if exists_and_is_empty(up.right(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Outward,
                            grid_edges[up.right().unwrap().i()].down.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[up.i()].right.unwrap(),
                        )
                    }
                };
                let back = grid_entity[tile.i()];
                entities_bundle.insert(
                    right_edge,
                    (
                        NavNode::VerticalEdge {
                            up,
                            up_kind,
                            down,
                            down_kind,
                            back,
                            is_left_side: false,
                        },
                        TransformBundle::from_transform(Transform::from_xyz(
                            half_tile_size,
                            0.,
                            half_tile_size * WALL_Z_FACTOR,
                        )),
                        Collider::polyline(
                            vec![
                                Vec2::new(0., half_tile_size),
                                Vec2::new(0., -half_tile_size),
                            ],
                            None,
                        ),
                        wall_collision_group,
                    ),
                );
            }
            // FIXME: use entities_bundle?
            commands
                .entity(grid_entity[tile.i()])
                .insert(NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                });
        }

        // Spawn surface edges
        let mut surface_edges = Vec::new();
        for tile in grid_iter.clone() {
            if grid_int[tile.i()] != TILE_INT_OVERGROUND {
                continue;
            }
            let Some(down) = tile.down() else {
                error!("Overground tile on the bottom of the map");
                continue;
            };
            let down_kind_is_ground = grid_int[down.i()] == TILE_INT_GROUND;
            if down_kind_is_ground {
                let edge = commands
                    .spawn_empty()
                    .set_parent(grid_entity[tile.i()])
                    .id();
                grid_edges[tile.i()].down = Some(edge);
                surface_edges.push((tile, edge));
            }
        }

        // Link surface edges
        for (tile, down_edge) in surface_edges.iter() {
            let get_down_edge = |neighbor: Option<Index2d>| {
                neighbor
                    .filter(|neighbor| grid_int[neighbor.i()] != TILE_INT_GROUND)
                    .and_then(|neighbor| grid_edges[neighbor.i()].down)
            };
            let left_edge_entity = get_down_edge(tile.left());
            let left_edge = match left_edge_entity {
                Some(e) => EdgeNeighbor::Straight(e),
                None => EdgeNeighbor::None,
            };
            // If linking to the underground, fix their link and collider too
            if let Some(left_edge_entity) = left_edge_entity {
                let left_tile = tile.left().unwrap();

                if grid_int[left_tile.i()] == TILE_INT_EMPTY {
                    let left_edge_nav_node =
                        &mut entities_bundle.get_mut(&left_edge_entity).unwrap().0;
                    // Fix link
                    let NavNode::HorizontalEdge { ref mut right, .. } = left_edge_nav_node else {
                        panic!()
                    };
                    *right = EdgeNeighbor::Straight(*down_edge);
                    // Fix collider
                    let left_edge_collider = &mut entities_bundle
                        .get_mut(&grid_edges[left_tile.i()].right.unwrap())
                        .unwrap()
                        .2;
                    *left_edge_collider = Collider::polyline(
                        vec![
                            Vec2::new(0., half_tile_size),
                            Vec2::new(0., half_tile_size - ANT_WALL_CLIPPING + 0.1),
                        ],
                        None,
                    );
                }
            }
            let right_edge_entity = get_down_edge(tile.right());
            let right_edge = match right_edge_entity {
                Some(e) => EdgeNeighbor::Straight(e),
                None => EdgeNeighbor::None,
            };
            // If linking to the underground, fix their link and collider too
            if let Some(right_edge_entity) = right_edge_entity {
                let right_tile = tile.right().unwrap();

                if grid_int[right_tile.i()] == TILE_INT_EMPTY {
                    let right_edge_nav_node =
                        &mut entities_bundle.get_mut(&right_edge_entity).unwrap().0;
                    // Fix link
                    let NavNode::HorizontalEdge { ref mut left, .. } = right_edge_nav_node else {
                        panic!()
                    };
                    *left = EdgeNeighbor::Straight(*down_edge);
                    // Fix collider
                    let right_edge_collider = &mut entities_bundle
                        .get_mut(&grid_edges[right_tile.i()].left.unwrap())
                        .unwrap()
                        .2;
                    *right_edge_collider = Collider::polyline(
                        vec![
                            Vec2::new(0., half_tile_size),
                            Vec2::new(0., half_tile_size - ANT_WALL_CLIPPING + 0.1),
                        ],
                        None,
                    );
                }
            }

            entities_bundle.insert(
                *down_edge,
                (
                    NavNode::HorizontalEdge {
                        left: left_edge,
                        right: right_edge,
                        back: None,
                        is_up_side: false,
                    },
                    TransformBundle::from_transform(Transform::from_xyz(
                        0.,
                        -half_tile_size,
                        half_tile_size * WALL_Z_FACTOR,
                    )),
                    Collider::polyline(
                        vec![
                            Vec2::new(-half_tile_size, 0.),
                            Vec2::new(half_tile_size, 0.),
                        ],
                        None,
                    ),
                    wall_collision_group,
                ),
            );
        }

        // Insert the bundles
        commands.insert_or_spawn_batch(entities_bundle);

        // Save the look-up tables
        commands.insert_resource(NavMeshLUT {
            grid_entity: grid_entity_vec,
            grid_edges,
            grid_is_empty: grid_is_empty_vec,
            grid_width: grid_width as usize,
            grid_height: grid_height as usize,
            tile_width: *tile_size as usize,
            tile_height: *tile_size as usize,
        });

        next_state.set(AppState::ProcessingOthers);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TileEdges {
    pub up: Option<Entity>,
    pub left: Option<Entity>,
    pub down: Option<Entity>,
    pub right: Option<Entity>,
}

pub fn debug_nav_mesh(
    query_nodes: Query<(Entity, &NavNode)>,
    query_transform: Query<&GlobalTransform, With<NavNode>>,
    mut gizmos: Gizmos,
) {
    for (id, node) in query_nodes.iter() {
        match node {
            NavNode::Background {
                up,
                left,
                down,
                right,
            } => {
                line_between(id, *up, Color::GREEN, &query_transform, &mut gizmos);
                line_between(id, *down, Color::GREEN, &query_transform, &mut gizmos);
                line_between(id, *left, Color::GREEN, &query_transform, &mut gizmos);
                line_between(id, *right, Color::GREEN, &query_transform, &mut gizmos);
            }
            NavNode::VerticalEdge { up, down, back, .. } => {
                line_between(id, *up, Color::YELLOW, &query_transform, &mut gizmos);
                line_between(id, *down, Color::YELLOW, &query_transform, &mut gizmos);
                line_between(id, *back, Color::YELLOW, &query_transform, &mut gizmos);
            }
            NavNode::HorizontalEdge {
                left, right, back, ..
            } => {
                if let Some(left) = left.get() {
                    line_between(id, left, Color::YELLOW, &query_transform, &mut gizmos);
                }
                if let Some(right) = right.get() {
                    line_between(id, right, Color::YELLOW, &query_transform, &mut gizmos);
                }
                if let Some(back) = back {
                    line_between(id, *back, Color::YELLOW, &query_transform, &mut gizmos);
                }
            }
        }
    }
}

fn line_between(
    node_a: Entity,
    node_b: Entity,
    color: Color,
    query_transform: &Query<&GlobalTransform, With<NavNode>>,
    gizmos: &mut Gizmos,
) {
    let pos_a = query_transform.get(node_a);
    let pos_b = query_transform.get(node_b);
    if let (Ok(pos_a), Ok(pos_b)) = (pos_a, pos_b) {
        let pos_a = pos_a.translation();
        let pos_b = pos_b.translation();
        let pos_middle = (pos_a + pos_b) / 2.;
        gizmos.line(pos_a, pos_middle, color);
    } else {
        warn!("Nav mesh is broken between {:?} and {:?}", node_a, node_b);
    }
}

fn exists_and_is_empty(
    i: Option<Index2d>,
    grid_entity: &[Entity],
    grid_is_empty: &[bool],
) -> Option<Entity> {
    let i = i?;
    if grid_is_empty[i.i()] {
        Some(grid_entity[i.i()])
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
struct Index2d {
    index: i32,
    grid_width: i32,
    grid_height: i32,
}

impl Index2d {
    fn new(i: usize, grid_width: i32, grid_height: i32) -> Self {
        Self {
            index: i as i32,
            grid_width,
            grid_height,
        }
    }
    fn i(&self) -> usize {
        self.index as usize
    }
    fn x(&self) -> i32 {
        self.index % self.grid_width
    }
    fn y(&self) -> i32 {
        self.index / self.grid_width
    }
    fn up(&self) -> Option<Self> {
        if self.y() == 0 {
            return None;
        }
        Some(Self {
            index: self.index - self.grid_width,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
        })
    }
    fn left(&self) -> Option<Self> {
        if self.x() == 0 {
            return None;
        }
        Some(Self {
            index: self.index - 1,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
        })
    }
    fn down(&self) -> Option<Self> {
        if self.y() == self.grid_height - 1 {
            return None;
        }
        Some(Self {
            index: self.index + self.grid_width,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
        })
    }
    fn right(&self) -> Option<Self> {
        if self.x() == self.grid_width - 1 {
            return None;
        }
        Some(Self {
            index: self.index + 1,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
        })
    }
}
