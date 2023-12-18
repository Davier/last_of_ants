use bevy::prelude::*;

use crate::components::{
    nav_mesh::NavNode,
    pheromones::{concentrations::PheromoneConcentrations, N_PHEROMONE_KINDS},
};

#[derive(Component)]
pub struct PheromoneGradients {
    pub gradients: [Vec3; N_PHEROMONE_KINDS],
}

impl Default for PheromoneGradients {
    fn default() -> Self {
        Self {
            gradients: [Vec3::ZERO; N_PHEROMONE_KINDS],
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
    mut gradients: Query<(Entity, &mut PheromoneGradients)>,
    nodes: Query<(&NavNode, &PheromoneConcentrations)>,
) {
    for i in 0..N_PHEROMONE_KINDS {
        for (entity, mut gradient) in gradients.iter_mut() {
            let (node, pheromones) = nodes.get(entity).unwrap();
            let mut components = GradientComponents::default();
            match node {
                NavNode::Background {
                    up,
                    left,
                    down,
                    right,
                } => {
                    let (up_neighbour, up_ph) = nodes.get(*up).unwrap();
                    if matches!(up_neighbour, NavNode::Background { .. }) {
                        components.up += up_ph.concentrations[i];
                    } else {
                        components.foreground += up_ph.concentrations[i];
                    }

                    let (down_neighbour, down_ph) = nodes.get(*down).unwrap();
                    if matches!(down_neighbour, NavNode::Background { .. }) {
                        components.down += down_ph.concentrations[i];
                    } else {
                        components.foreground += down_ph.concentrations[i];
                    }

                    let (left_neighbour, left_ph) = nodes.get(*left).unwrap();
                    if matches!(left_neighbour, NavNode::Background { .. }) {
                        components.left += left_ph.concentrations[i];
                    } else {
                        components.foreground += left_ph.concentrations[i];
                    }

                    let (right_neighbour, right_ph) = nodes.get(*right).unwrap();
                    if matches!(right_neighbour, NavNode::Background { .. }) {
                        components.right += right_ph.concentrations[i];
                    } else {
                        components.foreground += right_ph.concentrations[i];
                    }
                }
                NavNode::VerticalEdge { up, down, back, .. } => {
                    let (_, up_ph) = nodes.get(*up).unwrap();
                    components.up += up_ph.concentrations[i];

                    let (_, down_ph) = nodes.get(*down).unwrap();
                    components.down += down_ph.concentrations[i];

                    let (_, back_ph) = nodes.get(*back).unwrap();
                    components.background += back_ph.concentrations[i];
                }
                NavNode::HorizontalEdge {
                    left, right, back, ..
                } => {
                    if let Some(left_id) = left.get() {
                        let (_, left_ph) = nodes.get(left_id).unwrap();
                        components.left += left_ph.concentrations[i];
                    }

                    if let Some(right_id) = right.get() {
                        let (_, right_ph) = nodes.get(right_id).unwrap();
                        components.right += right_ph.concentrations[i];
                    }

                    if let Some(back_id) = back {
                        let (_, back_ph) = nodes.get(*back_id).unwrap();
                        components.background += back_ph.concentrations[i];
                    }
                }
            }

            if pheromones.concentrations[i] >= components.max() {
                gradient.gradients[i] = Vec3::ZERO;
            } else {
                gradient.gradients[i] = components.vec();
            }
        }
    }
}
