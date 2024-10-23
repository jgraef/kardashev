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
            CpuMaterial,
            GpuMaterial,
            GpuMaterialId,
            Material,
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
            srgb_to_array4,
            srgba_to_wgpu,
            vector3_to_array4,
            wgpu_buffer_size,
            GpuResourceCache,
            HasVertexBufferLayout,
        },
        Backend,
        SurfaceSize,
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
        let camera_buffer = context
            .backend
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("camera buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
                size: wgpu_buffer_size::<CameraUniform>(),
            });

        let camera_bind_group_layout =
            context
                .backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let camera_bind_group =
            context
                .backend
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        let light_buffer = context
            .backend
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("light buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
                size: wgpu_buffer_size::<LightUniform>(),
            });

        let light_bind_group_layout =
            context
                .backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("light bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let light_bind_group =
            context
                .backend
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &light_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: light_buffer.as_entire_binding(),
                    }],
                    label: None,
                });

        let pipeline = self
            .create_pipeline
            .create_pipeline(&CreateRender3dPipelineContext {
                backend: context.backend,
                surface_format: context.surface_format,
                depth_texture_format: DepthTexture::FORMAT,
                camera_bind_group_layout: &camera_bind_group_layout,
                light_bind_group_layout: &light_bind_group_layout,
            });

        let depth_texture = DepthTexture::new(context.backend, context.surface_size);
        let creation_time = Instant::now();
        let fps = TicksPerSecond::new(Duration::from_secs(1));

        Render3dPass {
            pipeline,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
            depth_texture,
            creation_time,
            fps,
        }
    }
}

#[derive(Debug)]
pub struct Render3dPass<P> {
    pipeline: P,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group: wgpu::BindGroup,
    depth_texture: DepthTexture,
    creation_time: Instant,
    fps: TicksPerSecond,
}

impl<P: Render3dPipeline> RenderPass for Render3dPass<P> {
    fn render(&mut self, context: &mut RenderPassContext) {
        self.depth_texture
            .resize_if_needed(context.target_size, context.backend);

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
                                .map(|c| {
                                    wgpu::LoadOp::Clear(srgba_to_wgpu(c.clear_color.into_format()))
                                })
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
            context.backend.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );

            // update lights uniform
            let mut light_uniform = LightUniform::default();
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
            context.backend.queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::bytes_of(&light_uniform),
            );

            self.pipeline.render(&mut Render3dPipelineContext {
                backend: &mut context.backend,
                render_pass: &mut render_pass,
                camera_bind_group: &self.camera_bind_group,
                light_bind_group: &self.light_bind_group,
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

    pub fn batch_meshes_with_material<M: CpuMaterial>(
        &mut self,
        draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, Instance>,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        tracing::trace!("batching");

        let mut render_entities = self
            .world
            .query::<(&GlobalTransform, &mut Mesh, &mut Material<M>)>();

        let gpu_resource_cache = self
            .resources
            .get_mut_or_insert_default::<GpuResourceCache>();

        for (_entity, (transform, mesh, material)) in render_entities.iter() {
            // todo: handle errors

            let Ok(mesh) = mesh.gpu(&self.backend, gpu_resource_cache)
            else {
                continue;
            };

            let Ok(material) = material.gpu(
                &self.backend,
                gpu_resource_cache,
                material_bind_group_layout,
            )
            else {
                continue;
            };

            draw_batcher.push(
                MeshMaterialPairKey {
                    mesh: mesh.get().id(),
                    material: material.get().id(),
                },
                || {
                    MeshMaterialPair {
                        mesh: mesh.clone(),
                        material: material.clone(),
                    }
                },
                Instance::from_transform(transform),
            );
        }
    }

    pub fn draw_batched_meshes_with_materials<M: CpuMaterial>(
        &mut self,
        draw_batcher: &mut DrawBatcher<MeshMaterialPairKey, MeshMaterialPair<M>, Instance>,
        instance_buffer_slot: u32,
        vertex_buffer_slot: u32,
        material_bind_group_index: u32,
    ) {
        if let Some(prepared_batch) = draw_batcher.prepare(self.backend) {
            self.render_pass
                .set_vertex_buffer(instance_buffer_slot, prepared_batch.instance_buffer);

            for batch_item in prepared_batch {
                let mesh = batch_item.value.mesh.get();
                let material = batch_item.value.material.get();

                self.render_pass
                    .set_vertex_buffer(vertex_buffer_slot, mesh.vertex_buffer.slice(..));
                self.render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                self.render_pass.set_bind_group(
                    material_bind_group_index,
                    &material.bind_group,
                    &[],
                );
                self.render_pass
                    .draw_indexed(0..mesh.num_indices as u32, 0, batch_item.range);
            }
        }
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
}

impl DepthTexture {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(backend: &Backend, surface_size: SurfaceSize) -> Self {
        let size = wgpu::Extent3d {
            width: surface_size.width,
            height: surface_size.height,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = backend.device.create_texture(&texture_descriptor);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            texture_view,
        }
    }

    pub fn resize_if_needed(&mut self, size: SurfaceSize, backend: &Backend) {
        if SurfaceSize::from_texture(&self.texture) != size {
            tracing::debug!(?size, "resizing depth texture");
            *self = DepthTexture::new(backend, size);
        }
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
    _padding2: [u32; 3],
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
            aspect: camera.aspect,
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
struct LightUniform {
    pub ambient_light: [f32; 4],
    pub num_point_lights: u32,
    _padding: [u32; 3],
    pub point_lights: [PointLightUniform; MAX_POINT_LIGHTS],
}

impl LightUniform {
    pub fn set_ambient_color(&mut self, color: Srgb<f32>) {
        self.ambient_light = srgb_to_array4(color);
    }

    pub fn add_point_light(&mut self, position: Point3<f32>, color: Srgb<f32>) -> bool {
        let index: usize = self.num_point_lights.try_into().unwrap();
        if index < MAX_POINT_LIGHTS {
            self.point_lights[index] = PointLightUniform {
                position: vector3_to_array4(position.coords),
                color: srgb_to_array4(color),
            };
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
struct PointLightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Instance {
    pub model_transform: [f32; 16],
    // note: we're using the trick mentioned here[1] to rotate the vertex normal by the rotation of
    // the model matrix: https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-normal-matrix
    //pub normal: [f32; 9],
}

impl Instance {
    pub fn from_transform(transform: &GlobalTransform) -> Self {
        Self {
            model_transform: transform.as_homogeneous_matrix_array(),
            /*normal: transform
            .model_matrix
            .isometry
            .rotation
            .to_rotation_matrix()
            .matrix()
            .as_slice()
            .try_into()
            .expect("convert rotation matrix to array"),*/
        }
    }
}

impl HasVertexBufferLayout for Instance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model transform
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // normal
                /*wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },*/
            ],
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
