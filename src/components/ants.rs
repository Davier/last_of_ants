use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::render::player_animation::Explosion;

pub mod dead_ants;
pub mod goal;
pub mod job;
pub mod live_ants;
pub mod movement;
pub mod position;
pub mod zombants;
use self::{dead_ants::DeadAntBundle, live_ants::LiveAnt};

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct AntStyle {
    /// TODO: Scaled is only used for rendering for now
    pub scale: f32,
    pub color_primary: Color,
    pub color_primary_kind: AntColorKind,
    pub color_secondary: Color,
    pub color_secondary_kind: AntColorKind,
    pub animation_phase: f32,
}

/// Kind of color, used to give the player clues
#[derive(Debug, Clone, Copy, Reflect)]
pub enum AntColorKind {
    BLACK,
    RED,
    BROWN,
    GREEN,
    YELLOW,
    WHITE,
}

impl AntColorKind {
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        *[
            Self::BLACK,
            Self::RED,
            Self::BROWN,
            Self::GREEN,
            Self::YELLOW,
        ]
        .choose(rng)
        .unwrap()
    }

    /// Not all colors are a good match
    pub fn new_random_from_primary(rng: &mut ThreadRng, primary: &Self) -> Self {
        *match primary {
            AntColorKind::BLACK => [Self::RED, Self::BROWN, Self::GREEN].as_slice(),
            AntColorKind::RED => [Self::BLACK, Self::RED, Self::BROWN].as_slice(),
            AntColorKind::BROWN => [Self::BLACK, Self::RED, Self::BROWN].as_slice(),
            AntColorKind::GREEN => [Self::BLACK, Self::GREEN, Self::YELLOW].as_slice(),
            AntColorKind::YELLOW => [Self::BROWN, Self::GREEN, Self::YELLOW].as_slice(),
            AntColorKind::WHITE => [Self::WHITE].as_slice(),
        }
        .choose(rng)
        .unwrap()
    }

    pub fn generate_color(&self, rng: &mut ThreadRng) -> Color {
        let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.005; // TODO:cleanup
        match self {
            AntColorKind::BLACK => Color::rgb(shade.x, shade.x, shade.x),
            AntColorKind::RED => Color::rgb(0.1 - shade.x, 0.001 + shade.y, shade.y),
            AntColorKind::BROWN => {
                let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.03;
                Color::rgb(0.15 + shade.x, 0.04 + shade.y, 0.02 + shade.z)
            }
            AntColorKind::GREEN => Color::rgb(shade.x, 0.06 - shade.y, shade.x),
            AntColorKind::YELLOW => {
                let shade = Vec3::from(rng.gen::<[f32; 3]>()) * 0.2;
                Color::rgb(1. - shade.x, 0.6 - shade.y, 0.)
            }
            AntColorKind::WHITE => Color::WHITE,
        }
    }
}

pub fn ant_explosion_collision(
    mut commands: Commands,
    ants: Query<(Entity, &CollidingEntities, &Parent, &Transform, &AntStyle), With<LiveAnt>>,
    explosions: Query<(), With<Explosion>>,
) {
    for (ant, colliding_entities, parent, ant_transform, ant_style) in ants.iter() {
        for colliding_entity in colliding_entities.iter() {
            if explosions.contains(colliding_entity) {
                // Despawn ant
                commands.entity(parent.get()).remove_children(&[ant]);
                commands.entity(ant).despawn();
                // Spawn dead ant
                commands
                    .spawn(DeadAntBundle::new(*ant_transform, *ant_style))
                    .set_parent(parent.get());
            }
        }
    }
}
