use bevy::prelude::*;

use crate::{
    components::{
        nav_mesh::NavNode,
        object::{Object, ObjectCoords},
    },
    resources::nav_mesh_lut::NavMeshLUT,
};

use super::{concentrations::PheromoneConcentrations, N_PHEROMONE_KINDS};

#[derive(Bundle)]
pub struct PheromoneSourcesBundle {
    concentrations: PheromoneSources,
    coords: TileMapCoords,
}

#[derive(Component)]
pub struct TileMapCoords {
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

pub fn apply_sources(mut nodes: Query<(&mut PheromoneConcentrations, &PheromoneSources)>) {
    for (mut pheromones, source) in nodes.iter_mut() {
        if let Some(concentrations) = source.concentrations {
            pheromones.concentrations = concentrations;
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
