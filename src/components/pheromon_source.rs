use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::{LdtkEntity, LdtkFields};

use super::pheromons::{DEFAULT, FOOD_SOURCE, FOOD_STORE};

#[derive(Clone, Copy, Reflect, Component)]
pub enum Object {
    Food {
        quantity: Option<f32>,
        concentration: f32,
    },
    Storage {
        quantity: f32,
        concentration: f32,
    },
    Corpse {
        quantity: i32,
        concentration: f32,
    },
    Cemetery {
        quantity: i32,
        concentration: f32,
    },
}

impl Object {
    pub fn pheromon_type(&self) -> usize {
        match self {
            Object::Food { .. } => FOOD_SOURCE,
            Object::Storage { .. } => FOOD_STORE,
            Object::Corpse { .. } => DEFAULT,
            Object::Cemetery { .. } => DEFAULT,
        }
    }

    pub fn concentration(&self) -> f32 {
        match self {
            Object::Food { concentration, .. } => *concentration,
            Object::Storage { concentration, .. } => *concentration,
            Object::Corpse { concentration, .. } => *concentration,
            Object::Cemetery { concentration, .. } => *concentration,
        }
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
        let object = match LdtkFields::get_enum_field(entity_instance, "SourceType")
            .unwrap()
            .as_str()
        {
            "Food" => {
                let quantity = *entity_instance.get_maybe_float_field("Quantity").unwrap();
                let concentration = *entity_instance.get_float_field("Concentration").unwrap();
                Object::Food {
                    quantity,
                    concentration,
                }
            }
            "Storage" => {
                let quantity = *entity_instance.get_float_field("Quantity").unwrap();
                let concentration = *entity_instance.get_float_field("Concentration").unwrap();
                Object::Storage {
                    quantity,
                    concentration,
                }
            }
            _ => unreachable!(),
        };

        let coords = ObjectCoords {
            x: entity_instance.grid.x,
            y: entity_instance.grid.y,
        };

        Self { object, coords }
    }
}
