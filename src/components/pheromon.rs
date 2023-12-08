use bevy::prelude::*;

use super::nav_mesh::NavNode;

pub const PH1: usize = 0;
pub const PH2: usize = 1;
pub const N_PH: usize = 2;
pub const PH_DIFFUSION_RATE: [f32;N_PH] = [0.0;N_PH];

#[derive(Component)]
pub struct PheromonsBuffer(pub [f32;N_PH]);

impl Default for PheromonsBuffer {
    fn default() -> Self {
        Self([0.0;N_PH])
    }
}

#[derive(Component)]
pub struct Pheromons(pub [f32;N_PH]);

impl Default for Pheromons {
    fn default() -> Self {
        Self([0.0;N_PH])
    }
}

#[derive(Component)]
pub struct Gradient(pub [Vec2;N_PH]);

impl Default for Gradient {
    fn default() -> Self {
        Self([Vec2::ZERO;N_PH])
    }
}

pub fn init_pheromons(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands.entity(id).insert((
            Pheromons::default(),
            PheromonsBuffer::default(),
            Gradient::default(),
        ));
    }
}


pub fn pheromon_diffusion(
    mut query_nodes: Query<(Entity, &NavNode, &mut Pheromons), With<PheromonsBuffer>>,
    mut query_pheromon_buffers: Query<&mut PheromonsBuffer, With<Pheromons>>,
) {
    // % going to neighbours
    let diffusion_rate = 0.01;

    // Compute diffusion to neighbours
    for i in 0..N_PH {
        for (_, node, ph) in query_nodes.iter() {
            let diffused = ph.0[i] * diffusion_rate;
            if diffused > 0.005 { // TODO extract quantity
                let neighbors = node.neighbors();
                let diffused_per_neighbor = diffused / neighbors.len() as f32;

                for neighbor in neighbors {
                    let mut ph_b_neighbor = query_pheromon_buffers.get_mut(neighbor).unwrap();
                    ph_b_neighbor.0[i] += diffused_per_neighbor;
                }
            }
        }

        // Apply diffusion
        for (id, _, mut ph) in query_nodes.iter_mut() {
            let mut ph_b = query_pheromon_buffers.get_mut(id).unwrap();
            let new_pheromon_quantity = ph.0[i] * (1.0 - diffusion_rate) + ph_b.0[i];
            if new_pheromon_quantity > 0.001{ // TODO extract quantity
                ph.0[i] = new_pheromon_quantity;
            } else {
                ph.0[i] = 0.;
            }
            ph_b.0[i] = 0.;
        }
    }
}

pub fn update_gradient(
    mut query_changed_nodes: Query<(Entity, &NavNode, &Pheromons, &mut Gradient)>,
    query_pheromon: Query<&Pheromons, With<NavNode>>,
) {
    let up = Vec2::new(0.0, 1.0);
    let down = Vec2::new(0.0, -1.0);
    let right = Vec2::new(1.0, 0.0);
    let left = Vec2::new(-1.0, 0.0);

    for i in 0..N_PH {
        for (id, node, ph, mut gd) in query_changed_nodes.iter_mut() {
            let (n, s, e, w, b) = match node {
                NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                } => (
                    query_pheromon.get(*up).unwrap().0[i],
                    query_pheromon.get(*down).unwrap().0[i],
                    query_pheromon.get(*right).unwrap().0[i],
                    query_pheromon.get(*left).unwrap().0[i],
                    0.,
                ),
                NavNode::HorizontalEdge {
                    left, right, back, ..
                } => (
                    0.0,
                    0.0,
                    query_pheromon.get(*right).unwrap().0[i],
                    query_pheromon.get(*left).unwrap().0[i],
                    query_pheromon.get(*back).unwrap().0[i],
                ),
                NavNode::VerticalEdge { up, down, back, .. } => (
                    query_pheromon.get(*up).unwrap().0[i],
                    query_pheromon.get(*down).unwrap().0[i],
                    0.0,
                    0.0,
                    query_pheromon.get(*back).unwrap().0[i],
                ),
            };
            // TODO: back

            if ph.0[i] >= n.max(s).max(e).max(w) {
                gd.0[i] = Vec2::ZERO;
            } else {
                gd.0[i] = n * up + s * down + e * right + w * left;
            }
        }
    }
}
