use bevy::prelude::*;
use bevy_ecs_ldtk::LevelEvent;

use crate::components::pheromon_source::ObjectCoords;

use super::{
    nav_mesh::{NavMeshLUT, NavNode},
    pheromon_source::Object,
};

pub const DEFAULT: usize = 0;
pub const FOOD_STORE: usize = 1;
pub const FOOD_SOURCE: usize = 2;
pub const N_PH: usize = 3;

#[derive(Resource, Reflect)]
pub struct PheromonsConfig {
    evaporation_rate: [f32; N_PH],
    diffusion_rate: [f32; N_PH],
    diffusion_floor: [f32; N_PH],
    concentration_floor: [f32; N_PH],
}

impl Default for PheromonsConfig {
    fn default() -> Self {
        Self {
            evaporation_rate: [0.005; N_PH],
            diffusion_rate: [0.01, 0.4, 0.1],
            diffusion_floor: [0.001; N_PH],
            concentration_floor: [0.001; N_PH],
        }
    }
}

#[derive(Bundle)]
pub struct PheromonSourceBundle {
    value: PheromonsSource,
    coord: SourceCoord,
}

#[derive(Component)]
pub struct SourceCoord {
    pub x: i32,
    pub y: i32,
}

// TODO associate source to navnode
#[derive(Component, Debug, Default)]
pub struct PheromonsSource {
    pub concentrations: Option<[f32; N_PH]>,
}

impl PheromonsSource {
    pub fn add(&mut self, kind: usize, concentration: f32) {
        if let Some(mut concentrations) = self.concentrations {
            concentrations[kind] += concentration;
        } else {
            let mut concentrations = [0.0_f32; N_PH];
            concentrations[kind] = concentration;
            self.concentrations = Some(concentrations);
        }
    }

    pub fn sub(&mut self, kind: usize, concentration: f32) {
        if let Some(mut concentrations) = self.concentrations {
            concentrations[kind] = (concentrations[kind] - concentration).max(0.);
            if concentrations.iter().all(|c| *c == 0.) {
                self.concentrations = None;
            }
        }
    }
}

#[derive(Component)]
pub struct PheromonsBuffers {
    pub add_buffer: [f32; N_PH],
}

impl Default for PheromonsBuffers {
    fn default() -> Self {
        Self {
            add_buffer: [0.0; N_PH],
        }
    }
}

#[derive(Component)]
pub struct Pheromons {
    pub concentrations: [f32; N_PH],
}

impl Default for Pheromons {
    fn default() -> Self {
        Self {
            concentrations: [0.0; N_PH],
        }
    }
}

#[derive(Component)]
pub struct PheromonsGradients {
    pub gradients: [Vec2; N_PH],
}

impl Default for PheromonsGradients {
    fn default() -> Self {
        Self {
            gradients: [Vec2::ZERO; N_PH],
        }
    }
}

pub fn init_pheromons(
    mut commands: Commands,
    nodes: Query<(Entity, &NavNode), Added<NavNode>>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        let LevelEvent::Transformed(_) = level_event else {
            continue;
        };

        for (id, node) in nodes.iter() {
            debug!("Init pheromons for nodes.");
            commands.entity(id).insert((
                Pheromons::default(),
                PheromonsBuffers::default(),
                PheromonsGradients::default(),
                PheromonsSource::default(),
            ));
        }
    }
}

pub fn apply_sources(mut nodes: Query<(&mut Pheromons, &PheromonsSource)>) {
    for (mut pheromons, source) in nodes.iter_mut() {
        if let Some(concentrations) = source.concentrations {
            pheromons.concentrations = concentrations;
        }
    }
}

pub fn diffuse_pheromons(
    mut nav_nodes: Query<(Entity, &NavNode, &mut Pheromons), With<PheromonsBuffers>>,
    mut pheromon_buffers: Query<&mut PheromonsBuffers, With<Pheromons>>,
    phcfg: Res<PheromonsConfig>,
) {
    for i in 0..N_PH {
        // Compute diffusion to neighbours
        for (_, node, node_pheromons) in nav_nodes.iter() {
            let diffused = node_pheromons.concentrations[i] * phcfg.diffusion_rate[i];
            if diffused > phcfg.diffusion_floor[i] {
                let neighbors = node.neighbors();
                let diffused_per_neighbor = diffused / neighbors.len() as f32;

                for neighbor in neighbors {
                    let mut neighbor_buffers = pheromon_buffers.get_mut(neighbor).unwrap();
                    neighbor_buffers.add_buffer[i] += diffused_per_neighbor;
                }
            }
        }

        // Apply diffusion & evaporation
        for (id, _, mut node_pheromons) in nav_nodes.iter_mut() {
            let mut node_buffers = pheromon_buffers.get_mut(id).unwrap();

            let new_pheromon_quantity = (node_pheromons.concentrations[i]
                * (1.0 - phcfg.diffusion_rate[i])
                + node_buffers.add_buffer[i])
                * (1.0 - phcfg.evaporation_rate[i]);

            if new_pheromon_quantity > phcfg.concentration_floor[i] {
                node_pheromons.concentrations[i] = new_pheromon_quantity;
            } else {
                node_pheromons.concentrations[i] = 0.;
            }

            node_buffers.add_buffer[i] = 0.;
        }
    }
}

pub fn compute_gradients(
    mut nodes: Query<(Entity, &NavNode, &Pheromons, &mut PheromonsGradients)>,
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
                    pheromons.get(*up).unwrap().concentrations[i],
                    pheromons.get(*down).unwrap().concentrations[i],
                    pheromons.get(*right).unwrap().concentrations[i],
                    pheromons.get(*left).unwrap().concentrations[i],
                    0.,
                ),
                NavNode::HorizontalEdge {
                    left, right, back, ..
                } => (
                    0.0,
                    0.0,
                    pheromons.get(*right).unwrap().concentrations[i],
                    pheromons.get(*left).unwrap().concentrations[i],
                    pheromons.get(*back).unwrap().concentrations[i],
                ),
                NavNode::VerticalEdge { up, down, back, .. } => (
                    pheromons.get(*up).unwrap().concentrations[i],
                    pheromons.get(*down).unwrap().concentrations[i],
                    0.0,
                    0.0,
                    pheromons.get(*back).unwrap().concentrations[i],
                ),
            };
            // TODO: back

            if ph.concentrations[i] >= n.max(s).max(e).max(w).max(b) {
                gd.gradients[i] = Vec2::ZERO;
            } else {
                gd.gradients[i] = n * up + s * down + e * right + w * left;
            }
        }
    }
}

pub fn init_sources(
    mut commands: Commands,
    sources: Query<(&Object, &ObjectCoords)>,
    nav_mesh_lut: Res<NavMeshLUT>,
    mut nodes: Query<&mut PheromonsSource, With<NavNode>>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        let LevelEvent::Transformed(_) = level_event else {
            continue;
        };

        for (object, ObjectCoords { x, y }) in sources.iter() {
            let (node_id, _) = nav_mesh_lut
                .get_tile_entity_grid(*x as usize, *y as usize)
                .unwrap();

            if let Ok(mut node_source) = nodes.get_mut(node_id) {
                debug!("Init source.");
                node_source.add(object.pheromon_type(), object.concentration());

                commands.get_entity(node_id).unwrap().insert(*object);
            }
        }
    }
}
