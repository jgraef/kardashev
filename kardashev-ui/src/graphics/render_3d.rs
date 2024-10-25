use std::{
    sync::Arc,
    time::Duration,
};

use bytemuck::{
    Pod,
    Zeroable,
};
use nalgebra::Point3;
use palette::Srgb;

use crate::{
    ecs::resource::Resources,
    graphics::{
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
        },
        mesh::{
            GpuMesh,
            GpuMeshId,
            Mesh,
        },
        render_frame::{
            CreateRenderPass,
            CreateRenderPassContext,
            RenderPass,
            RenderPassContext,
        },
        transform::GlobalTransform,
        utils::{
            batch_meshes_with_material,
            draw_batched_meshes_with_materials,
            GpuResourceCache,
            Srgb32Ext,
            Srgba64Ext,
            TextureBuffer,
            UniformBuffer,
        },
        Backend,
    },
    utils::{
        thread_local_cell::ThreadLocalCell,
        time::{
            Instant,
            TicksPerSecond,
        },
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub struct CreateRender3dPass<P> {
    pub create_pipeline: P,
}

impl<P: CreateRender3dPipeline> CreateRenderPass for CreateRender3dPass<P> {
    type RenderPass = Render3dPass<P::Pipeline>;

    fn create_render_pass(self, context: &CreateRenderPassContext) -> Self::RenderPass {
        let camera_buffer = UniformBuffer::new(context.backend);
        let lights_buffer = UniformBuffer::new(context.backend);

        let depth_texture = TextureBuffer::new(
            context.backend,
            context.surface_size,
            wgpu::TextureFormat::Depth32Float,
            Some("depth texture"),
        );

        let pipeline = self
            .create_pipeline
            .create_pipeline(&CreateRender3dPipelineContext {
                backend: context.backend,
                surface_format: context.surface_format,
                depth_texture_format: depth_texture.format,
                camera_bind_group_layout: &camera_buffer.bind_group_layout,
                light_bind_group_layout: &lights_buffer.bind_group_layout,
            });

        let creation_time = Instant::now();
        let fps = TicksPerSecond::new(Duration::from_secs(1));

        Render3dPass {
            pipeline,
            camera_buffer,
            lights_buffer,
            depth_texture,
            creation_time,
            fps,
        }
    }
}

#[derive(Debug)]
pub struct Render3dPass<P> {
    pipeline: P,
    camera_buffer: UniformBuffer<CameraUniform>,
    lights_buffer: UniformBuffer<LightsUniform>,
    depth_texture: TextureBuffer,
    creation_time: Instant,
    fps: TicksPerSecond,
}

impl<P: Render3dPipeline> RenderPass for Render3dPass<P> {
    fn render(&mut self, context: &mut RenderPassContext) {
        self.depth_texture
            .resize(context.backend, context.target_size);

        let mut query_camera = context
            .world
            .query_one::<(Option<&ClearColor>, &GlobalTransform, &CameraProjection)>(
                context.render_target_entity,
            )
            .expect("render target entity doesn't exist");

        if let Some((clear_color, camera_transform, camera_projection)) = query_camera.get() {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render3d render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: context.target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: clear_color
                                .map(|c| wgpu::LoadOp::Clear(c.clear_color.into_format().as_wgpu()))
                                .unwrap_or(wgpu::LoadOp::Load),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            // update timing information
            let now = Instant::now();
            self.fps.push(now);

            // update camera uniform
            let camera_uniform = CameraUniform::from_camera(camera_projection, camera_transform)
                .with_time(now.duration_since(self.creation_time).as_secs_f32());
            self.camera_buffer.write(context.backend, &camera_uniform);

            // update lights uniform
            let mut light_uniform = LightsUniform::default();
            if let Some(ambient_light) = context.resources.get::<AmbientLight>() {
                light_uniform.set_ambient_color(ambient_light.color);
            }
            let mut query_lights = context.world.query::<(&GlobalTransform, &PointLight)>();
            for (_, (transform, point_light)) in query_lights.iter() {
                if !light_uniform.add_point_light(
                    transform.model_matrix.transform_point(&Point3::origin()),
                    point_light.color,
                ) {
                    break;
                }
            }
            self.lights_buffer.write(context.backend, &light_uniform);

            self.pipeline.render(&mut Render3dPipelineContext {
                backend: &mut context.backend,
                render_pass: &mut render_pass,
                camera_bind_group: &self.camera_buffer.bind_group,
                light_bind_group: &self.lights_buffer.bind_group,
                world: context.world,
                resources: context.resources,
            });
        }
        else {
            tracing::warn!("entity with RenderTarget component is missing other camera components");
        }
    }
}

pub trait CreateRender3dPipeline {
    type Pipeline: Render3dPipeline;

    fn create_pipeline(self, context: &CreateRender3dPipelineContext) -> Self::Pipeline;
}

pub trait Render3dPipeline {
    fn render(&mut self, context: &mut Render3dPipelineContext);
}

#[derive(Debug)]
pub struct CreateRender3dPipelineContext<'a> {
    pub backend: &'a Backend,
    pub surface_format: wgpu::TextureFormat,
    pub depth_texture_format: wgpu::TextureFormat,
    pub camera_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub light_bind_group_layout: &'a wgpu::BindGroupLayout,
}

// todo: impl Debug
pub struct Render3dPipelineContext<'a> {
    pub backend: &'a Backend,
    pub render_pass: &'a mut wgpu::RenderPass<'a>,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}

impl<'a> Render3dPipelineContext<'a> {
    pub fn bind_camera_uniform(&mut self, bind_group_index: u32) {
        self.render_pass
            .set_bind_group(bind_group_index, &self.camera_bind_group, &[]);
    }

    pub fn bind_light_uniform(&mut self, bind_group_index: u32) {
        self.render_pass
            .set_bind_group(bind_group_index, &self.light_bind_group, &[]);
    }

    pub fn batch_meshes_with_material<M: PipelineMaterial, I: Pod>(
        &mut self,
        draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, I>,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        make_instance: impl Fn(&GlobalTransform, &M) -> I,
    ) {
        batch_meshes_with_material(
            &self.world,
            &mut self.resources,
            &self.backend,
            draw_batcher,
            material_bind_group_layout,
            make_instance,
        );
    }

    pub fn draw_batched_meshes_with_materials<M: PipelineMaterial, I: Pod>(
        &mut self,
        draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, I>,
        instance_buffer_slot: u32,
        vertex_buffer_slot: u32,
        material_bind_group_index: u32,
    ) {
        draw_batched_meshes_with_materials(
            &mut self.render_pass,
            &self.backend,
            draw_batcher,
            instance_buffer_slot,
            vertex_buffer_slot,
            material_bind_group_index,
        );
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_projection: [f32; 16],
    pub view_position: [f32; 3],
    _padding1: u32,
    pub aspect: f32,
    pub time: f32,
    _padding2: [u32; 2],
}

impl CameraUniform {
    fn from_camera(camera: &CameraProjection, transform: &GlobalTransform) -> Self {
        Self {
            view_projection: (camera.projection_matrix.as_matrix()
                * transform.model_matrix.inverse().to_homogeneous())
            .as_slice()
            .try_into()
            .unwrap(),
            view_position: transform
                .model_matrix
                .isometry
                .translation
                .vector
                .as_slice()
                .try_into()
                .unwrap(),
            _padding1: Default::default(),
            aspect: camera.projection_matrix.aspect(),
            time: 0.0,
            _padding2: Default::default(),
        }
    }

    fn with_time(mut self, time: f32) -> Self {
        self.time = time;
        self
    }
}

pub const MAX_POINT_LIGHTS: usize = 16;

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct LightsUniform {
    pub ambient_light: [f32; 3],
    pub num_point_lights: u32,
    pub point_lights: [PointLightUniform; MAX_POINT_LIGHTS],
}

impl LightsUniform {
    pub fn set_ambient_color(&mut self, color: Srgb<f32>) {
        self.ambient_light = color.as_array3();
    }

    pub fn add_point_light(&mut self, position: Point3<f32>, color: Srgb<f32>) -> bool {
        let index: usize = self.num_point_lights.try_into().unwrap();
        if index < MAX_POINT_LIGHTS {
            self.point_lights[index] = PointLightUniform::new(position, color);
            self.num_point_lights += 1;
            true
        }
        else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    _padding0: u32,
    pub color: [f32; 3],
    _padding1: u32,
}

impl PointLightUniform {
    pub fn new(position: Point3<f32>, color: Srgb<f32>) -> Self {
        Self {
            position: position.coords.as_slice().try_into().unwrap(),
            _padding0: 0,
            color: color.as_array3(),
            _padding1: 0,
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
