use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::components::{
    nav_mesh::NavNode, object::Object, pheromones::PheromoneKind, zombants::ZombAntQueen,
};

use super::{job::Job, movement::AntMovement};

#[derive(Default, Debug, Clone, Copy, Reflect)]
pub struct AntGoal {
    pub job: Job,
    pub holds: f32,
}

impl AntGoal {
    pub fn reached_food_target(
        &mut self,
        // commands: &mut Commands,
        object_id: Entity,
        object: &mut Object,
        direction: &mut Vec3,
    ) {
        self.job = Job::Storage;

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

    pub fn reached_zombqueen(&mut self, direction: &mut Vec3, zombqueen: &mut ZombAntQueen) {
        match self.job {
            Job::Offering => {
                self.job = Job::Thief;
                self.holds = 0.0;
                zombqueen.holds += 2.;
                *direction *= -1.;
            }
            _ => (),
        }
    }

    pub fn reached_storage_target(&mut self, object: &mut Object, direction: &mut Vec3) {
        match self.job {
            Job::Storage => {
                self.job = Job::Food;

                object.quantity = object.quantity.map(|q| q + 1.0).or(Some(1.0));
                self.holds = 0.;
                *direction *= -1.;
            }
            Job::Thief => {
                self.job = Job::Offering;

                if let Some(quantity) = object.quantity {
                    object.quantity = Some((quantity - 2.).max(0.));
                    self.holds = 2.;
                    *direction *= -1.;
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Resource, Default, Reflect)]
pub struct Metrics {
    pub food: f32,
    pub stolen: f32,
}

pub fn update_metrics(
    mut metrics: ResMut<Metrics>,
    objects: Query<&Object>,
    zombqueen: Query<&ZombAntQueen>,
) {
    metrics.food = 0.;
    for object in objects.iter() {
        match object.kind {
            PheromoneKind::Storage => metrics.food += object.quantity.unwrap_or_default(),
            _ => (),
        }
    }

    if let Ok(zombqueen) = zombqueen.get_single() {
        metrics.stolen = zombqueen.holds;
    }
}

pub fn update_ant_goal(
    //commands: &mut Commands,
    mut ants: Query<&mut AntMovement, Without<ZombAntQueen>>,
    mut objects: Query<(Entity, &mut Object, &GlobalTransform), With<NavNode>>,
    mut zombqueen: Query<(&mut ZombAntQueen, &AntMovement)>,
) {
    for mut ant_movement in ants.iter_mut() {
        let current_object = objects.get_mut(ant_movement.current_node.0);
        if let Ok((object_id, mut object, _)) = current_object {
            if object.kind == ant_movement.goal.job.follows() {
                ant_movement.reached_object(/*commands,*/ object_id, &mut object)
            }
        }

        if let Ok((mut zombqueen, zombqueen_movement)) = zombqueen.get_single_mut() {
            if zombqueen_movement.current_node.0 == ant_movement.current_node.0 {
                ant_movement.reached_zombqueen(&mut zombqueen);
            }
        }
    }
}
