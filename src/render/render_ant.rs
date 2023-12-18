//! A material that draws ants

use std::{f32::consts::PI, mem::size_of, ops::DerefMut};

use crate::{
    components::ants::{
        movement::{position::AntPositionKind, AntMovement},
        AntStyle,
    },
    render::render_cocoon::CocoonMaterial,
    ANT_SIZE,
};
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    math::Mat3A,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::GpuBufferInfo,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctionId, DrawFunctions, PhaseItem, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            AsBindGroup, BufferUsages, BufferVec, ShaderRef, ShaderType,
            SpecializedMeshPipelineError, VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{
        extract_mesh2d, Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2d, Mesh2dHandle,
        Mesh2dUniform, RenderMesh2dInstance, RenderMesh2dInstances, SetMaterial2dBindGroup,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
    utils::nonmax::NonMaxU32,
};
use bytemuck::{Pod, Zeroable};

pub struct AntMaterialPlugin;
impl Plugin for AntMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<AntMaterial>::default(),
            ExtractComponentPlugin::<RenderAnt>::default(),
        ));

        let mut materials = app.world.resource_mut::<Assets<AntMaterial>>();
        materials.insert(ANT_MATERIAL_SIDE, AntMaterial::new(true, false, false));
        materials.insert(ANT_MATERIAL_TOP, AntMaterial::new(false, false, false));
        materials.insert(ANT_MATERIAL_DEAD, AntMaterial::new(false, true, false));
        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        meshes.insert(
            ANT_MESH2D.0,
            Mesh::from(shape::Quad {
                size: ANT_SIZE,
                flip: false,
            }),
        );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<AntMaterialMeta>()
                .add_render_command::<Transparent2d, DrawAntCommands>()
                .add_systems(
                    ExtractSchedule,
                    extract_ants_render_transform.after(extract_mesh2d),
                )
                .add_systems(Render, (prepare_ant_material.in_set(RenderSet::Prepare),));
        }
    }
}

pub type AntMaterialBundle = MaterialMesh2dBundle<AntMaterial>;
pub const ANT_MATERIAL_SIDE: Handle<AntMaterial> = Handle::weak_from_u128(12261044474578958661);
pub const ANT_MATERIAL_TOP: Handle<AntMaterial> = Handle::weak_from_u128(6041464017875828972);
pub const ANT_MATERIAL_DEAD: Handle<AntMaterial> = Handle::weak_from_u128(17744549271943886986);
pub const ANT_MESH2D: Mesh2dHandle = Mesh2dHandle(Handle::weak_from_u128(17147126180050932214));

#[derive(Asset, TypePath, AsBindGroup, Debug, Copy, Clone)]
#[uniform(0, AntMaterialUniform)]
pub struct AntMaterial {
    pub flags: u32,
}

#[derive(ShaderType)]
pub struct AntMaterialUniform {
    pub flags: u32,
    pub _padding: Vec3,
}

impl From<&AntMaterial> for AntMaterialUniform {
    fn from(value: &AntMaterial) -> Self {
        Self {
            flags: value.flags,
            _padding: Vec3::ZERO,
        }
    }
}

const ANT_MATERIAL_FLAG_IS_SIDE: u32 = 0b0001;
const ANT_MATERIAL_FLAG_IS_DEAD: u32 = 0b0010;
const ANT_MATERIAL_FLAG_HAS_HALO: u32 = 0b0100;

impl AntMaterial {
    pub fn new(is_side: bool, is_dead: bool, has_halo: bool) -> AntMaterial {
        let mut flags = 0;
        if is_side {
            flags |= ANT_MATERIAL_FLAG_IS_SIDE;
        }
        if is_dead {
            flags |= ANT_MATERIAL_FLAG_IS_DEAD;
        }
        if has_halo {
            flags |= ANT_MATERIAL_FLAG_HAS_HALO;
        }
        Self { flags }
    }
}

impl Material2d for AntMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ant.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/ant.wgsl".into()
    }

    fn draw_function_id<P: PhaseItem>(draw_functions: &DrawFunctions<P>) -> DrawFunctionId {
        draw_functions.read().id::<DrawAntCommands>()
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Instance vertex buffer
        let mut instance_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            vec![
                // color_primary
                VertexFormat::Float32x4,
                // color_secondary
                VertexFormat::Float32x4,
                // phase
                VertexFormat::Float32,
                // padding
                VertexFormat::Float32x3,
            ],
        );
        // The mesh pipeline uses location 0 through 4
        for (i, attribute) in instance_layout.attributes.iter_mut().enumerate() {
            attribute.shader_location = 5 + i as u32;
        }
        descriptor.vertex.buffers.push(instance_layout);

        // TODO: specialize on top/side view, remove useless data?
        Ok(())
    }
}

#[derive(Resource)]
pub struct AntMaterialMeta {
    instances: BufferVec<AntMaterialInstance>,
}

impl Default for AntMaterialMeta {
    fn default() -> Self {
        Self {
            instances: BufferVec::new(BufferUsages::VERTEX),
        }
    }
}

// TODO: get rid of RenderAnt
#[derive(Debug, Component)]
pub struct RenderAnt {
    instance: AntMaterialInstance,
    index: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Component)]
pub struct AntMaterialInstance {
    color_primary: Vec4,
    color_secondary: Vec4,
    animation_phase: f32,
    _padding: Vec3,
}

impl Default for AntMaterialInstance {
    fn default() -> Self {
        Self {
            color_primary: Color::WHITE.into(),
            color_secondary: Color::PURPLE.into(),
            animation_phase: 0.,
            _padding: Default::default(),
        }
    }
}

impl ExtractComponent for RenderAnt {
    type Query = Read<AntStyle>;

    type Filter = ();

    type Out = Self;

    fn extract_component(
        ant_style: bevy::ecs::query::QueryItem<'_, Self::Query>,
    ) -> Option<Self::Out> {
        Some(Self {
            instance: AntMaterialInstance {
                color_primary: ant_style.color_primary.into(),
                color_secondary: ant_style.color_secondary.into(),
                animation_phase: ant_style.animation_phase,
                _padding: Vec3::ZERO,
            },
            index: u64::MAX,
        })
    }
}

pub fn prepare_ant_material(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    phases: Query<&mut RenderPhase<Transparent2d>>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut ant_material_meta: ResMut<AntMaterialMeta>,
    mut ants: Query<&mut RenderAnt>,
    meshes: Query<(), With<Mesh2d>>,
) {
    for phase in phases.iter() {
        for item in phase.items.iter() {
            if !meshes.contains(item.entity) {
                continue;
            }
            if item.draw_function == AntMaterial::draw_function_id(&draw_functions) {
                let mut ant = ants.get_mut(item.entity).unwrap();
                ant.index = ant_material_meta.instances.push(ant.instance) as u64;
            } else if item.draw_function == CocoonMaterial::draw_function_id(&draw_functions) {
                // All other Material2d share the same vertex buffer 0, so we need to add a
                // placeholder entry in the instance buffer...
                // TODO: is there another way?
                ant_material_meta
                    .instances
                    .push(AntMaterialInstance::default());
            }
        }
    }
    ant_material_meta
        .instances
        .write_buffer(&render_device, &render_queue);
    ant_material_meta.instances.clear();
}

pub fn extract_ants_render_transform(
    mut ants: Extract<Query<(Entity, &AntMovement, &ViewVisibility)>>,
    mut meshes: ResMut<RenderMesh2dInstances>,
) {
    let meshes = meshes.deref_mut().deref_mut();
    for (entity, ant, view_visibility) in ants.iter_mut() {
        if !view_visibility.get() {
            continue;
        }
        let mesh2d = &mut meshes.get_mut(&entity).unwrap().transforms;
        mesh2d.transform.matrix3 = match ant.position_kind {
            AntPositionKind::Background => {
                let angle = ant.direction.y.atan2(ant.direction.x);
                Mat3A::from_angle(angle - PI / 2.).into()
            }
            AntPositionKind::VerticalWall { is_left_side } => {
                let mut scale = Vec2::ONE;
                if ant.direction.y < 0. {
                    scale.y = -1.;
                }
                if !is_left_side {
                    scale.x = -1.;
                }
                (Mat3A::from_scale(scale) * Mat3A::from_angle(-PI / 2.)).into()
            }
            AntPositionKind::HorizontalWall { is_up_side } => {
                let mut scale = Vec2::ONE;
                if ant.direction.x > 0. {
                    scale.x = -1.;
                }
                if is_up_side {
                    scale.y = -1.;
                }
                Mat3A::from_scale(scale).into()
            }
        };
    }
}

pub type DrawAntCommands = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMaterial2dBindGroup<AntMaterial, 1>,
    SetMesh2dBindGroup<2>,
    DrawAnt,
);

/// Copied from [bevy::sprite::DrawMesh2d], adding an instance vertex buffer
pub struct DrawAnt;
impl<P: PhaseItem> RenderCommand<P> for DrawAnt {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SRes<RenderMesh2dInstances>,
        SRes<AntMaterialMeta>,
    );
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<RenderAnt>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        ant: &RenderAnt,
        (meshes, render_mesh2d_instances, ant_material_meta): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let meshes = meshes.into_inner();
        let render_mesh2d_instances = render_mesh2d_instances.into_inner();

        let Some(RenderMesh2dInstance { mesh_asset_id, .. }) =
            render_mesh2d_instances.get(&item.entity())
        else {
            return RenderCommandResult::Failure;
        };
        let Some(gpu_mesh) = meshes.get(*mesh_asset_id) else {
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

        // In addition to the normal mesh2d drawing, we have a second vertex buffer
        // On WASM, the mesh instance buffer is chunked, so we need to slice our buffer accordingly.

        let batch_range = item.batch_range().clone();
        #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
        pass.set_push_constants(
            bevy::render::render_resource::ShaderStages::VERTEX,
            0,
            &(batch_range.start as i32).to_le_bytes(),
        );

        // TODO: bind buffer at the index of the first ant
        let _ = ant.index;
        let ant_buffer_start = (item
            .dynamic_offset()
            .unwrap_or(NonMaxU32::new(0).unwrap())
            .get() as usize
            / size_of::<Mesh2dUniform>()
            * size_of::<AntMaterialInstance>()) as u64;

        pass.set_vertex_buffer(
            1,
            ant_material_meta
                .into_inner()
                .instances
                .buffer()
                .unwrap()
                .slice(ant_buffer_start..),
        );

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, batch_range.clone());
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, batch_range.clone());
            }
        }
        RenderCommandResult::Success
    }
}
