use bevy::prelude::*;

use crate::{components::object::ObjectCoords, resources::nav_mesh_lut::NavMeshLUT};

use super::{nav_mesh::NavNode, object::Object};

#[derive(Default, Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum PheromoneKind {
    #[default]
    Default = 0,
    Storage = 1,
    Food = 2,
    Zombqueen = 3,
    Zombant = 4,
}
pub const N_PHEROMONE_KINDS: usize = 5;

pub const DEFAULT: usize = 0;
pub const FOOD_STORE: usize = 1;
pub const FOOD_SOURCE: usize = 2;

#[derive(Resource, Reflect)]
pub struct PheromoneConfig {
    evaporation_rate: [f32; N_PHEROMONE_KINDS],
    diffusion_rate: [f32; N_PHEROMONE_KINDS],
    diffusion_floor: [f32; N_PHEROMONE_KINDS],
    concentration_floor: [f32; N_PHEROMONE_KINDS],
    pub color: [(Color, Color); N_PHEROMONE_KINDS],
    pub zombant_deposit: f32,
    pub zombqueen_source: f32,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        use PheromoneKind::*;

        let mut config = Self {
            evaporation_rate: [0.001; N_PHEROMONE_KINDS],
            diffusion_rate: [0.01, 0.6, 0.6, 0.6, 0.],
            diffusion_floor: [0.001; N_PHEROMONE_KINDS],
            concentration_floor: [0.001; N_PHEROMONE_KINDS],
            color: [(Color::BLACK, Color::WHITE); N_PHEROMONE_KINDS],
            zombant_deposit: 1.0,
            zombqueen_source: 40.0,
        };

        config.color[Default as usize] = (Color::PURPLE, Color::FUCHSIA);
        config.color[Food as usize] = (Color::DARK_GREEN, Color::LIME_GREEN);
        config.color[Storage as usize] = (Color::BLUE, Color::AZURE);
        config.color[Zombqueen as usize] = (Color::MAROON, Color::CRIMSON);
        config.color[Zombant as usize] = (Color::BEIGE, Color::DARK_GRAY);

        config.evaporation_rate[Zombant as usize] = 0.1;
        config.diffusion_rate[Zombant as usize] = 0.01;

        config.evaporation_rate[Zombqueen as usize] = 0.01;
        config.diffusion_rate[Zombqueen as usize] = 0.9;
        config.diffusion_floor[Zombqueen as usize] = 0.0001;
        config.concentration_floor[Zombqueen as usize] = 0.0001;

        config
    }
}

#[derive(Bundle)]
pub struct PheromoneSourceBundle {
    value: PheromoneSources,
    coord: SourceCoord,
}

#[derive(Component)]
pub struct SourceCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Default)]
pub struct PheromoneSources {
    pub concentrations: Option<[f32; N_PHEROMONE_KINDS]>,
}

impl PheromoneSources {
    pub fn set(&mut self, kind: usize, concentration: f32) {
        if let Some(mut concentrations) = self.concentrations {
            concentrations[kind] = concentration;
        } else {
            let mut concentrations = [0.0_f32; N_PHEROMONE_KINDS];
            concentrations[kind] = concentration;
            self.concentrations = Some(concentrations);
        }
    }

    pub fn clear(&mut self, kind: usize) {
        if let Some(mut concentrations) = self.concentrations {
            concentrations[kind] = 0.;
            if concentrations.iter().all(|c| *c == 0.) {
                self.concentrations = None;
            }
        }
    }

    pub fn add(&mut self, kind: usize, concentration: f32) {
        if let Some(mut concentrations) = self.concentrations {
            concentrations[kind] += concentration;
        } else {
            let mut concentrations = [0.0_f32; N_PHEROMONE_KINDS];
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

#[derive(Component)]
pub struct PheromonsGradients {
    pub gradients: [Vec3; N_PHEROMONE_KINDS],
}

impl Default for PheromonsGradients {
    fn default() -> Self {
        Self {
            gradients: [Vec3::ZERO; N_PHEROMONE_KINDS],
        }
    }
}

pub fn init_pheromons(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands.entity(id).insert((
            PheromoneConcentrations::default(),
            PheromoneBuffers::default(),
            PheromonsGradients::default(),
            PheromoneSources::default(),
        ));
    }
}

pub fn apply_sources(mut nodes: Query<(&mut PheromoneConcentrations, &PheromoneSources)>) {
    for (mut pheromons, source) in nodes.iter_mut() {
        if let Some(concentrations) = source.concentrations {
            pheromons.concentrations = concentrations;
        }
    }
}

pub fn diffuse_pheromons(
    mut nav_nodes: Query<(Entity, &NavNode, &mut PheromoneConcentrations), With<PheromoneBuffers>>,
    mut pheromon_buffers: Query<&mut PheromoneBuffers, With<PheromoneConcentrations>>,
    phcfg: Res<PheromoneConfig>,
) {
    for i in 0..N_PHEROMONE_KINDS {
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

#[derive(Default)]
struct GradientComponents {
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
    pub background: f32,
    pub foreground: f32,
}

impl GradientComponents {
    fn max(&self) -> f32 {
        self.up
            .max(self.down)
            .max(self.left)
            .max(self.right)
            .max(self.background)
            .max(self.foreground)
    }

    fn vec(&self) -> Vec3 {
        Vec3::new(
            self.right - self.left,
            self.up - self.down,
            self.foreground - self.background,
        )
    }
}

pub fn compute_gradients(
    mut gradients: Query<(Entity, &mut PheromonsGradients)>,
    nodes: Query<(&NavNode, &PheromoneConcentrations)>,
) {
    for i in 0..N_PHEROMONE_KINDS {
        for (entity, mut gradient) in gradients.iter_mut() {
            let (node, pheromons) = nodes.get(entity).unwrap();
            let mut components = GradientComponents::default();
            match node {
                NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                } => {
                    let (up_neighbour, up_pheromons) = nodes.get(*up).unwrap();
                    if matches!(up_neighbour, NavNode::Background { .. }) {
                        components.up += up_pheromons.concentrations[i];
                    } else {
                        components.foreground += up_pheromons.concentrations[i];
                    }

                    let (down_neighbour, down_pheromons) = nodes.get(*down).unwrap();
                    if matches!(down_neighbour, NavNode::Background { .. }) {
                        components.down += down_pheromons.concentrations[i];
                    } else {
                        components.foreground += down_pheromons.concentrations[i];
                    }

                    let (left_neighbour, left_pheromons) = nodes.get(*left).unwrap();
                    if matches!(left_neighbour, NavNode::Background { .. }) {
                        components.left += left_pheromons.concentrations[i];
                    } else {
                        components.foreground += left_pheromons.concentrations[i];
                    }

                    let (right_neighbour, right_pheromons) = nodes.get(*right).unwrap();
                    if matches!(right_neighbour, NavNode::Background { .. }) {
                        components.right += right_pheromons.concentrations[i];
                    } else {
                        components.foreground += right_pheromons.concentrations[i];
                    }
                }
                NavNode::VerticalEdge { up, down, back, .. } => {
                    let (_, up_pheromons) = nodes.get(*up).unwrap();
                    components.up += up_pheromons.concentrations[i];

                    let (_, down_pheromons) = nodes.get(*down).unwrap();
                    components.down += down_pheromons.concentrations[i];

                    let (_, back_pheromons) = nodes.get(*back).unwrap();
                    components.background += back_pheromons.concentrations[i];
                }
                NavNode::HorizontalEdge {
                    left, right, back, ..
                } => {
                    if let Some(left_id) = left.get() {
                        let (_, left_pheromons) = nodes.get(left_id).unwrap();
                        components.left += left_pheromons.concentrations[i];
                    }

                    if let Some(right_id) = right.get() {
                        let (_, right_pheromons) = nodes.get(right_id).unwrap();
                        components.right += right_pheromons.concentrations[i];
                    }

                    if let Some(back_id) = back {
                        let (_, back_pheromons) = nodes.get(*back_id).unwrap();
                        components.background += back_pheromons.concentrations[i];
                    }
                }
            }

            if pheromons.concentrations[i] >= components.max() {
                gradient.gradients[i] = Vec3::ZERO;
            } else {
                gradient.gradients[i] = components.vec();
            }
        }
    }
}

pub fn init_sources(
    mut commands: Commands,
    sources: Query<(Entity, &Object, &ObjectCoords)>,
    nav_mesh_lut: Res<NavMeshLUT>,
    mut nodes: Query<&mut PheromoneSources, With<NavNode>>,
) {
    for (tile_id, object, ObjectCoords { x, y }) in sources.iter() {
        let (node_id, _) = nav_mesh_lut
            .get_tile_entity_grid(*x as usize, *y as usize)
            .unwrap(); // FIXME: bad order

        if let Ok(mut node_source) = nodes.get_mut(node_id) {
            node_source.add(object.kind(), object.concentration);

            // Object is added to the corresponding NavNode
            // then removed from the tile so that it won't come up again.
            commands.get_entity(node_id).unwrap().insert(*object);
            commands.get_entity(tile_id).unwrap().remove::<Object>();
            commands
                .get_entity(tile_id)
                .unwrap()
                .remove::<ObjectCoords>();
        }
    }
}
