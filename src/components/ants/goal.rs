use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::components::object::{Object, ObjectKind};

#[derive(Default, Debug, Clone, Copy, Reflect)]
pub struct AntGoal {
    pub kind: ObjectKind,
    pub holds: f32,
}

impl AntGoal {
    pub fn step_food(
        &mut self,
        // commands: &mut Commands,
        object_id: Entity,
        object: &mut Object,
        direction: &mut Vec3,
    ) {
        self.kind = ObjectKind::Storage;

        // None quantity represents unlimited resource
        if let Some(quantity) = object.quantity {
            if quantity > 1.0 {
                object.quantity = Some(quantity - 1.0);
                // TODO concentration variation when resource quantity decreases
            } else {
                // Remove food when depleted
                // commands.get_entity(object_id).unwrap().remove::<Object>();
            }
        }

        self.holds = 1.0;
        *direction *= -1.;
    }

    pub fn step_storage(&mut self, object: &mut Object, direction: &mut Vec3) {
        self.kind = ObjectKind::Food;

        object.quantity = object.quantity.map(|q| q + 1.0).or(Some(1.0));
        self.holds = 0.;
        *direction *= -1.;
    }
}
