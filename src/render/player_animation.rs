use std::time::Duration;

use bevy::prelude::*;

use crate::PLAYER_SIZE;

pub struct PlayerAnimationPlugin;
impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_player_animation);
    }
}

#[derive(Bundle)]
pub struct PlayerAnimationBundle {
    pub animation: PlayerAnimation,
    pub timer: AnimationTimer,
    pub sprite: SpriteSheetBundle,
}

impl PlayerAnimationBundle {
    pub fn new(asset_server: &AssetServer, texture_atlases: &mut Assets<TextureAtlas>) -> Self {
        let texture_handle = asset_server.load("textures/Robot.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(64.0, 80.0),
            7,
            6,
            Some(Vec2::new(16., 0.)),
            Some(Vec2::new(8., 0.)),
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let animation = PlayerAnimation {
            standing: AnimationIndices {
                first: 0,
                last: 0,
                speed: 0.05,
            },
            running: AnimationIndices {
                first: 7,
                last: 12,
                speed: 0.09,
            },
            jumping: AnimationIndices {
                first: 14,
                last: 20,
                speed: 0.15,
            },
            state: PlayerAnimationState::Standing,
            index: 0,
            is_facing_right: true,
        };
        Self {
            animation,
            timer: AnimationTimer(Timer::from_seconds(
                animation.get_indices().speed,
                TimerMode::Repeating,
            )),
            sprite: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite {
                    index: animation.standing.first,
                    custom_size: Some(PLAYER_SIZE * 1.25),
                    ..default()
                },
                transform: Transform::from_scale(Vec3::splat(1.)),
                ..default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct PlayerAnimation {
    pub standing: AnimationIndices,
    pub running: AnimationIndices,
    pub jumping: AnimationIndices,
    pub state: PlayerAnimationState,
    pub index: usize,
    pub is_facing_right: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAnimationState {
    Standing,
    Running,
    Jumping,
}

impl PlayerAnimation {
    pub fn get_indices(&self) -> &AnimationIndices {
        match self.state {
            PlayerAnimationState::Standing => &self.standing,
            PlayerAnimationState::Running => &self.running,
            PlayerAnimationState::Jumping => &self.jumping,
        }
    }

    pub fn set_state(&mut self, state: PlayerAnimationState) {
        self.state = state;
        self.index = self.get_indices().first;
    }
}
#[derive(Debug, Clone, Copy)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub speed: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

fn update_player_animation(
    time: Res<Time>,
    mut query: Query<(
        &mut PlayerAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut animation, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if animation.index == animation.get_indices().last {
                if matches!(animation.state, PlayerAnimationState::Jumping) {
                    animation.set_state(PlayerAnimationState::Standing);
                }
                timer.set_duration(Duration::from_secs_f32(animation.get_indices().speed)); // FIXME: does nothing?
                animation.index = animation.get_indices().first
            } else {
                animation.index += 1;
            };
            sprite.index = animation.index;
            sprite.flip_x = !animation.is_facing_right;
        }
    }
}
