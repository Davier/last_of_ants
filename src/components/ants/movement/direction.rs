use std::f32::consts::PI;

use crate::components::pheromones::PheromoneGradients;
use bevy::prelude::*;

use super::{position::AntPositionKind, AntMovement};

use rand::Rng;

/// Calculate desired direction of ants according to the gradient of the current node
pub fn update_ant_direction(
    mut ants: Query<&mut AntMovement>,
    gradients: Query<&PheromoneGradients>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for mut ant_movement in ants.iter_mut() {
        let closest_gradient = gradients.get(ant_movement.current_node.0).unwrap();

        let elapsed = time.elapsed_seconds();

        let random = rng.gen_range(0.0..1.0);
        // the gradient for the pheromon the ant follows is not null: follow its direction for at least a second
        let goal_gradient = closest_gradient.gradients[ant_movement.goal.job.follows() as usize];
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

pub fn update_ant_direction_randomly(mut ants: Query<&mut AntMovement>, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_seconds_f64();
    for mut ant_movement in ants.iter_mut() {
        if rng.gen_bool(dt) {
            ant_movement.direction = Vec3::new(
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
                rng.gen::<f32>() - 0.5,
            )
            .normalize();
        }
    }
}
