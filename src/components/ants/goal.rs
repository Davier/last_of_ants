use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::components::{
    nav_mesh::NavNode,
    object::{Object, ObjectKind},
};

use super::movement::AntMovement;

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

#[derive(Resource, Default, Reflect)]
pub struct Metrics {
    pub food: f32,
}

pub fn update_metrics(mut metrics: ResMut<Metrics>, objects: Query<&Object>) {
    metrics.food = 0.;
    for object in objects.iter() {
        match object.kind {
            ObjectKind::Storage => metrics.food += object.quantity.unwrap_or_default(),
            _ => (),
        }
    }
}

pub fn update_ant_goal(
    //commands: &mut Commands,
    mut ants: Query<&mut AntMovement>,
    mut objects: Query<(Entity, &mut Object), With<NavNode>>,
) {
    for mut ant_movement in ants.iter_mut() {
        if let Ok((object_id, mut object)) = objects.get_mut(ant_movement.current_node.0) {
            if object.kind == ant_movement.goal.kind {
                ant_movement.step_goal(/*commands,*/ object_id, &mut object)
            }
        }
    }
}
