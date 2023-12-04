use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterController,
    geometry::Collider,
};

/// Attach sprites to the newly spawned entities
pub fn setup_entity_sprites(
    mut added_players: Query<&mut Sprite, (Added<Player>, Without<Ant>)>,
    mut added_ants: Query<&mut Sprite, (Added<Ant>, Without<Player>)>,
) {
    for mut player_sprite in added_players.iter_mut() {
        player_sprite.color = Color::GREEN;
        player_sprite.custom_size = Some(Vec2::new(8., 16.));
    }
    for mut ant_sprite in added_ants.iter_mut() {
        ant_sprite.color = Color::BLACK;
        ant_sprite.custom_size = Some(Vec2::new(8., 8.));
    }
}

#[derive(Bundle, Clone, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    sprite: SpriteBundle,
    // rigid_body: RigidBody,
    controller: KinematicCharacterController,
    collider: Collider,
    // velocity: Velocity,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Default::default(),
            sprite: Default::default(),
            // rigid_body: RigidBody::KinematicPositionBased,
            controller: Default::default(),
            collider: Collider::cuboid(4., 8.),
            // velocity: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Default, Component, Reflect)]
pub struct Player;

#[derive(Bundle, Clone, Default, LdtkEntity)]
pub struct AntBundle {
    pub ant: Ant,
    pub sprite: SpriteBundle,
}

#[derive(Clone, Copy, Default, Component, Reflect)]
pub struct Ant;
