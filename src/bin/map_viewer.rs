use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::view::RenderLayers,
    window::PrimaryWindow,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use itertools::Itertools;
use last_of_ants::{
    components::{
        ants::{debug_ants, Ant, AntBundle},
        nav_mesh::{debug_nav_mesh, NavNode},
        pheromon::{Gradient, Pheromon, PheromonBuffer},
    },
    helpers::{on_key_just_pressed, toggle_on_key, toggle_physics_debug},
    GamePlugin,
};
use rand::{seq::IteratorRandom, Rng};

fn main() {
    App::new()
        .add_plugins((
            GamePlugin,
            WorldInspectorPlugin::default().run_if(toggle_on_key(KeyCode::I)),
            RapierDebugRenderPlugin::default().disabled(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_pheromons,
                spawn_ants_on_navmesh.run_if(on_key_just_pressed(KeyCode::Space)),
                // move_ants_on_mesh,
                debug_nav_mesh.run_if(toggle_on_key(KeyCode::N)),
                debug_ants.run_if(toggle_on_key(KeyCode::O)),
                toggle_physics_debug.run_if(on_key_just_pressed(KeyCode::P)),
                camera_movement,
                debug_pheromons.run_if(toggle_on_key(KeyCode::H)),
                pheromon_diffusion.run_if(toggle_on_key(KeyCode::H)),
                update_gradient.after(pheromon_diffusion),
                update_text_counters,
            ),
        )
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(500., 500., 500.),
            ..default()
        })
        .insert(RenderLayers::all());

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("Ant nest.ldtk"),
        ..Default::default()
    });

    commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "\
        Use WASD to move\n\
        Use A and E to zoom in and out\n\
        Press SPACE to spawn ants\n\
        Press N to show the navigation mesh\n\
        Press I to show the world inspector\n\
        Press P to show the physics debug view\n\
        Press H to show the pheromons then click left/right to add/sub\n\
        Press O to show the ants debug view\n",
                default(),
            ),
            TextSection::default(), // FPS counter
            TextSection::default(), // Ant counter
        ]))
        .insert(TextCounters);
}

#[derive(Debug, Component)]
struct TextCounters;

fn update_text_counters(
    mut texts: Query<&mut Text, With<TextCounters>>,
    diagnostics: Res<DiagnosticsStore>,
    ants: Query<(), With<Ant>>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(ema) = fps.smoothed() {
            texts.single_mut().sections[1].value = format!("FPS: {ema:.2}\n")
        }
    }
    let num_ants = ants.iter().len();
    texts.single_mut().sections[2].value = format!("Ants: {num_ants}\n");
}

fn init_pheromons(mut commands: Commands, nodes: Query<(Entity, &NavNode), Added<NavNode>>) {
    for (id, node) in nodes.iter() {
        commands
            .entity(id)
            .insert(Pheromon::default())
            .insert(PheromonBuffer::default())
            .insert(Gradient::default());
    }
}

fn spawn_ants_on_navmesh(
    mut commands: Commands,
    nav_nodes: Query<(Entity, &GlobalTransform, &NavNode)>,
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
        // let scale = rng.gen::<f32>() + 0.5;
        let scale = 1.; // TODO
        let speed = 40.;
        AntBundle::spawn_on_nav_node(
            &mut commands,
            direction,
            speed,
            scale,
            nav_node_entity,
            nav_node,
            nav_node_pos,
            entities_holder,
            entities_holder_pos,
        );
        // .insert(MovementGoal(_id));
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
    mut cameras: Query<&mut OrthographicProjection, With<Camera2d>>,
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
    // Zoom
    let mut camera_projection = cameras.single_mut();
    if inputs.pressed(KeyCode::Q) {
        camera_projection.scale = (camera_projection.scale - 0.05).max(0.5);
    }
    if inputs.pressed(KeyCode::E) {
        camera_projection.scale = (camera_projection.scale + 0.05).min(2.);
    }
}

fn pheromon_diffusion(
    mut query_nodes: Query<(Entity, &NavNode, &mut Pheromon), With<PheromonBuffer>>,
    mut query_pheromon_buffers: Query<&mut PheromonBuffer, With<Pheromon>>,
) {
    // % going to neighbours
    let diffusion_rate = 0.01;

    // Compute diffusion to neighbours
    for (_, node, ph) in query_nodes.iter() {
        let diffused = ph.0 * diffusion_rate;
        if diffused > 0.005 {
            let neighbors = node.neighbors();
            let diffused_per_neighbor = diffused / neighbors.len() as f32;

            for neighbor in neighbors {
                let mut ph_b_neighbor = query_pheromon_buffers.get_mut(neighbor).unwrap();
                ph_b_neighbor.0 += diffused_per_neighbor;
            }
        }
    }

    // Apply diffusion
    for (id, _, mut ph) in query_nodes.iter_mut() {
        let mut ph_b = query_pheromon_buffers.get_mut(id).unwrap();
        ph.0 = ph.0 * (1.0 - diffusion_rate) + ph_b.0;
        ph_b.0 = 0.;
    }
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
        let (n, s, e, w, b) = match node {
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
                0.,
            ),
            NavNode::HorizontalEdge {
                left, right, back, ..
            } => (
                0.0,
                0.0,
                query_pheromon.get(*right).unwrap().0,
                query_pheromon.get(*left).unwrap().0,
                query_pheromon.get(*back).unwrap().0,
            ),
            NavNode::VerticalEdge { up, down, back, .. } => (
                query_pheromon.get(*up).unwrap().0,
                query_pheromon.get(*down).unwrap().0,
                0.0,
                0.0,
                query_pheromon.get(*back).unwrap().0,
            ),
        };
        // TODO: back

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

        if buttons.pressed(MouseButton::Left) {
            closest.3 .0 += 1.;
        } else if buttons.just_pressed(MouseButton::Right) {
            closest.3 .0 = 0.0_f32.max(closest.3 .0 - 1.0);
        }
    }

    for (e, n, ph, g) in query_nodes.iter() {
        let t = query_transform.get(e).unwrap();
        gizmos.circle_2d(t.translation().xy(), ph.0, Color::PINK);
        gizmos.ray_2d(t.translation().xy(), g.0 * 2.0, Color::BLUE);
    }
}
