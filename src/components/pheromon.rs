use bevy::prelude::{Component, Vec2};

#[derive(Component)]
pub struct Pheromon(pub f32);

impl Default for Pheromon {
    fn default() -> Self {
        Self(0.)
    }
}

#[derive(Component)]
pub struct Gradient(pub Vec2);

impl Default for Gradient {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}
