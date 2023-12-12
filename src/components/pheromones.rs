use bevy::prelude::*;

pub mod concentrations;
pub mod gradients;
pub mod sources;

#[derive(Default, Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum PheromoneKind {
    #[default]
    Default = 0,
    Storage = 1,
    Food = 2,
    Zombqueen = 3,
    Zombant = 4,
    DeadAnt = 5,
}
pub const N_PHEROMONE_KINDS: usize = 6;

#[derive(Resource, Reflect)]
pub struct PheromoneConfig {
    evaporation_rate: [f32; N_PHEROMONE_KINDS],
    diffusion_rate: [f32; N_PHEROMONE_KINDS],
    diffusion_floor: [f32; N_PHEROMONE_KINDS],
    concentration_floor: [f32; N_PHEROMONE_KINDS],
    pub color: [(Color, Color); N_PHEROMONE_KINDS],
    pub dead_ant_deposit: f32,
    pub zombant_deposit: f32,
    pub zombqueen_source: f32,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        use PheromoneKind::*;

        let mut config = Self {
            evaporation_rate: [0.001; N_PHEROMONE_KINDS],
            diffusion_rate: [0.0; N_PHEROMONE_KINDS],
            diffusion_floor: [0.001; N_PHEROMONE_KINDS],
            concentration_floor: [0.001; N_PHEROMONE_KINDS],
            color: [(Color::BLACK, Color::WHITE); N_PHEROMONE_KINDS],
            dead_ant_deposit: 1.0,
            zombant_deposit: 1.0,
            zombqueen_source: 40.0,
        };

        config.color[Default as usize] = (Color::PURPLE, Color::FUCHSIA);
        config.color[Food as usize] = (Color::DARK_GREEN, Color::LIME_GREEN);
        config.color[Storage as usize] = (Color::BLUE, Color::AZURE);
        config.color[Zombqueen as usize] = (Color::MAROON, Color::CRIMSON);
        config.color[Zombant as usize] = (Color::BEIGE, Color::DARK_GRAY);
        config.color[DeadAnt as usize] = (Color::BLACK, Color::GRAY);

        config.diffusion_rate[Default as usize] = 0.01;
        config.diffusion_rate[Storage as usize] = 0.06;
        config.diffusion_rate[Food as usize] = 0.06;
        config.diffusion_rate[Zombqueen as usize] = 0.06;

        config.evaporation_rate[DeadAnt as usize] = 0.05;
        config.diffusion_rate[DeadAnt as usize] = 0.01;

        config.evaporation_rate[Zombant as usize] = 0.1;
        config.diffusion_rate[Zombant as usize] = 0.01;

        config.evaporation_rate[Zombqueen as usize] = 0.01;
        config.diffusion_rate[Zombqueen as usize] = 0.9;
        config.diffusion_floor[Zombqueen as usize] = 0.0001;
        config.concentration_floor[Zombqueen as usize] = 0.0001;

        config
    }
}
