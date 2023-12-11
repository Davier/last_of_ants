use bevy::reflect::Reflect;

use crate::components::pheromones::PheromoneKind;

#[derive(Debug, Default, Clone, Copy, Reflect)]
pub enum Job {
    #[default]
    Food,
    Storage,
    Thief,
    Offering,
}

impl Job {
    pub fn follows(&self) -> PheromoneKind {
        match self {
            Job::Food => PheromoneKind::Food,
            Job::Storage => PheromoneKind::Storage,
            Job::Thief => PheromoneKind::Storage,
            Job::Offering => PheromoneKind::Zombqueen,
        }
    }

    pub fn next_job(&self) -> Job {
        match self {
            Job::Food => Job::Storage,
            Job::Storage => Job::Food,
            Job::Thief => Job::Offering,
            Job::Offering => Job::Thief,
        }
    }
}
