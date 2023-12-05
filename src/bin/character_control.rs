use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
// use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use last_of_ants::{
    components::{
        entities::{update_player_sensor, Player},
        nav_mesh::debug_nav_mesh,
    },
    helpers::{toggle_on_key, toggle_physics_debug},
    GamePlugin,
};

fn main() {
    App::new()
        .add_plugins((
            GamePlugin,
            WorldInspectorPlugin::default().run_if(toggle_on_key(KeyCode::I)),
            RapierDebugRenderPlugin::default().disabled(),
            // FramepacePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_nav_mesh.run_if(toggle_on_key(KeyCode::N)),
                toggle_physics_debug,
                attach_camera_to_player,
                player_movement.after(update_player_sensor),
            ),
        )
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("Ant nest.ldtk"),
        ..Default::default()
    });
}

fn player_movement(
    mut players: Query<
        (
            &mut KinematicCharacterController,
            &mut Velocity,
            &Player,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Player>,
    >,
    inputs: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut last_time_on_left_wall: Local<f32>,
    mut last_time_on_right_wall: Local<f32>,
    mut last_time_on_ground: Local<f32>,
    mut jump_available: Local<bool>,
) {
    let Ok((mut controller, mut velocity, player, controller_output)) = players.get_single_mut()
    else {
        return;
    };

    let mut walk_speed = 160.;
    let gravity = -9.81 * 10.0 * 16.; // FIXME: scale
    let dt = time.delta_seconds();
    let jump_impulse = 8. / dt;
    let max_sliding_velocity = 50.;
    let jump_tolerance = 0.1; // [s]

    let is_on_ground = !player.on_ground.is_empty();
    let is_on_wall = !player.on_wall.is_empty();
    if player.is_on_left_wall {
        *last_time_on_left_wall = time.elapsed_seconds();
    }
    let is_on_left_wall_recently =
        time.elapsed_seconds() - *last_time_on_left_wall < jump_tolerance;
    if player.is_on_right_wall {
        *last_time_on_right_wall = time.elapsed_seconds();
    }
    let is_on_right_wall_recently =
        time.elapsed_seconds() - *last_time_on_right_wall < jump_tolerance;
    if is_on_ground {
        *last_time_on_ground = time.elapsed_seconds();
    }
    let is_on_ground_recently = time.elapsed_seconds() - *last_time_on_ground < jump_tolerance;
    let is_on_ceiling = controller_output
        .map(|output| output.effective_translation.y < output.desired_translation.y)
        .unwrap_or(false);

    let space_pressed = inputs.pressed(KeyCode::Space);
    let left_pressed = inputs.pressed(KeyCode::A);
    let right_pressed = inputs.pressed(KeyCode::D);
    let up_pressed = inputs.pressed(KeyCode::W);
    let down_pressed = inputs.pressed(KeyCode::S);
    let shift_pressed = inputs.pressed(KeyCode::ShiftLeft);

    let v = &mut velocity.linvel;
    // info!("{:5?} | {:?}", is_on_ground, is_on_wall);

    // Slow down if shift is pressed
    if shift_pressed {
        walk_speed *= 0.5;
    }

    // Only allow one jump per pressed key
    if inputs.just_pressed(KeyCode::Space) {
        *jump_available = true;
    }

    // Reset velocity
    if is_on_ground {
        v.x = 0.;
        v.y = 0.;
    }
    if is_on_ceiling {
        v.y = 0.;
    }
    if player.is_on_left_wall && v.x < 0. {
        v.x = 0.;
    }
    if player.is_on_right_wall && v.x > 0. {
        v.x = 0.;
    }

    // Apply gravity
    v.y += gravity * dt;
    if is_on_wall && !down_pressed && v.y < -max_sliding_velocity {
        // info!("Sliding");
        v.y = -max_sliding_velocity;
    }

    // Move from inputs
    if right_pressed
        && (is_on_ground_recently || is_on_left_wall_recently || is_on_right_wall_recently)
    {
        v.x = walk_speed;
    };
    if left_pressed
        && (is_on_ground_recently || is_on_left_wall_recently || is_on_right_wall_recently)
    {
        v.x = -walk_speed;
    };
    if space_pressed && *jump_available {
        if is_on_ground {
            v.y = jump_impulse;
            *jump_available = false;
        } else if is_on_left_wall_recently && right_pressed {
            v.y = jump_impulse;
            v.x = jump_impulse / 3.;
            *jump_available = false;
            // info!("Wall jump left");
        } else if is_on_right_wall_recently && left_pressed {
            v.y = jump_impulse;
            v.x = -jump_impulse / 3.;
            *jump_available = false;
            // info!("Wall jump right");
        }
    }

    let delta_position = *v * dt;
    // dbg!(&delta_position);
    controller.translation = Some(delta_position);
}

fn attach_camera_to_player(
    mut commands: Commands,
    added_player: Query<Entity, Added<Player>>,
    camera: Query<Entity, With<Camera2d>>,
) {
    if let Ok(player) = added_player.get_single() {
        let camera = camera.single();
        commands.entity(camera).set_parent(player);
    }
}
