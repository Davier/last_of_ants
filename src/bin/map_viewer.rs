use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use itertools::Itertools;
use last_of_ants::{
    components::{
        entities::AntBundle,
        nav_mesh::{debug_nav_mesh, spawn_nav_mesh, NavNode},
        pheromon::{Gradient, Pheromon},
    },
    helpers::{on_key_just_pressed, toggle_on_key, toggle_physics_debug},
    GamePlugin,
};
use rand::seq::IteratorRandom;

fn main() {
    App::new()
        .add_plugins((
            GamePlugin,
            WorldInspectorPlugin::default().run_if(toggle_on_key(KeyCode::I)),
            RapierDebugRenderPlugin::default().disabled(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_pheromons,
                spawn_ants_on_navmesh.run_if(on_key_just_pressed(KeyCode::Space)),
                move_ants_on_mesh,
                debug_pheromons.run_if(toggle_on_key(KeyCode::H)),
                debug_nav_mesh.run_if(toggle_on_key(KeyCode::N)),
                toggle_physics_debug,
                camera_movement,
                update_gradient,
            ),
        )
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(500., 500., 0.),
        ..default()
    });

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("Ant nest.ldtk"),
        ..Default::default()
    });

    commands.spawn(TextBundle::from_section(
        "\
        Use WASD to move\n\
        Press SPACE to spawn ants\n\
        Press N to show the navigation mesh\n\
        Press I to show the world inspector\n\
        Press P to show the physics debug view\n\
        Press H to show the pheromons then click left/right to add/sub",
        default(),
    ));
}

fn init_pheromons(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands
            .entity(id)
            .insert(Pheromon::default())
            .insert(Gradient::default());
    }
}

fn spawn_ants_on_navmesh(
    mut commands: Commands,
    nav_nodes: Query<(Entity, &GlobalTransform), With<NavNode>>,
    level: Query<&Children, With<LevelIid>>,
    named_transform: Query<(Entity, &Name, &GlobalTransform)>,
) {
    let mut rng = rand::thread_rng();
    let level_children = level.single();
    let (entities_holder, _, entities_holder_pos) = level_children
        .iter()
        .filter_map(|child| named_transform.get(*child).ok())
        .find(|(_, name, _)| name.as_str() == "Entities")
        .unwrap();

    for _ in 0..5 {
        let Some((id, node_pos)) = nav_nodes.iter().choose(&mut rng) else {
            return;
        };

        let mut transform = node_pos.reparented_to(entities_holder_pos);
        transform.translation.z += 10.;
        let mut bundle = AntBundle::default();
        bundle.sprite.transform = transform;
        commands
            .spawn((bundle, MovementGoal(id)))
            .set_parent(entities_holder);
    }
}

// Simple movement system to test the navigation mesh
#[derive(Debug, Clone, Copy, Component, Reflect)]
struct MovementGoal(Entity);

fn move_ants_on_mesh(
    mut ants: Query<(&mut MovementGoal, &Parent, &mut Transform)>,
    mut transforms: Query<&GlobalTransform>,
    nav_nodes: Query<&NavNode>,
) {
    let mut rng = rand::thread_rng();
    for (mut goal, parent, mut pos_current) in ants.iter_mut() {
        let pos_goal = *transforms.get(goal.0).unwrap();
        let pos_parent = transforms.get_mut(parent.get()).unwrap();
        let pos_goal = pos_goal.reparented_to(pos_parent);

        let speed = 1.0;
        let pos_current_xy = pos_current.translation.xy();
        let pos_goal_xy = pos_goal.translation.xy();
        let distance = pos_current_xy.distance(pos_goal_xy);
        if distance < speed {
            let node = nav_nodes.get(goal.0).unwrap();
            let next_node = *node.neighbors().iter().choose(&mut rng).unwrap();
            goal.0 = next_node;
            continue;
        }

        let vector = (pos_goal_xy - pos_current_xy).normalize() * speed;
        pos_current.translation.x += vector.x;
        pos_current.translation.y += vector.y;
    }
}

fn camera_movement(
    mut transform: Query<&mut Transform, With<Camera2d>>,
    inputs: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let speed = 400.;
    let mut delta_pos = Vec2::ZERO;
    if inputs.pressed(KeyCode::W) {
        delta_pos.y += 1.;
    }
    if inputs.pressed(KeyCode::A) {
        delta_pos.x -= 1.;
    }
    if inputs.pressed(KeyCode::S) {
        delta_pos.y -= 1.;
    }
    if inputs.pressed(KeyCode::D) {
        delta_pos.x += 1.;
    }
    transform.single_mut().translation.x += delta_pos.x * speed * dt;
    transform.single_mut().translation.y += delta_pos.y * speed * dt;
}

fn update_gradient(
    mut query_changed_nodes: Query<(Entity, &NavNode, &Pheromon, &mut Gradient)>,
    query_pheromon: Query<&Pheromon, With<NavNode>>,
) {
    let vn = Vec2::new(0.0, 1.0);
    let vs = Vec2::new(0.0, -1.0);
    let ve = Vec2::new(1.0, 0.0);
    let vw = Vec2::new(-1.0, 0.0);

    for (id, node, ph, mut gd) in query_changed_nodes.iter_mut() {
        let (n, s, e, w) = match node {
            NavNode::Background {
                up,
                left,
                down,
                right,
            } => (
                query_pheromon.get(*up).unwrap().0,
                query_pheromon.get(*down).unwrap().0,
                query_pheromon.get(*right).unwrap().0,
                query_pheromon.get(*left).unwrap().0,
            ),
            NavNode::HorizontalEdge {
                left,
                left_kind,
                right,
                right_kind,
                back,
                is_up_side,
            } => (
                0.0,
                0.0,
                query_pheromon.get(*right).unwrap().0,
                query_pheromon.get(*left).unwrap().0,
            ),
            NavNode::VerticalEdge {
                up,
                up_kind,
                down,
                down_kind,
                back,
                is_left_side,
            } => (
                query_pheromon.get(*up).unwrap().0,
                query_pheromon.get(*down).unwrap().0,
                0.0,
                0.0,
            ),
        };

        if ph.0 >= n.max(s).max(e).max(w) {
            gd.0 = Vec2::ZERO;
        } else {
            gd.0 = n * vn + s * vs + e * ve + w * vw;
        }
    }
}

fn debug_pheromons(
    mut query_nodes: Query<(Entity, &NavNode, &mut Pheromon, &Gradient)>,
    query_transform: Query<&GlobalTransform, With<NavNode>>,
    mut gizmos: Gizmos,

    buttons: Res<Input<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    if let Some(cursor_world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        let mut distances = query_nodes
            .iter_mut()
            .map(|(id, _, ph, gd)| {
                let pos = query_transform.get(id).unwrap().translation().xy();
                (id, pos, pos.distance(cursor_world_position), ph, gd)
            })
            .collect_vec();

        distances.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        gizmos.circle_2d(distances[1].1, 0.5, Color::RED);
        gizmos.circle_2d(distances[2].1, 0.5, Color::RED);
        gizmos.circle_2d(distances[3].1, 0.5, Color::RED);
        let closest = &mut distances[0];

        gizmos.circle_2d(closest.1, 0.5, Color::RED);
        gizmos.ray_2d(cursor_world_position, closest.4 .0, Color::ALICE_BLUE);

        if buttons.just_pressed(MouseButton::Left) {
            closest.3 .0 += 1.;
        } else if buttons.just_pressed(MouseButton::Right) {
            closest.3 .0 = 0.0_f32.max(closest.3 .0 - 1.0);
        }
    }

    for (e, n, ph, g) in query_nodes.iter() {
        let t = query_transform.get(e).unwrap();
        gizmos.circle_2d(t.translation().xy(), ph.0, Color::PINK);
        gizmos.ray_2d(t.translation().xy(), g.0 * 2.0, Color::BLUE);
        debug!("{}", t.translation());
    }
}
