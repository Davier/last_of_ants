use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{control::KinematicCharacterController, render::RapierDebugRenderPlugin};
use last_of_ants::{
    components::{
        entities::{AntBundle, Player},
        nav_mesh::{debug_nav_mesh, NavNode},
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
                spawn_ants_on_navmesh.run_if(on_key_just_pressed(KeyCode::Space)),
                move_ants_on_mesh,
                debug_nav_mesh.run_if(toggle_on_key(KeyCode::N)),
                toggle_physics_debug,
                player_movement,
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
        "Press SPACE to spawn ants\n\
        Press N to show the navigation mesh\n\
        Press I to show the world inspector\n\
        Press P to show the physics debug view",
        default(),
    ));
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
        commands
            .spawn((
                AntBundle {
                    sprite: SpriteBundle {
                        transform,
                        ..default()
                    },
                    ..default()
                },
                MovementGoal(id),
            ))
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

fn player_movement(
    mut controllers: Query<&mut KinematicCharacterController, With<Player>>,
    inputs: Res<Input<KeyCode>>,
) {
    let mut delta_pos = Vec2::new(0., -2.);
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
    for mut controller in controllers.iter_mut() {
        controller.translation = Some(delta_pos);
    }
}
