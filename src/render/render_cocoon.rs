use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::ANT_SIZE;

pub struct CocoonMaterialPlugin;
impl Plugin for CocoonMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((Material2dPlugin::<CocoonMaterial>::default(),));

        let mut materials = app.world.resource_mut::<Assets<CocoonMaterial>>();
        materials.insert(COCOON_MATERIAL, CocoonMaterial { is_clue: 0 });
        materials.insert(COCOON_MATERIAL_CLUE, CocoonMaterial { is_clue: 1 });
        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        meshes.insert(
            COCOON_MESH2D.0,
            Mesh::from(shape::Quad {
                size: ANT_SIZE,
                flip: false,
            }),
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Copy, Clone)]
#[uniform(0, CocoonMaterialUniform)]
pub struct CocoonMaterial {
    pub is_clue: u32,
}

#[derive(ShaderType)]
pub struct CocoonMaterialUniform {
    pub is_clue: u32,
    pub _padding: Vec3,
}

impl From<&CocoonMaterial> for CocoonMaterialUniform {
    fn from(value: &CocoonMaterial) -> Self {
        Self {
            is_clue: value.is_clue,
            _padding: Vec3::ZERO,
        }
    }
}

impl Material2d for CocoonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cocoon.wgsl".into()
    }
}

pub type CocoonMaterialBundle = MaterialMesh2dBundle<CocoonMaterial>;
pub const COCOON_MATERIAL: Handle<CocoonMaterial> = Handle::weak_from_u128(13511600823605874864);
pub const COCOON_MATERIAL_CLUE: Handle<CocoonMaterial> =
    Handle::weak_from_u128(1401247957767436212);
pub const COCOON_MESH2D: Mesh2dHandle = Mesh2dHandle(Handle::weak_from_u128(17407549215165429623));
