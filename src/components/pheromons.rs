use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::{LdtkEntity, LdtkFields};

use super::nav_mesh::NavNode;

pub const PH1: usize = 0;
pub const PH2: usize = 1;
pub const N_PH: usize = 2;
pub const PH_EVAPORATION_RATE: [f32; N_PH] = [0.005; N_PH];
pub const PH_DIFFUSION_RATE: [f32; N_PH] = [0.3, 0.03];
pub const PH_DIFFUSION_FLOOR: [f32; N_PH] = [0.001,0.002];
pub const PH_FLOOR: [f32; N_PH] = [0.001, 0.005];

#[derive(Bundle)]
pub struct PheromonSourceBundle {
    value: PheromonSource,
    coord: SourceCoord,
}

#[derive(Component)]
pub struct SourceCoord {
    pub x: i32,
    pub y: i32,
}

// TODO associate source to navnode
#[derive(Component, Debug, Default)]
pub struct PheromonSource {
    pub value: f32,
}

impl LdtkEntity for PheromonSourceBundle {
    fn bundle_entity(
        entity_instance: &bevy_ecs_ldtk::EntityInstance,
        _: &bevy_ecs_ldtk::prelude::LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
        Self {
            value: PheromonSource {
                value: *entity_instance.get_float_field("Value").unwrap(),
            },
            coord: SourceCoord {
                x: entity_instance.grid.x,
                y: entity_instance.grid.y,
            },
        }
    }
}
#[derive(Component)]
pub struct PheromonsBuffer {
    pub buffers: [f32; N_PH],
}

impl Default for PheromonsBuffer {
    fn default() -> Self {
        Self {
            buffers: [0.0; N_PH],
        }
    }
}

#[derive(Component)]
pub struct Pheromons {
    pub pheromons: [f32; N_PH],
}

impl Default for Pheromons {
    fn default() -> Self {
        Self {
            pheromons: [0.0; N_PH],
        }
    }
}

#[derive(Component)]
pub struct PheromonGradients {
    pub gradients: [Vec2; N_PH],
}

impl Default for PheromonGradients {
    fn default() -> Self {
        Self {
            gradients: [Vec2::ZERO; N_PH],
        }
    }
}

pub fn init_pheromons(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands.entity(id).insert((
            Pheromons::default(),
            PheromonsBuffer::default(),
            PheromonGradients::default(),
        ));
    }
}

pub fn apply_sources(mut nodes: Query<(&mut Pheromons, &PheromonSource)>) {
    for (mut pheromons, source) in nodes.iter_mut() {
        pheromons.pheromons[PH1] = source.value;
    }
}

pub fn diffuse_pheromons(
    mut nav_nodes: Query<(Entity, &NavNode, &mut Pheromons), With<PheromonsBuffer>>,
    mut pheromon_buffers: Query<&mut PheromonsBuffer, With<Pheromons>>,
) {
    for i in 0..N_PH {
        // Compute diffusion to neighbours
        for (_, node, node_pheromons) in nav_nodes.iter() {
            let diffused = node_pheromons.pheromons[i] * PH_DIFFUSION_RATE[i];
            if diffused > PH_DIFFUSION_FLOOR[i] {
                let neighbors = node.neighbors();
                let diffused_per_neighbor = diffused / neighbors.len() as f32;

                for neighbor in neighbors {
                    let mut neighbor_buffers = pheromon_buffers.get_mut(neighbor).unwrap();
                    neighbor_buffers.buffers[i] += diffused_per_neighbor;
                }
            }
        }

        // Apply diffusion & evaporation
        for (id, _, mut node_pheromons) in nav_nodes.iter_mut() {
            let mut node_buffers = pheromon_buffers.get_mut(id).unwrap();

            let new_pheromon_quantity = (node_pheromons.pheromons[i]
                * (1.0 - PH_DIFFUSION_RATE[i])
                + node_buffers.buffers[i])
                * (1.0 - PH_EVAPORATION_RATE[i]);

            if new_pheromon_quantity > PH_FLOOR[i] {
                node_pheromons.pheromons[i] = new_pheromon_quantity;
            } else {
                node_pheromons.pheromons[i] = 0.;
            }

            node_buffers.buffers[i] = 0.;
        }
    }
}

pub fn compute_gradients(
    mut nodes: Query<(Entity, &NavNode, &Pheromons, &mut PheromonGradients)>,
    pheromons: Query<&Pheromons, With<NavNode>>,
) {
    let up = Vec2::new(0.0, 1.0);
    let down = Vec2::new(0.0, -1.0);
    let right = Vec2::new(1.0, 0.0);
    let left = Vec2::new(-1.0, 0.0);
    let foreground = Vec3::new(0., 0., 1.);
    let background = Vec3::new(0.0, 0.0, -1.0);

    for i in 0..N_PH {
        for (id, node, ph, mut gd) in nodes.iter_mut() {
            let (n, s, e, w, b) = match node {
                NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                } => (
                    pheromons.get(*up).unwrap().pheromons[i],
                    pheromons.get(*down).unwrap().pheromons[i],
                    pheromons.get(*right).unwrap().pheromons[i],
                    pheromons.get(*left).unwrap().pheromons[i],
                    0.,
                ),
                NavNode::HorizontalEdge {
                    left, right, back, ..
                } => (
                    0.0,
                    0.0,
                    pheromons.get(*right).unwrap().pheromons[i],
                    pheromons.get(*left).unwrap().pheromons[i],
                    pheromons.get(*back).unwrap().pheromons[i],
                ),
                NavNode::VerticalEdge { up, down, back, .. } => (
                    pheromons.get(*up).unwrap().pheromons[i],
                    pheromons.get(*down).unwrap().pheromons[i],
                    0.0,
                    0.0,
                    pheromons.get(*back).unwrap().pheromons[i],
                ),
            };
            // TODO: back

            if ph.pheromons[i] >= n.max(s).max(e).max(w).max(b) {
                gd.gradients[i] = Vec2::ZERO;
            } else {
                gd.gradients[i] = n * up + s * down + e * right + w * left;
            }
        }
    }
}
