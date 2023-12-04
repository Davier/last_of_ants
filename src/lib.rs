pub mod components;
pub mod helpers;

use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, systems::process_ldtk_levels};

use components::{entities::*, nav_mesh::*, tiles::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, spawn_nav_mesh.after(process_ldtk_levels))
            .add_systems(Update, (setup_entity_sprites,))
            .register_ldtk_entity::<PlayerBundle>("Player")
            // .register_ldtk_entity::<AntBundle>("Ant")
            .register_ldtk_int_cell::<TileGround>(1)
            .register_ldtk_int_cell::<TileEmptyUnderground>(2)
            .register_type::<components::nav_mesh::NavNode>();
    }
}
