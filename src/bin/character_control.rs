use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_ecs_ldtk::prelude::*;
// use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use last_of_ants::{
    components::{
        ants::{debug_ants, goal::AntGoal, job::Job, AntColorKind, LiveAntBundle},
        nav_mesh::{debug_nav_mesh, NavNode},
        player::{update_player_sensor, Player},
        zombants::{spawn_zombant_queen, ZombAntBundle},
    },
    helpers::{on_key_just_pressed, run_after, toggle_on_key, toggle_physics_debug},
    render::{
        player_animation::{
            AnimationTimer, EffectAnimationBundle, Explosion, PlayerAnimation, PlayerAnimationState,
        },
        MainCamera2d, MainCamera2dBundle,
    },
    ui::ui_clues::UiCluesPlugin,
    AppState, GamePlugin, COLLISION_GROUP_ANTS, COLLISION_GROUP_EXPLOSION, PLAYER_SIZE, TILE_SIZE,
};
use rand::{seq::IteratorRandom, Rng};

fn main() {
    App::new()
        .add_plugins((
            GamePlugin,
            UiCluesPlugin,
            WorldInspectorPlugin::default().run_if(toggle_on_key(KeyCode::I)),
            RapierDebugRenderPlugin::default().disabled(),
            // FramepacePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_nav_mesh.run_if(toggle_on_key(KeyCode::N)),
                debug_ants.run_if(toggle_on_key(KeyCode::O)),
                toggle_physics_debug.run_if(on_key_just_pressed(KeyCode::P)),
                player_movement.after(update_player_sensor),
                //spawn_ants_on_navmesh.run_if(run_after(10)), // FIXME
                spawn_explosions,
            ),
        )
        .add_systems(
            OnEnter(AppState::ProcessingOthers),
            (spawn_ants_on_navmesh, attach_camera_to_player),
        )
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = MainCamera2dBundle::default();
    camera.camera.projection.scale = 0.5;
    commands.spawn(camera);
}

fn player_movement(
    mut players: Query<
        (
            &mut KinematicCharacterController,
            &mut Velocity,
            &mut Player,
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
    mut animations: Query<(&mut PlayerAnimation, &mut AnimationTimer)>,
) {
    let Ok((mut controller, mut velocity, mut player, controller_output)) =
        players.get_single_mut()
    else {
        return;
    };

    let mut walk_speed = 10.0 * TILE_SIZE;
    let gravity = -9.81 * 10.0 * TILE_SIZE;
    let dt = time.delta_seconds();
    let mut jump_impulse = 0.5 * TILE_SIZE / dt;
    let max_sliding_velocity = 4. * TILE_SIZE;
    let jump_tolerance = 0.1; // [s]

    // let is_on_ground = !player.on_ground.is_empty();
    let is_on_ground = controller_output
        .map(|output| output.grounded)
        .unwrap_or(true);
    player.is_grounded = is_on_ground; // FIXME: renames
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
    let _left_pressed = inputs.pressed(KeyCode::A) || inputs.pressed(KeyCode::Left);
    let _right_pressed = inputs.pressed(KeyCode::D) || inputs.pressed(KeyCode::Right);
    let left_pressed = _left_pressed && !_right_pressed;
    let right_pressed = _right_pressed && !_left_pressed;
    let up_pressed = inputs.pressed(KeyCode::W) || inputs.pressed(KeyCode::Up);
    let down_pressed = inputs.pressed(KeyCode::S) || inputs.pressed(KeyCode::Down);
    let shift_pressed = inputs.pressed(KeyCode::ShiftLeft);

    let v = &mut velocity.linvel;
    // info!("{:5?} | {:?}", is_on_ground, is_on_wall);

    let (mut animation, mut animation_timer) = animations.single_mut();
    // let mut animation_moving_state = PlayerAnimationState::Running;

    // Slow down if shift is pressed
    if shift_pressed {
        walk_speed *= 0.4;
        jump_impulse *= 0.8;
        // animation_moving_state = PlayerAnimationState::Walking; // FIXME: animation speed not working
        player.is_crouching = true;
    } else {
        player.is_crouching = false;
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

    // Climbing animation
    if is_on_left_wall_recently
        && !is_on_ground_recently
        && !matches!(animation.state, PlayerAnimationState::Climbing)
    {
        animation.set_state(PlayerAnimationState::Climbing, &mut animation_timer);
        animation.is_facing_right = false;
    }
    if is_on_right_wall_recently
        && !is_on_ground_recently
        && !matches!(animation.state, PlayerAnimationState::Climbing)
    {
        animation.set_state(PlayerAnimationState::Climbing, &mut animation_timer);
        animation.is_facing_right = true;
    }
    if (!is_on_wall || is_on_ground_recently)
        && matches!(animation.state, PlayerAnimationState::Climbing)
    {
        animation.set_state(PlayerAnimationState::Standing, &mut animation_timer);
    }
    // Move from inputs
    if right_pressed
        && (is_on_ground_recently || is_on_left_wall_recently || is_on_right_wall_recently)
    {
        v.x = walk_speed;
        animation.is_facing_right = true;
        if matches!(animation.state, PlayerAnimationState::Standing) {
            animation.set_state(PlayerAnimationState::Running, &mut animation_timer);
        }
    };
    if left_pressed
        && (is_on_ground_recently || is_on_left_wall_recently || is_on_right_wall_recently)
    {
        v.x = -walk_speed;
        animation.is_facing_right = false;
        if matches!(animation.state, PlayerAnimationState::Standing) {
            animation.set_state(PlayerAnimationState::Running, &mut animation_timer);
        }
    };
    if !left_pressed && !right_pressed && matches!(animation.state, PlayerAnimationState::Running) {
        animation.set_state(PlayerAnimationState::Standing, &mut animation_timer);
    }
    if space_pressed && *jump_available {
        if is_on_ground {
            v.y = jump_impulse;
            *jump_available = false;
            animation.set_state(PlayerAnimationState::Jumping, &mut animation_timer);
        } else if is_on_left_wall_recently && right_pressed {
            v.y = jump_impulse;
            v.x = jump_impulse / 3.;
            *jump_available = false;
            animation.set_state(PlayerAnimationState::Jumping, &mut animation_timer);
            // info!("Wall jump left");
        } else if is_on_right_wall_recently && left_pressed {
            v.y = jump_impulse;
            v.x = -jump_impulse / 3.;
            *jump_available = false;
            animation.set_state(PlayerAnimationState::Jumping, &mut animation_timer);
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
    camera: Query<Entity, With<MainCamera2d>>,
) {
    if let Ok(player) = added_player.get_single() {
        let camera = camera.single();
        commands.entity(camera).set_parent(player);
    }
}

fn spawn_ants_on_navmesh(
    mut commands: Commands,
    // mut level_events: EventReader<LevelEvent>,
    nav_nodes: Query<(Entity, &GlobalTransform, &NavNode)>,
    level: Query<(&LevelIid, &Children)>,
    named_transform: Query<(Entity, &Name, &GlobalTransform)>,
) {
    // for level_event in level_events.read() {
    //     let LevelEvent::Transformed(level_iid) = level_event else {
    //         continue;
    //     };
    let mut rng = rand::thread_rng();
    // let level_children = level.iter().find(|(iid, _)| *iid == level_iid).unwrap().1;
    let level_children = level.single().1;
    let (entities_holder, _, entities_holder_pos) = level_children
        .iter()
        .filter_map(|child| named_transform.get(*child).ok())
        .find(|(_, name, _)| name.as_str() == "Entities")
        .unwrap();

    for _ in 0..100 {
        let Some((nav_node_entity, nav_node_pos, nav_node)) = nav_nodes.iter().choose(&mut rng)
        else {
            return;
        };

        let direction = Vec3::new(
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
        )
        .normalize();
        let color_primary_kind = AntColorKind::new_random(&mut rng);
        let color_secondary_kind =
            AntColorKind::new_random_from_primary(&mut rng, &color_primary_kind);
        let speed = 40.;
        // let scale = rng.gen::<f32>() + 0.5;
        let scale = 1.; // TODO
        LiveAntBundle::spawn_on_nav_node(
            &mut commands,
            direction,
            speed,
            scale,
            color_primary_kind,
            color_secondary_kind,
            nav_node_entity,
            nav_node,
            nav_node_pos,
            entities_holder,
            entities_holder_pos,
            &mut rng,
            AntGoal {
                job: Job::Food,
                holds: 0.,
            },
        );
    }

    for _ in 0..20 {
        let Some((nav_node_entity, nav_node_pos, nav_node)) = nav_nodes.iter().choose(&mut rng)
        else {
            return;
        };

        let direction = Vec3::new(
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
            rng.gen::<f32>() - 0.5,
        )
        .normalize();
        let color_primary_kind = AntColorKind::new_random(&mut rng);
        let color_secondary_kind =
            AntColorKind::new_random_from_primary(&mut rng, &color_primary_kind);
        let speed = 40.;
        // let scale = rng.gen::<f32>() + 0.5;
        let scale = 1.; // TODO
        ZombAntBundle::spawn_on_nav_node(
            &mut commands,
            direction,
            speed,
            scale,
            color_primary_kind,
            color_secondary_kind,
            nav_node_entity,
            nav_node,
            nav_node_pos,
            entities_holder,
            entities_holder_pos,
            &mut rng,
            AntGoal {
                job: Job::Thief,
                holds: 0.,
            },
        );
    }
    // }
}

pub fn spawn_explosions(
    mut commands: Commands,
    mut mouse_events: EventReader<MouseButtonInput>,
    mut player: Query<(
        &Player,
        &GlobalTransform,
        &mut PlayerAnimation,
        &mut AnimationTimer,
    )>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for mouse_event in mouse_events.read() {
        let MouseButtonInput {
            state: ButtonState::Pressed,
            button: MouseButton::Left,
            window: _,
        } = mouse_event
        else {
            return;
        };
        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };
        let Ok((camera, camera_transform)) = cameras.get_single() else {
            return;
        };
        let Some(cursor_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        else {
            return;
        };

        let Ok((player, player_transform, mut player_animation, mut animation_timer)) =
            player.get_single_mut()
        else {
            return;
        };
        if !player.is_grounded {
            return;
        }

        let player_position = player_transform.translation().xy();

        let direction = (cursor_position - player_position).normalize_or_zero();
        let reach = 30.;
        let effect_position = player_position + direction * reach;
        let effect_transform = GlobalTransform::from_xyz(effect_position.x, effect_position.y, 10.);
        commands.spawn((
            EffectAnimationBundle::new_explosion(
                effect_transform,
                &asset_server,
                &mut texture_atlases,
            ),
            Collider::ball(PLAYER_SIZE.y),
            CollisionGroups::new(COLLISION_GROUP_EXPLOSION, COLLISION_GROUP_ANTS),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::STATIC_STATIC,
            Explosion,
        ));
        player_animation.set_state(PlayerAnimationState::Attacking, &mut animation_timer);
    }
}
