//! A material that draws ants

use std::{f32::consts::PI, ops::DerefMut};

use crate::components::ants::{Ant, AntPositionKind};
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
            AsBindGroup, BufferUsages, BufferVec, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{
        extract_mesh2d, Material2d, Material2dPlugin, RenderMesh2dInstance, RenderMesh2dInstances,
        SetMaterial2dBindGroup, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};
use bytemuck::{Pod, Zeroable};

pub struct AntMaterialPlugin;
impl Plugin for AntMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<AntMaterial>::default(),
            ExtractComponentPlugin::<RenderAnt>::default(),
        ));

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
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AntMaterial {
    #[uniform(0)]
    pub is_side: u32,
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
                // color
                VertexFormat::Float32x4,
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

#[derive(Debug, Component)]
pub struct RenderAnt {
    instance: AntMaterialInstance,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Component)]
pub struct AntMaterialInstance {
    color: Vec4,
}

impl ExtractComponent for RenderAnt {
    type Query = Read<Ant>;

    type Filter = ();

    type Out = Self;

    fn extract_component(ant: bevy::ecs::query::QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        let color = match ant.position_kind {
            AntPositionKind::Background => Color::BLACK,
            AntPositionKind::VerticalWall { .. } => Color::BLUE,
            AntPositionKind::HorizontalWall { .. } => Color::GREEN,
        }
        .into();
        Some(Self {
            instance: AntMaterialInstance { color },
        })
    }
}

pub fn prepare_ant_material(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    phases: Query<&mut RenderPhase<Transparent2d>>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut ant_material_meta: ResMut<AntMaterialMeta>,
    ants: Query<&RenderAnt>,
) {
    for phase in phases.iter() {
        for item in phase.items.iter() {
            if item.draw_function == AntMaterial::draw_function_id(&draw_functions) {
                let ant = ants.get(item.entity).unwrap();
                ant_material_meta.instances.push(ant.instance);
            }
        }
    }
    ant_material_meta
        .instances
        .write_buffer(&render_device, &render_queue);
    ant_material_meta.instances.clear();
}

pub fn extract_ants_render_transform(
    mut ants: Extract<Query<(Entity, &Ant, &ViewVisibility)>>,
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
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: (),
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
        pass.set_vertex_buffer(
            1,
            ant_material_meta
                .into_inner()
                .instances
                .buffer()
                .unwrap()
                .slice(..),
        );

        let batch_range = item.batch_range();
        #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
        pass.set_push_constants(
            ShaderStages::VERTEX,
            0,
            &(batch_range.start as i32).to_le_bytes(),
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
