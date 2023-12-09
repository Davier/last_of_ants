//! A shader and a material that uses it.

use std::f32::consts::PI;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use last_of_ants::{
    components::ants::{Ant, AntColorKind, AntPositionKind},
    render::render_ant::{AntMaterial, AntMaterialPlugin},
    ANT_SIZE,
};
use rand::Rng;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins((
            DefaultPlugins,
            AntMaterialPlugin,
            FrameTimeDiagnosticsPlugin,
            // LogDiagnosticsPlugin::default(),
            // last_of_ants::helpers::render_world_inspector::RenderWorldInspectorPlugin::default(),
            // WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_forward)
        .run();
}

// Setup a simple 2d scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<AntMaterial>>,
) {
    // camera
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.2;
    commands.spawn(camera);

    let mut rng = rand::thread_rng();
    let mesh: Mesh2dHandle = meshes
        .add(Mesh::from(shape::Quad {
            size: ANT_SIZE,
            flip: false,
        }))
        .into();
    let material_side: Handle<AntMaterial> = materials.add(AntMaterial { is_side: 1 });
    let material_top: Handle<AntMaterial> = materials.add(AntMaterial { is_side: 0 });
    for _ in 0..10 {
        let angle = rng.gen::<f32>() * 2. * PI;
        let direction = Vec3::new(angle.cos(), angle.sin(), 0.);
        let is_side = rng.gen_bool(0.5);
        let scale = rng.gen::<f32>() + 0.5;
        let mut transform = Transform::from_xyz(
            30. * (rng.gen::<f32>() * 2. - 1.),
            30. * (rng.gen::<f32>() * 2. - 1.),
            0.,
        );
        let material = if is_side {
            transform.translation.z = 1.;
            material_side.clone()
        } else {
            material_top.clone()
        };
        // let color_primary_kind = AntColorKind::YELLOW;
        // let color_secondary_kind = AntColorKind::BLACK;
        let color_primary_kind = AntColorKind::new_random(&mut rng);
        let color_secondary_kind =
            AntColorKind::new_random_from_primary(&mut rng, &color_primary_kind);
        let color_primary = color_primary_kind.generate_color(&mut rng);
        let color_secondary = color_secondary_kind.generate_color(&mut rng);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone(),
                transform,
                material,
                ..default()
            },
            Ant {
                position_kind: if !is_side {
                    AntPositionKind::Background
                } else if rng.gen_bool(0.5) {
                    AntPositionKind::VerticalWall {
                        is_left_side: rng.gen_bool(0.5),
                    }
                } else {
                    AntPositionKind::HorizontalWall {
                        is_up_side: rng.gen_bool(0.5),
                    }
                },
                speed: 30.,
                direction,
                current_wall: (Entity::PLACEHOLDER, GlobalTransform::default()),
                scale,
                color_primary,
                color_primary_kind,
                color_secondary,
                color_secondary_kind,
                animation_phase: rng.gen::<f32>() * 2. * PI,
                goal: 0,
            },
        ));
    }
}

fn move_forward(
    mut ants: Query<(&mut Ant, &mut Transform), With<Handle<AntMaterial>>>,
    time: Res<Time>,
    inputs: Res<Input<KeyCode>>,
) {
    for (mut ant, mut transform) in ants.iter_mut() {
        let forward = ant.direction;
        if inputs.pressed(KeyCode::W) {
            transform.translation += forward * 50.0 * time.delta_seconds();
        } else if inputs.pressed(KeyCode::S) {
            transform.translation -= forward * 50.0 * time.delta_seconds();
        }
        let angle = if inputs.pressed(KeyCode::A) {
            1.
        } else if inputs.pressed(KeyCode::D) {
            -1.
        } else {
            0.
        } * PI
            / 8.
            * time.delta_seconds()
            * ant.speed
            / (2. * PI);
        // transform.rotate_local_z(angle * PI / 8. * time.delta_seconds());
        // let (sin, cos) = angle.sin_cos();
        ant.direction = Mat3::from_angle(angle) * ant.direction;
    }
}
