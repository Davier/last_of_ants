use bevy::prelude::*;

use crate::components::{
    ants::{goal::AntGoal, movement::position::AntPositionKind, zombants::ZombAntQueen},
    object::Object,
    pheromones::PheromoneKind,
};

pub mod direction;
pub mod position;

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
    pub fn reached_object(
        &mut self,
        /*commands: &mut Commands, FIXME breaks trait for `.chain` in lib */
        object_id: Entity,
        object: &mut Object,
    ) {
        match object.kind {
            PheromoneKind::Storage => self
                .goal
                .reached_storage_target(object, &mut self.direction),
            PheromoneKind::Food => {
                self.goal
                    .reached_food_target(object_id, object, &mut self.direction)
            } /*commands,*/
            _ => (),
        }
    }

    pub fn reached_zombqueen(&mut self, zombqueen: &mut ZombAntQueen) {
        self.goal.reached_zombqueen(&mut self.direction, zombqueen);
    }
}
