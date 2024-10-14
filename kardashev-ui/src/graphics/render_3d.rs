use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::Vertex;
use nalgebra::Point3;
use palette::{
    Srgba,
    WithAlpha,
};

use crate::{
    graphics::{
        camera::{
            CameraProjection,
            ClearColor,
        },
        draw_batch::DrawBatcher,
        material::Material,
        mesh::Mesh,
        render_frame::{
            CreateRenderPass,
            RenderPass,
            RenderPassContext,
        },
        transform::GlobalTransform,
        utils::{
            color_to_array,
            color_to_wgpu,
            wgpu_buffer_size,
            GpuResourceCache,
            HasVertexBufferLayout,
        },
        Backend,
        Surface,
        SurfaceSize,
    },
    world::resource::Resources,
};

#[derive(Debug)]
pub struct Render3d<P = Render3dMeshesWithMaterial> {
    pipeline: P,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group: wgpu::BindGroup,
    depth_texture: DepthTexture,
    draw_batcher: DrawBatcher,
}

impl<P: Render3dPipeline> Render3d<P> {
    pub fn new(
        backend: &Backend,
        surface_size: SurfaceSize,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let camera_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: wgpu_buffer_size::<CameraUniform>(),
        });

        let camera_bind_group_layout =
            backend
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

        let camera_bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let light_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: wgpu_buffer_size::<LightUniform>(),
        });

        let light_bind_group_layout =
            backend
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

        let light_bind_group = backend
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                }],
                label: None,
            });

        let create_pipeline_context = CreatePipelineContext {
            backend,
            surface_format,
            camera_bind_group_layout: &camera_bind_group_layout,
            light_bind_group_layout: &light_bind_group_layout,
        };

        let pipeline = P::create_pipeline(&create_pipeline_context);

        let depth_texture = DepthTexture::new(backend, surface_size);
        let draw_batcher = DrawBatcher::new(backend);

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
            depth_texture,
            draw_batcher,
        }
    }
}

impl RenderPass for Render3d {
    fn render(&mut self, context: &mut RenderPassContext) {
        let mut query = context
            .world
            .query_one::<(Option<&ClearColor>, &GlobalTransform, &CameraProjection)>(
                context.render_target_entity,
            )
            .expect("render target entity doesn't exist");

        if let Some((clear_color, camera_transform, camera_projection)) = query.get() {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: context.target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: clear_color
                                .map(|c| {
                                    wgpu::LoadOp::Clear(color_to_wgpu(c.clear_color.into_format()))
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

            let camera_uniform = CameraUniform::from_camera(camera_projection, camera_transform);
            context.backend.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );

            // todo: query lights from world
            let white = palette::named::WHITE.into_format();
            let light_uniform = LightUniform::new(Point3::new(0., -2., 5.))
                .with_ambient_color(white.with_alpha(0.1))
                .with_diffuse_color(white.with_alpha(1.0))
                .with_specular_color(white.with_alpha(1.0));
            context.backend.queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::bytes_of(&light_uniform),
            );

            self.pipeline.render(&mut RenderPipelineContext {
                backend: &mut context.backend,
                render_pass: &mut render_pass,
                camera_bind_group: &self.camera_bind_group,
                light_bind_group: &self.light_bind_group,
                world: context.world,
                resources: context.resources,
            });
        }
    }

    fn resize(&mut self, backend: &Backend, surface_size: SurfaceSize) {
        self.depth_texture = DepthTexture::new(backend, surface_size);
    }
}

impl CreateRenderPass for Render3d {
    fn create_render_pass(surface: &Surface) -> Self {
        Self::new(
            &surface.backend,
            surface.size(),
            surface.surface_configuration.format,
        )
    }
}

pub trait Render3dPipeline {
    fn create_pipeline(pipeline_context: &CreatePipelineContext) -> Self;
    fn render(&mut self, pipeline_context: &mut RenderPipelineContext);
}

#[derive(Debug)]
pub struct CreatePipelineContext<'a> {
    pub backend: &'a Backend,
    pub surface_format: wgpu::TextureFormat,
    pub camera_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub light_bind_group_layout: &'a wgpu::BindGroupLayout,
}

// todo: impl Debug
pub struct RenderPipelineContext<'a> {
    pub backend: &'a Backend,
    pub render_pass: &'a mut wgpu::RenderPass<'a>,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub world: &'a hecs::World,
    pub resources: &'a mut Resources,
}

#[derive(Debug)]
pub struct Render3dMeshesWithMaterial {
    pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    draw_batcher: DrawBatcher,
}

impl Render3dPipeline for Render3dMeshesWithMaterial {
    fn create_pipeline(context: &CreatePipelineContext) -> Self {
        let shader = context
            .backend
            .device
            .create_shader_module(wgpu::include_wgsl!("../../../assets/shader/shader.wgsl"));

        const fn material_texture_view_bind_group_entry(
            binding: u32,
        ) -> wgpu::BindGroupLayoutEntry {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            }
        }

        const fn material_sampler_bind_group_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        }

        let material_bind_group_layout =
            context
                .backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture bind group layout"),
                    entries: &[
                        material_texture_view_bind_group_entry(0),
                        material_sampler_bind_group_entry(1),
                        material_texture_view_bind_group_entry(2),
                        material_sampler_bind_group_entry(3),
                        material_texture_view_bind_group_entry(4),
                        material_sampler_bind_group_entry(5),
                        material_texture_view_bind_group_entry(6),
                        material_sampler_bind_group_entry(7),
                        material_texture_view_bind_group_entry(8),
                        material_sampler_bind_group_entry(9),
                        material_texture_view_bind_group_entry(10),
                        material_sampler_bind_group_entry(11),
                    ],
                });

        let pipeline_layout =
            context
                .backend
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &material_bind_group_layout,
                        &context.camera_bind_group_layout,
                        &context.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline =
            context
                .backend
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::layout(), Instance::layout()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.surface_format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DepthTexture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        Self {
            pipeline,
            material_bind_group_layout,
            draw_batcher: DrawBatcher::new(context.backend),
        }
    }

    fn render(&mut self, context: &mut RenderPipelineContext) {
        context.render_pass.set_pipeline(&self.pipeline);

        context
            .render_pass
            .set_bind_group(1, &context.camera_bind_group, &[]);
        context
            .render_pass
            .set_bind_group(2, &context.light_bind_group, &[]);

        tracing::trace!("batching");

        let mut render_entities = context
            .world
            .query::<(&GlobalTransform, &mut Mesh, &mut Material)>();

        let gpu_resource_cache = context
            .resources
            .get_mut_or_insert_default::<GpuResourceCache>();

        for (entity, (transform, mesh, material)) in render_entities.iter() {
            tracing::trace!(?entity, ?mesh, ?material, "rendering entity");

            // handle errors

            let Ok(mesh) = mesh.gpu(&context.backend, gpu_resource_cache)
            else {
                continue;
            };

            let Ok(material) = material.gpu(
                &context.backend,
                gpu_resource_cache,
                &self.material_bind_group_layout,
            )
            else {
                continue;
            };

            self.draw_batcher
                .push(mesh, material, Instance::from_transform(transform));
        }

        self.draw_batcher.draw(context.backend, context.render_pass);
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
}

impl DepthTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
            format: Self::DEPTH_FORMAT,
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
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    pub view_projection: [f32; 16],
    pub view_position: [f32; 3],
    padding: u32,
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
            padding: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub ambient_color: [f32; 4],
    pub diffuse_color: [f32; 4],
    pub specular_color: [f32; 4],
    pub position: [f32; 3],
    _padding: u32,
}

impl LightUniform {
    pub fn new(position: Point3<f32>) -> Self {
        Self {
            ambient_color: Default::default(),
            diffuse_color: Default::default(),
            specular_color: Default::default(),
            position: position.coords.as_slice().try_into().unwrap(),
            _padding: 0,
        }
    }

    pub fn with_ambient_color(mut self, ambient_color: Srgba<f32>) -> Self {
        self.ambient_color = color_to_array(ambient_color);
        self
    }

    pub fn with_diffuse_color(mut self, diffuse_color: Srgba<f32>) -> Self {
        self.diffuse_color = color_to_array(diffuse_color);
        self
    }

    pub fn with_specular_color(mut self, specular_color: Srgba<f32>) -> Self {
        self.specular_color = color_to_array(specular_color);
        self
    }
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
            model_transform: transform
                .model_matrix
                .to_homogeneous()
                .as_slice()
                .try_into()
                .expect("convert model matrix to array"),
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
