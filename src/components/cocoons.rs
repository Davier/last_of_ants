use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_ecs_ldtk::prelude::*;

use crate::{
    render::render_cocoon::{
        CocoonMaterialBundle, COCOON_MATERIAL, COCOON_MATERIAL_CLUE, COCOON_MESH2D,
    },
    COCOON_ROOMS,
};

#[derive(Bundle)]
pub struct CocoonBundle {
    pub cocoon: Cocoon,
    pub material: CocoonMaterialBundle,
}

/// Cocoons sprinkled on the map. A few of them are clues.
#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
pub struct Cocoon {
    pub is_clue: bool,
    pub room: u8,
}

impl Default for CocoonBundle {
    fn default() -> Self {
        Self::new(false, 0)
    }
}

impl CocoonBundle {
    pub fn new(is_clue: bool, room: u8) -> Self {
        Self {
            cocoon: Cocoon { is_clue, room },
            material: MaterialMesh2dBundle {
                mesh: COCOON_MESH2D.clone(),
                material: if is_clue {
                    COCOON_MATERIAL_CLUE.clone()
                } else {
                    COCOON_MATERIAL.clone()
                },
                ..default()
            },
        }
    }
}

impl LdtkEntity for CocoonBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        _layer_instance: &LayerInstance,
        _tileset: Option<&Handle<Image>>,
        _tileset_definition: Option<&TilesetDefinition>,
        _asset_server: &AssetServer,
        _texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let room = *entity_instance.get_int_field("Room").unwrap() as u8;
        assert!(COCOON_ROOMS.contains(&room));
        Self::new(false, room)
    }
}
