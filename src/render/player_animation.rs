use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{PLAYER_SIZE, RENDERLAYER_PLAYER};

pub struct PlayerAnimationPlugin;
impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_player_animation, update_effect_animations));
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
        let texture_handle = asset_server.load("textures/owlet_spritesheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 8, 5, None, None);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let animation = PlayerAnimation {
            standing: AnimationIndices {
                first: 0,
                last: 0, // 3
                speed: 0.05,
            },
            walking: AnimationIndices {
                first: 8,
                last: 13,
                speed: 0.05,
            },
            running: AnimationIndices {
                first: 16,
                last: 21,
                speed: 0.05,
            },
            jumping: AnimationIndices {
                first: 24,
                last: 31,
                speed: 0.05,
            },
            attacking: AnimationIndices {
                first: 32,
                last: 35,
                speed: 0.05,
            },
            climbing: AnimationIndices {
                first: 34,
                last: 34,
                speed: 0.05,
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
                    custom_size: Some(PLAYER_SIZE),
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
    pub walking: AnimationIndices,
    pub running: AnimationIndices,
    pub jumping: AnimationIndices,
    pub attacking: AnimationIndices,
    pub climbing: AnimationIndices,
    pub state: PlayerAnimationState,
    pub index: usize,
    pub is_facing_right: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAnimationState {
    Standing,
    Walking,
    Running,
    Jumping,
    Attacking,
    Climbing,
}

impl PlayerAnimation {
    pub fn get_indices(&self) -> &AnimationIndices {
        match self.state {
            PlayerAnimationState::Standing => &self.standing,
            PlayerAnimationState::Walking => &self.walking,
            PlayerAnimationState::Running => &self.running,
            PlayerAnimationState::Jumping => &self.jumping,
            PlayerAnimationState::Attacking => &self.attacking,
            PlayerAnimationState::Climbing => &self.climbing,
        }
    }

    pub fn set_state(&mut self, state: PlayerAnimationState, timer: &mut AnimationTimer) {
        self.state = state;
        self.index = self.get_indices().first;
        timer.set_duration(Duration::from_secs_f32(self.get_indices().speed));
        timer.reset();
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
    for (mut animation, mut animation_timer, mut sprite) in &mut query {
        animation_timer.tick(time.delta());
        if animation_timer.just_finished() {
            if animation.index == animation.get_indices().last {
                if matches!(
                    animation.state,
                    PlayerAnimationState::Jumping | PlayerAnimationState::Attacking
                ) {
                    animation.set_state(PlayerAnimationState::Standing, &mut animation_timer);
                }
                animation.index = animation.get_indices().first
            } else {
                animation.index += 1;
            };
            sprite.index = animation.index;
            sprite.flip_x = !animation.is_facing_right;
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct EffectAnimation {
    pub indices: AnimationIndices,
}

#[derive(Bundle)]
pub struct EffectAnimationBundle {
    pub animation: EffectAnimation,
    pub timer: AnimationTimer,
    pub sprite: SpriteSheetBundle,
    pub render_layer: RenderLayers,
}

#[derive(Debug, Component)]
pub struct Explosion;

impl EffectAnimationBundle {
    pub fn new_explosion(
        transform: GlobalTransform,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let texture_handle = asset_server.load("textures/explosion_spritesheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(128.0, 114.0), 1, 15, None, None);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let animation = EffectAnimation {
            indices: AnimationIndices {
                first: 0,
                last: 14,
                speed: 0.01,
            },
        };
        Self {
            animation,
            timer: AnimationTimer(Timer::from_seconds(
                animation.indices.speed,
                TimerMode::Repeating,
            )),
            sprite: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite {
                    index: animation.indices.first,
                    custom_size: Some(PLAYER_SIZE * 2.),
                    ..default()
                },
                transform: transform.into(),
                ..default()
            },
            render_layer: RENDERLAYER_PLAYER,
        }
    }
}

pub fn update_effect_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &EffectAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        Option<&Parent>,
    )>,
) {
    for (entity, animation, mut timer, mut sprite, parent) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if sprite.index == animation.indices.last {
                if let Some(parent) = parent {
                    commands.entity(parent.get()).remove_children(&[entity]);
                }
                commands.entity(entity).despawn();
            } else {
                sprite.index += 1;
            };
        }
    }
}
