use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::{LdtkEntity, LdtkFields};

use crate::components::pheromones::PheromoneKind;

#[derive(Clone, Default, Debug, Copy, Component, Reflect)]
pub struct Object {
    pub kind: PheromoneKind,
    pub quantity: Option<f32>,
    pub concentration: f32,
}

impl Object {
    pub fn kind(&self) -> usize {
        self.kind as usize
    }
}

#[derive(Bundle)]
pub struct ObjectBundle {
    pub object: Object,
    pub coords: ObjectCoords,
}

#[derive(Component)]
pub struct ObjectCoords {
    pub x: i32,
    pub y: i32,
}

impl LdtkEntity for ObjectBundle {
    fn bundle_entity(
        entity_instance: &bevy_ecs_ldtk::EntityInstance,
        _: &bevy_ecs_ldtk::prelude::LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
        let kind = match LdtkFields::get_enum_field(entity_instance, "SourceType")
            .unwrap()
            .as_str()
        {
            "Food" => PheromoneKind::Food,
            "Storage" => PheromoneKind::Storage,
            _ => PheromoneKind::Default,
        };

        let quantity = *entity_instance.get_maybe_float_field("Quantity").unwrap();
        let concentration = *entity_instance.get_float_field("Concentration").unwrap();

        let coords = ObjectCoords {
            x: entity_instance.grid.x,
            y: entity_instance.grid.y,
        };

        let object = Object {
            kind,
            quantity,
            concentration,
        };

        Self { object, coords }
    }
}
