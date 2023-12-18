//! A shader and a material that uses it.

use std::f32::consts::PI;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, sprite::MaterialMesh2dBundle};
use last_of_ants::{
    components::ants::{
        goal::AntGoal,
        live_ants::LiveAnt,
        movement::{position::AntPositionKind, AntMovement},
        AntColorKind, AntStyle,
    },
    helpers::on_key_just_pressed,
    render::render_ant::{
        AntMaterial, AntMaterialPlugin, ANT_MATERIAL_SIDE, ANT_MATERIAL_TOP, ANT_MESH2D,
    },
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
        .add_systems(
            Update,
            (
                move_forward,
                spawn.run_if(on_key_just_pressed(KeyCode::Space)),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.2;
    commands.spawn(camera);
}

fn spawn(mut commands: Commands) {
    let mut rng = rand::thread_rng();
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
            ANT_MATERIAL_SIDE
        } else {
            ANT_MATERIAL_TOP
        };
        let color_primary_kind = AntColorKind::BLACK;
        let color_secondary_kind = AntColorKind::BLACK;
        // let color_primary_kind = AntColorKind::new_random(&mut rng);
        // let color_secondary_kind =
        //     AntColorKind::new_random_from_primary(&mut rng, &color_primary_kind);
        let color_primary = color_primary_kind.generate_color(&mut rng);
        let color_secondary = color_secondary_kind.generate_color(&mut rng);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: ANT_MESH2D,
                transform,
                material,
                ..default()
            },
            LiveAnt {},
            AntMovement {
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
                current_node: (Entity::PLACEHOLDER, GlobalTransform::default()),
                goal: AntGoal::default(),
                last_direction_update: 0.,
            },
            AntStyle {
                scale,
                color_primary,
                color_primary_kind,
                color_secondary,
                color_secondary_kind,
                animation_phase: rng.gen::<f32>() * 2. * PI,
            },
        ));
    }
}

fn move_forward(
    mut ants: Query<(&mut AntMovement, &mut Transform), With<Handle<AntMaterial>>>,
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
