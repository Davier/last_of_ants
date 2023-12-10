use bevy::prelude::*;

use crate::{components::nav_mesh::TileEdges, TILE_SIZE};

// FIXME: this works for 1 level only
#[derive(Debug, Default, Resource)]
pub struct NavMeshLUT {
    /// Entities of the tiles
    pub grid_entity: Vec<Entity>,
    /// Entities of the edges of each tile
    pub grid_edges: Vec<TileEdges>,
    /// Identifies empty tiles
    pub grid_is_empty: Vec<bool>,
    /// Number of tiles in X
    pub grid_width: usize,
    /// Number of tiles in Y
    pub grid_height: usize,
}

impl NavMeshLUT {
    pub fn get_tile_entity(&self, mut pos: Vec2) -> Option<(Entity, usize)> {
        if pos.x < 0.
            || pos.y < 0.
            || pos.x > self.grid_width as f32 * TILE_SIZE
            || pos.y > self.grid_height as f32 * TILE_SIZE
        {
            warn!("Trying to find a tile outside the map");
            return None;
        }
        pos.y = self.grid_height as f32 * TILE_SIZE - pos.y;
        let grid_pos_x = (pos.x / TILE_SIZE) as usize;
        let grid_pos_y = (pos.y / TILE_SIZE) as usize;
        let index = grid_pos_x + grid_pos_y * self.grid_width;
        if !self.grid_is_empty[index] {
            warn!("Trying to find a non-empty tile");
            return None;
        }
        Some((self.grid_entity[index], index))
    }

    pub fn get_tile_entity_grid(&self, x: usize, y: usize) -> Option<(Entity, usize)> {
        let index = x + y * self.grid_width;
        if index > self.grid_entity.len() {
            warn!("Trying to find a tile outside the map");
            return None;
        }

        Some((self.grid_entity[index], index))
    }

    pub fn get_tile_edges(&self, tile_index: usize) -> TileEdges {
        self.grid_edges[tile_index]
    }
}
