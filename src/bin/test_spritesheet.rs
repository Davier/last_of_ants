use bevy::prelude::*;

use last_of_ants::render::player_animation::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites
            PlayerAnimationPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(PlayerAnimationBundle::new(
        &asset_server,
        &mut texture_atlases,
    ));
}

fn update(inputs: Res<Input<KeyCode>>, mut animations: Query<&mut PlayerAnimation>) {
    let mut animation = animations.single_mut();
    if inputs.just_pressed(KeyCode::Space) {
        animation.set_state(PlayerAnimationState::Jumping);
    } else if inputs.pressed(KeyCode::D) {
        if matches!(animation.state, PlayerAnimationState::Standing) {
            animation.set_state(PlayerAnimationState::Running);
            animation.is_facing_right = true;
        }
    } else if inputs.pressed(KeyCode::A) {
        if matches!(animation.state, PlayerAnimationState::Standing) {
            animation.set_state(PlayerAnimationState::Running);
            animation.is_facing_right = false;
        }
    } else if !matches!(animation.state, PlayerAnimationState::Jumping) {
        animation.set_state(PlayerAnimationState::Standing);
    }
}
