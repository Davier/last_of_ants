use ::bevy::prelude::*;

use super::{sources::PheromoneSources, N_PHEROMONE_KINDS};

use crate::components::nav_mesh::NavNode;

use super::{gradients::PheromoneGradients, PheromoneConfig};

#[derive(Component)]
pub struct PheromoneBuffers {
    pub add_buffer: [f32; N_PHEROMONE_KINDS],
}

impl Default for PheromoneBuffers {
    fn default() -> Self {
        Self {
            add_buffer: [0.0; N_PHEROMONE_KINDS],
        }
    }
}

#[derive(Component)]
pub struct PheromoneConcentrations {
    pub concentrations: [f32; N_PHEROMONE_KINDS],
}

impl Default for PheromoneConcentrations {
    fn default() -> Self {
        Self {
            concentrations: [0.0; N_PHEROMONE_KINDS],
        }
    }
}

pub fn init_pheromones(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands.entity(id).insert((
            PheromoneConcentrations::default(),
            PheromoneBuffers::default(),
            PheromoneGradients::default(),
            PheromoneSources::default(),
        ));
    }
}

pub fn diffuse_pheromones(
    mut nav_nodes: Query<(Entity, &NavNode, &mut PheromoneConcentrations), With<PheromoneBuffers>>,
    mut ph_buffers: Query<&mut PheromoneBuffers, With<PheromoneConcentrations>>,
    phcfg: Res<PheromoneConfig>,
) {
    for i in 0..N_PHEROMONE_KINDS {
        // Compute diffusion to neighbours
        for (_, node, node_ph) in nav_nodes.iter() {
            let diffused = node_ph.concentrations[i] * phcfg.diffusion_rate[i];
            if diffused > phcfg.diffusion_floor[i] {
                let neighbors = node.neighbors();
                let diffused_per_neighbor = diffused / neighbors.len() as f32;

                for neighbor in neighbors {
                    let mut neighbor_buffers = ph_buffers.get_mut(neighbor).unwrap();
                    neighbor_buffers.add_buffer[i] += diffused_per_neighbor;
                }
            }
        }

        // Apply diffusion & evaporation
        for (id, _, mut node_ph) in nav_nodes.iter_mut() {
            let mut node_buffers = ph_buffers.get_mut(id).unwrap();

            let new_pheromone_quantity = (node_ph.concentrations[i]
                * (1.0 - phcfg.diffusion_rate[i])
                + node_buffers.add_buffer[i])
                * (1.0 - phcfg.evaporation_rate[i]);

            if new_pheromone_quantity > phcfg.concentration_floor[i] {
                node_ph.concentrations[i] = new_pheromone_quantity;
            } else {
                node_ph.concentrations[i] = 0.;
            }

            node_buffers.add_buffer[i] = 0.;
        }
    }
}
