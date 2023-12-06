//! A shader and a material that uses it.

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_forward)
        .run();
}

// Setup a simple 2d scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_xyz(100., 0., 0.).with_scale(Vec3::splat(128.)),
        material: materials.add(CustomMaterial { color: Color::BLUE }),
        ..default()
    });
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ant.wgsl".into()
    }
    // TODO: specialize on top/side view
    // TODO: add instance buffer for customization: orientation
}

fn move_forward(
    mut ants: Query<&mut Transform, With<Handle<CustomMaterial>>>,
    time: Res<Time>,
    inputs: Res<Input<KeyCode>>,
) {
    for mut transform in ants.iter_mut() {
        let forward = dbg!(transform.local_x());
        if inputs.pressed(KeyCode::W) {
            transform.translation += forward * 50.0 * time.delta_seconds();
        }
        let angle = if inputs.pressed(KeyCode::A) {
            1.
        } else if inputs.pressed(KeyCode::D) {
            -1.
        } else {
            0.
        };
        transform.rotate_local_z(angle * PI / 8. * time.delta_seconds());
        dbg!(transform.translation);
    }
}
