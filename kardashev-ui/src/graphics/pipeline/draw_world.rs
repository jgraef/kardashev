use std::sync::Arc;

use bytemuck::Pod;
use palette::Srgba;

use crate::{
    ecs::resource::Resources,
    graphics::{
        backend::Backend,
        camera::{
            CameraProjection,
            ClearColor,
        },
        draw_batch::DrawBatcher,
        light::{
            AmbientLight,
            PointLight,
        },
        material::{
            GpuMaterial,
            GpuMaterialId,
            Material,
            PipelineMaterial,
            Tint,
        },
        mesh::{
            GpuMesh,
            GpuMeshId,
            Mesh,
        },
        pipeline::{
            globals::GlobalsUniform,
            RenderWorldInput,
        },
        transform::GlobalTransform,
        utils::GpuResourceCache,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Clone, Debug)]
pub struct DrawMeshesWithMaterialsConfig {
    pub instance_buffer_slot: u32,
    pub vertex_buffer_slot: u32,
    pub material_bind_group_index: u32,
}

#[derive(Debug)]
pub struct DrawMeshesWithMaterials<M, I> {
    draw_batcher: DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, I>,
    config: DrawMeshesWithMaterialsConfig,
}

impl<M, I> DrawMeshesWithMaterials<M, I> {
    pub fn new(backend: &Backend, config: DrawMeshesWithMaterialsConfig) -> Self {
        Self {
            draw_batcher: DrawBatcher::new(backend),
            config,
        }
    }
}

impl<M: PipelineMaterial, I: Pod> DrawMeshesWithMaterials<M, I> {
    pub fn batch(
        &mut self,
        backend: &Backend,
        world: &hecs::World,
        resources: &mut Resources,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        make_instance: impl Fn(&GlobalTransform, &M, Option<&Tint>) -> I,
    ) {
        tracing::trace!("batching");

        let mut render_entities =
            world.query::<(&GlobalTransform, &mut Mesh, &mut Material<M>, Option<&Tint>)>();

        let gpu_resource_cache = resources.get_mut_or_insert_default::<GpuResourceCache>();

        for (_entity, (transform, mesh, material, tint)) in render_entities.iter() {
            // todo: handle errors

            let instance = make_instance(transform, &material.cpu, tint);

            let Ok(mesh_gpu) = mesh.gpu(backend, gpu_resource_cache)
            else {
                continue;
            };

            let Ok(material_gpu) =
                material.gpu(backend, gpu_resource_cache, material_bind_group_layout)
            else {
                continue;
            };

            self.draw_batcher.push(
                MeshMaterialPairKey {
                    mesh: mesh_gpu.get().id(),
                    material: material_gpu.get().id(),
                },
                || {
                    MeshMaterialPair {
                        mesh: mesh_gpu.clone(),
                        material: material_gpu.clone(),
                    }
                },
                instance,
            );
        }
    }

    pub fn draw(&mut self, backend: &Backend, render_pass: &mut wgpu::RenderPass) {
        if let Some(prepared_batch) = self.draw_batcher.prepare(backend) {
            tracing::trace!("drawing batch");

            render_pass.set_vertex_buffer(
                self.config.instance_buffer_slot,
                prepared_batch.instance_buffer,
            );

            for batch_item in prepared_batch {
                let mesh = batch_item.value.mesh.get();
                let material = batch_item.value.material.get();

                render_pass.set_vertex_buffer(
                    self.config.vertex_buffer_slot,
                    mesh.vertex_buffer.slice(..),
                );
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(
                    self.config.material_bind_group_index,
                    &material.bind_group,
                    &[],
                );
                render_pass.draw_indexed(0..mesh.num_indices as u32, 0, batch_item.range);
            }
        }
    }
}

pub struct DrawWorldGlobals {
    pub globals: GlobalsUniform,
    pub clear_color: Option<Srgba<f32>>,
}

impl DrawWorldGlobals {
    pub fn from_world(input: &RenderWorldInput) -> Option<Self> {
        let mut query_camera = input
            .world
            .query_one::<(Option<&ClearColor>, &GlobalTransform, &CameraProjection)>(
                input.view_entity,
            )
            .expect("render target entity doesn't exist");

        if let Some((clear_color, camera_transform, camera_projection)) = query_camera.get() {
            let mut globals = GlobalsUniform::default();
            globals.set_camera(camera_projection, camera_transform);
            globals.set_ambient_color(
                input
                    .resources
                    .get::<AmbientLight>()
                    .map(|ambient_light| ambient_light.color)
                    .unwrap_or_default(),
            );

            let mut query_lights = input.world.query::<(&GlobalTransform, &PointLight)>();
            for (_, (transform, point_light)) in query_lights.iter() {
                if !globals.add_point_light(transform.position(), point_light.color) {
                    break;
                }
            }

            Some(Self {
                globals,
                clear_color: clear_color.map(|clear_color| clear_color.clear_color.into_format()),
            })
        }
        else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshMaterialPairKey {
    pub mesh: GpuMeshId,
    pub material: GpuMaterialId,
}

#[derive(Clone, Debug)]
pub struct MeshMaterialPair<M> {
    pub mesh: Arc<ThreadLocalCell<GpuMesh>>,
    pub material: Arc<ThreadLocalCell<GpuMaterial<M>>>,
}
