use std::f32::consts::PI;

use bevy::prelude::*;

use rand::Rng;

use crate::components::{
    object::{Object, ObjectKind},
    pheromones::PheromonsGradients,
};

use super::{goal::AntGoal, AntPositionKind};

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct AntMovement {
    pub position_kind: AntPositionKind,
    pub speed: f32,
    pub direction: Vec3,
    pub current_node: (Entity, GlobalTransform), // FIXME: use relative transforms
    pub goal: AntGoal,
    pub last_direction_update: f32,
}

impl AntMovement {
    pub fn step_goal(
        &mut self,
        /*commands: &mut Commands, FIXME breaks trait for `.chain` in lib */
        object_id: Entity,
        mut object: &mut Object,
    ) {
        match object.kind {
            ObjectKind::Storage => self.goal.step_storage(object, &mut self.direction),
            ObjectKind::Food => self
                .goal
                .step_food(object_id, &mut object, &mut self.direction), /*commands,*/
            _ => (),
        }
    }
}

/// Calculate desired direction of ants according to the gradient of the current node
pub fn update_ant_direction(
    mut ants: Query<&mut AntMovement>,
    gradients: Query<&PheromonsGradients>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for mut ant_movement in ants.iter_mut() {
        let closest_gradient = gradients.get(ant_movement.current_node.0).unwrap();

        let elapsed = time.elapsed_seconds();

        let random = rng.gen_range(0.0..1.0);
        // the gradient for the pheromon the ant follows is not null: follow its direction for at least a second
        let goal_gradient = closest_gradient.gradients[ant_movement.goal.kind as usize];
        if goal_gradient != Vec3::ZERO
            && elapsed - ant_movement.last_direction_update > random + 0.5
        {
            // TODO randomize a bit the direction
            ant_movement.direction = goal_gradient;
            ant_movement.last_direction_update = elapsed;
        } else {
            match ant_movement.position_kind {
                AntPositionKind::Background => {
                    if 0.01 > random {
                        ant_movement.direction =
                            Quat::from_rotation_z(rng.gen_range(-(PI / 2.)..(PI / 2.)))
                                * ant_movement.direction;
                    } else if 0.1 > random {
                        ant_movement.direction =
                            Quat::from_rotation_z(rng.gen_range(-(PI / 6.)..(PI / 6.)))
                                * ant_movement.direction;
                    }
                }
                AntPositionKind::VerticalWall { .. } => {
                    if elapsed - ant_movement.last_direction_update > random + 2. {
                        ant_movement.direction =
                            Quat::from_rotation_x(rng.gen_range(-(PI / 6.)..(PI / 6.)))
                                * ant_movement.direction;
                    }
                }
                AntPositionKind::HorizontalWall { .. } => {
                    if elapsed - ant_movement.last_direction_update > random + 2. {
                        ant_movement.direction =
                            Quat::from_rotation_y(rng.gen_range(-(PI / 6.)..(PI / 6.)))
                                * ant_movement.direction;
                    }
                }
            }
        }
    }
}
