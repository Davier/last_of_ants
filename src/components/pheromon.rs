use bevy::prelude::{Component, Vec2};

pub const PH1: usize = 0;
pub const PH2: usize = 1;
pub const N_PH: usize = 2;
pub const PH_DIFFUSION_RATE: [f32;N_PH] = [0.0;N_PH];

#[derive(Component)]
pub struct PheromonsBuffer(pub [f32;N_PH]);

impl Default for PheromonsBuffer {
    fn default() -> Self {
        Self([0.0;N_PH])
    }
}

#[derive(Component)]
pub struct Pheromons(pub [f32;N_PH]);

impl Default for Pheromons {
    fn default() -> Self {
        Self([0.0;N_PH])
    }
}

#[derive(Component)]
pub struct Gradient(pub [Vec2;N_PH]);

impl Default for Gradient {
    fn default() -> Self {
        Self([Vec2::ZERO;N_PH])
    }
}
