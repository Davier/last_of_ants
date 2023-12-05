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
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::new(0.0, -2000.0),
        //     ..Default::default()
        // })
        .add_plugins((
            GamePlugin,
            WorldInspectorPlugin::default().run_if(toggle_on_key(KeyCode::I)),
            RapierDebugRenderPlugin::default(),
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
    mut players: Query<(&mut KinematicCharacterController, &mut Velocity, &Player), With<Player>>,
    inputs: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let walk_speed = 160.;
    let fly_speed = 10.;
    let gravity = -9.81 * 10.0 * 16.; // FIXME: scale
    let dt = time.delta_seconds();
    let jump_impulse = 8. / dt;
    let max_sliding_velocity = 50.;

    for (mut controller, mut velocity, player) in &mut players {
        let is_on_ground = !player.on_ground.is_empty();
        let is_on_wall = !player.on_wall.is_empty();
        let space_pressed = inputs.pressed(KeyCode::Space);
        let left_pressed = inputs.pressed(KeyCode::A);
        let right_pressed = inputs.pressed(KeyCode::D);
        let up_pressed = inputs.pressed(KeyCode::W);
        let down_pressed = inputs.pressed(KeyCode::S);
        // info!("{:5?} | {:?}", is_on_ground, is_on_wall);

        // Reset velocity
        if is_on_ground {
            velocity.linvel.x = 0.;
            velocity.linvel.y = 0.;
        }
        if player.is_on_left_wall && velocity.linvel.x < 0. {
            velocity.linvel.x = 0.;
        }
        if player.is_on_right_wall && velocity.linvel.x > 0. {
            velocity.linvel.x = 0.;
        }

        // Apply gravity
        velocity.linvel.y += gravity * dt;
        if is_on_wall && !down_pressed && velocity.linvel.y < -max_sliding_velocity {
            // info!("Sliding");
            velocity.linvel.y = -max_sliding_velocity;
        }

        // Move from inputs
        if right_pressed && (is_on_ground) {
            velocity.linvel.x += walk_speed;
        };
        if left_pressed && (is_on_ground) {
            velocity.linvel.x -= walk_speed;
        };
        if space_pressed {
            if is_on_ground {
                velocity.linvel.y = jump_impulse;
            } else if player.is_on_left_wall && right_pressed && velocity.linvel.x < 0.01 {
                velocity.linvel.y = jump_impulse;
                velocity.linvel.x = jump_impulse / 3.;
                // info!("Wall jump left");
            } else if player.is_on_right_wall && left_pressed && velocity.linvel.x > -0.01 {
                velocity.linvel.y = jump_impulse;
                velocity.linvel.x = -jump_impulse / 3.;
                // info!("Wall jump right");
            }
        }

        let delta_position = velocity.linvel * dt;
        // dbg!(&delta_position);
        controller.translation = Some(delta_position);
    }
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
