use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_rapier2d::geometry::{Collider, CollisionGroups, Group};
use itertools::Itertools;

use super::entities::COLLISION_GROUP_WALLS;

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
        left: Entity,
        left_kind: EdgeNeighborKind,
        right: Entity,
        right_kind: EdgeNeighborKind,
        back: Entity,
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
            } => vec![*left, *right, *back],
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum EdgeNeighborKind {
    Straight,
    Concave,
    Convex,
}

const EMPTY: i32 = 2; //FIXME
pub fn spawn_nav_mesh(
    mut commands: Commands,
    mut level_events: EventReader<LevelEvent>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    tile_storage: Query<(&TileStorage, &Name)>,
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
            ..
        } = &level
            .layer_instances()
            .iter()
            .find(|layer| layer.identifier == "Structure")
            .unwrap();
        let grid_width = *grid_width;
        let grid_height = *grid_height;

        let grid_is_empty = grid_int.iter().map(|i| *i == EMPTY).collect_vec();
        let grid_is_empty: &[bool] = grid_is_empty.as_ref();

        let grid_entity = tile_storage
            .iter()
            .find(|(_, name)| name.as_str() == "Structure")
            .unwrap()
            .0
            .iter()
            .map(|entity| entity.unwrap())
            .collect_vec();
        let grid_entity = grid_entity
            .chunks(grid_width as usize)
            .rev()
            .flatten()
            .copied()
            .collect_vec();
        let grid_entity: &[Entity] = grid_entity.as_ref();

        let grid_iter = (0..grid_entity.len()).map(|i| Index2d::new(i, grid_width, grid_height));

        // Spawn an entity for every edge
        #[derive(Debug, Clone, Copy, Default)]
        struct TileEdges {
            up: Option<Entity>,
            left: Option<Entity>,
            down: Option<Entity>,
            right: Option<Entity>,
        }
        let grid_edges = grid_iter
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
                // dbg!(&tile);
                TileEdges {
                    up: spawn_edge(tile.up()),
                    left: spawn_edge(tile.left()),
                    down: spawn_edge(tile.down()),
                    right: spawn_edge(tile.right()),
                }
            })
            .collect::<Vec<TileEdges>>();

        // Make a Node for each tile and edge, linking them to their neighbors' entities
        for tile in grid_iter {
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
                let (right_kind, right) = if let Some(edge_right) = tile_edges.right {
                    (EdgeNeighborKind::Concave, edge_right)
                } else {
                    let right = tile.right().unwrap();
                    if exists_and_is_empty(right.up(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
                            grid_edges[right.up().unwrap().i()].left.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[right.i()].up.unwrap(),
                        )
                    }
                };
                let (left_kind, left) = if let Some(edge_left) = tile_edges.left {
                    (EdgeNeighborKind::Concave, edge_left)
                } else {
                    let left = tile.left().unwrap();
                    if exists_and_is_empty(left.up(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
                            grid_edges[left.up().unwrap().i()].right.unwrap(),
                        )
                    } else {
                        (EdgeNeighborKind::Straight, grid_edges[left.i()].up.unwrap())
                    }
                };
                let back = grid_entity[tile.i()];
                commands.entity(up_edge).insert((
                    NavNode::HorizontalEdge {
                        left,
                        left_kind,
                        right,
                        right_kind,
                        back,
                        is_up_side: true,
                    },
                    TransformBundle::from_transform(Transform::from_xyz(0., 8., 0.)),
                ));
            }
            // Downside edge
            let down = tile_edges
                .down
                .unwrap_or_else(|| grid_entity[tile.down().unwrap().i()]);
            if let Some(down_edge) = tile_edges.down {
                let (right_kind, right) = if let Some(edge_right) = tile_edges.right {
                    (EdgeNeighborKind::Concave, edge_right)
                } else {
                    let right = tile.right().unwrap();
                    if exists_and_is_empty(right.down(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
                            grid_edges[right.down().unwrap().i()].left.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[right.i()].down.unwrap(),
                        )
                    }
                };
                let (left_kind, left) = if let Some(edge_left) = tile_edges.left {
                    (EdgeNeighborKind::Concave, edge_left)
                } else {
                    let left = tile.left().unwrap();
                    if exists_and_is_empty(left.down(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
                            grid_edges[left.down().unwrap().i()].right.unwrap(),
                        )
                    } else {
                        (
                            EdgeNeighborKind::Straight,
                            grid_edges[left.i()].down.unwrap(),
                        )
                    }
                };
                let back = grid_entity[tile.i()];
                commands.entity(down_edge).insert((
                    NavNode::HorizontalEdge {
                        left,
                        left_kind,
                        right,
                        right_kind,
                        back,
                        is_up_side: false,
                    },
                    TransformBundle::from_transform(Transform::from_xyz(0., -8., 0.)),
                ));
            }
            // Left-side edge
            let left = tile_edges
                .left
                .unwrap_or_else(|| grid_entity[tile.left().unwrap().i()]);
            if let Some(left_edge) = tile_edges.left {
                let (down_kind, down) = if let Some(edge_down) = tile_edges.down {
                    (EdgeNeighborKind::Concave, edge_down)
                } else {
                    let down = tile.down().unwrap();
                    if exists_and_is_empty(down.left(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
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
                    (EdgeNeighborKind::Concave, edge_up)
                } else {
                    let up = tile.up().unwrap();
                    if exists_and_is_empty(up.left(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
                            grid_edges[up.left().unwrap().i()].down.unwrap(),
                        )
                    } else {
                        (EdgeNeighborKind::Straight, grid_edges[up.i()].left.unwrap())
                    }
                };
                let back = grid_entity[tile.i()];
                commands.entity(left_edge).insert((
                    NavNode::VerticalEdge {
                        up,
                        up_kind,
                        down,
                        down_kind,
                        back,
                        is_left_side: true,
                    },
                    TransformBundle::from_transform(Transform::from_xyz(-8., 0., 0.)),
                ));
            }
            // Right-side edge
            let right = tile_edges
                .right
                .unwrap_or_else(|| grid_entity[tile.right().unwrap().i()]);
            if let Some(right_edge) = tile_edges.right {
                let (down_kind, down) = if let Some(edge_down) = tile_edges.down {
                    (EdgeNeighborKind::Concave, edge_down)
                } else {
                    let down = tile.down().unwrap();
                    if exists_and_is_empty(down.right(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
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
                    (EdgeNeighborKind::Concave, edge_up)
                } else {
                    let up = tile.up().unwrap();
                    if exists_and_is_empty(up.right(), grid_entity, grid_is_empty).is_some() {
                        (
                            EdgeNeighborKind::Convex,
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
                commands.entity(right_edge).insert((
                    NavNode::VerticalEdge {
                        up,
                        up_kind,
                        down,
                        down_kind,
                        back,
                        is_left_side: false,
                    },
                    TransformBundle::from_transform(Transform::from_xyz(8., 0., 0.)),
                ));
            }
            commands
                .entity(grid_entity[tile.i()])
                .insert(NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                });
        }
    }
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
                line_between(id, *left, Color::YELLOW, &query_transform, &mut gizmos);
                line_between(id, *right, Color::YELLOW, &query_transform, &mut gizmos);
                line_between(id, *back, Color::YELLOW, &query_transform, &mut gizmos);
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

#[derive(Debug)]
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

pub fn insert_edge_colliders(
    mut commands: Commands,
    nodes: Query<(Entity, &NavNode), Added<NavNode>>,
) {
    let collision_group = CollisionGroups::new(COLLISION_GROUP_WALLS, Group::all());
    for (id, node) in nodes.iter() {
        match *node {
            NavNode::Background { .. } => (),
            NavNode::VerticalEdge {
                up_kind,
                down_kind,
                is_left_side,
                ..
            } => {
                let mut up_pos = match up_kind {
                    EdgeNeighborKind::Straight => Vec2::new(0., 8.),
                    EdgeNeighborKind::Concave => Vec2::new(4., 4.),
                    EdgeNeighborKind::Convex => Vec2::new(-4., 4.),
                };
                let mut down_pos = match down_kind {
                    EdgeNeighborKind::Straight => Vec2::new(0., -8.),
                    EdgeNeighborKind::Concave => Vec2::new(4., -4.),
                    EdgeNeighborKind::Convex => Vec2::new(-4., -4.),
                };
                if !is_left_side {
                    up_pos.x = -up_pos.x;
                    down_pos.x = -down_pos.x;
                }
                let middle_pos = Vec2::ZERO;
                commands.entity(id).insert((
                    Collider::polyline(vec![up_pos, middle_pos, down_pos], None),
                    collision_group,
                ));
            }
            NavNode::HorizontalEdge {
                left_kind,
                right_kind,
                is_up_side,
                ..
            } => {
                let mut left_pos = match left_kind {
                    EdgeNeighborKind::Straight => Vec2::new(-8., 0.),
                    EdgeNeighborKind::Concave => Vec2::new(-4., -4.),
                    EdgeNeighborKind::Convex => Vec2::new(-4., 4.),
                };
                let mut right_pos = match right_kind {
                    EdgeNeighborKind::Straight => Vec2::new(8., 0.),
                    EdgeNeighborKind::Concave => Vec2::new(4., -4.),
                    EdgeNeighborKind::Convex => Vec2::new(4., 4.),
                };
                if !is_up_side {
                    left_pos.y = -left_pos.y;
                    right_pos.y = -right_pos.y;
                }
                let middle_pos = Vec2::ZERO;
                commands.entity(id).insert((
                    Collider::polyline(vec![left_pos, middle_pos, right_pos], None),
                    collision_group,
                ));
            }
        }
    }
}
