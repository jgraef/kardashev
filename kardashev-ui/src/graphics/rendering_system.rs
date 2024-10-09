use std::sync::Arc;

use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::Vertex;
use nalgebra::Matrix4;
use palette::{
    Srgba,
    WithAlpha,
};
use wgpu::util::DeviceExt;

use super::{
    camera::{
        Camera,
        ClearColor,
    },
    draw_batch::DrawBatcher,
    material::Material,
    mesh::Mesh,
    transform::GlobalTransform,
    Backend,
    Surface,
    SurfaceSize,
    SurfaceSizeListener,
    SurfaceVisibilityListener,
};
use crate::{
    error::Error,
    graphics::{
        loading::OnGpu,
        texture::LoadedTexture,
    },
    utils::thread_local_cell::ThreadLocalCell,
    world::{
        Label,
        RunSystemContext,
        System,
    },
};

#[derive(Debug)]
pub struct RenderingSystem;

impl System for RenderingSystem {
    fn label(&self) -> &'static str {
        "rendering"
    }

    async fn run<'s: 'c, 'c: 'd, 'd>(
        &'s mut self,
        context: &'d mut RunSystemContext<'c>,
    ) -> Result<(), Error> {
        let mut cameras = context.world.query::<(
            &Camera,
            &GlobalTransform,
            Option<&ClearColor>,
            &mut RenderTarget,
            Option<&Label>,
        )>();

        for (_, (camera, camera_transform, clear_color, render_target, label)) in cameras.iter() {
            let render_target = render_target.inner.get_mut();

            if let Some(surface_size) = render_target.surface_size_listener.poll() {
                tracing::debug!(?label, ?surface_size, "surface resized");
                render_target.depth_texture =
                    DepthTexture::new(&render_target.backend, surface_size);
            }

            if !render_target.is_visible() {
                tracing::debug!(?label, "skipping camera (not visible)");
                continue;
            }

            tracing::trace!(?label, "rendering camera");

            let target_texture = render_target
                .surface
                .get_current_texture()
                .expect("could not get target texture");

            let target_view = target_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = render_target.backend.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("render encoder"),
                },
            );

            let camera_uniform = CameraUniform::from_camera(camera, camera_transform);
            render_target.backend.queue.write_buffer(
                &render_target.camera_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );

            tracing::trace!("begin render pass");

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: clear_color
                            .map(|c| {
                                wgpu::LoadOp::Clear(convert_color_palette_to_wgpu(
                                    c.clear_color.into_format(),
                                ))
                            })
                            .unwrap_or(wgpu::LoadOp::Load),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &render_target.depth_texture.texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&render_target.pipeline.pipeline);

            tracing::trace!("batching");

            let mut render_entities =
                context
                    .world
                    .query::<(&GlobalTransform, &OnGpu<Mesh>, &OnGpu<Material>)>();

            for (entity, (transform, mesh, material)) in render_entities.iter() {
                tracing::trace!(?entity, ?mesh, ?material, "rendering entity");

                let Some(mesh) = mesh.get(render_target.backend.id())
                else {
                    continue;
                };

                let Some(material) = material.get(render_target.backend.id())
                else {
                    continue;
                };

                render_target.draw_batcher.push(
                    mesh,
                    material,
                    Instance::from_transform(transform),
                );
            }

            render_target.draw_batcher.draw(
                &render_target.backend,
                &render_target.camera_bind_group,
                &mut render_pass,
            );

            tracing::trace!("submit command encoder");
            drop(render_pass);
            render_target.backend.queue.submit(Some(encoder.finish()));
            target_texture.present();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub default_sampler: Arc<wgpu::Sampler>,
    pub fallback_texture: LoadedTexture,
}

impl Pipeline {
    pub fn new(surface: &Surface) -> Self {
        let device = &surface.backend.device;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

        fn material_texture_view_bind_group_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

        fn material_sampler_bind_group_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&material_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: surface.surface_configuration.format,
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

        let default_sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        let fallback_texture = LoadedTexture::color1x1(
            palette::named::BLACK.with_alpha(0),
            default_sampler.clone(),
            &surface.backend,
        );

        Self {
            pipeline_layout,
            pipeline,
            material_bind_group_layout,
            camera_bind_group_layout,
            default_sampler,
            fallback_texture,
        }
    }
}

#[derive(Debug)]
pub struct RenderTarget {
    pub(super) inner: ThreadLocalCell<RenderTargetInner>,
}

impl RenderTarget {
    pub fn from_surface(surface: &Surface) -> Self {
        let pipeline = Pipeline::new(surface);

        let camera_buffer =
            surface
                .backend
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("camera buffer"),
                    contents: bytemuck::bytes_of(&CameraUniform::default()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group =
            surface
                .backend
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &pipeline.camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        let depth_texture = DepthTexture::new(&surface.backend, surface.size());

        let draw_batcher = DrawBatcher::new(&surface.backend);

        Self {
            inner: ThreadLocalCell::new(RenderTargetInner {
                surface: surface.surface.clone(),
                surface_size_listener: surface.size_listener(),
                surface_visibility_listener: surface.visibility_listener(),
                backend: surface.backend.clone(),
                pipeline: Arc::new(pipeline),
                camera_buffer,
                camera_bind_group,
                depth_texture,
                draw_batcher,
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct RenderTargetInner {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub surface_size_listener: SurfaceSizeListener,
    pub surface_visibility_listener: SurfaceVisibilityListener,
    pub backend: Backend,
    pub pipeline: Arc<Pipeline>,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub depth_texture: DepthTexture,
    pub draw_batcher: DrawBatcher,
}

impl RenderTargetInner {
    pub fn is_visible(&self) -> bool {
        self.surface_visibility_listener.is_visible()
    }
}

trait HasVertexBufferLayout {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

impl HasVertexBufferLayout for Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Instance {
    pub transform: [f32; 16],
}

impl Instance {
    pub fn from_transform(transform: &GlobalTransform) -> Self {
        Self {
            transform: transform
                .model_matrix
                .to_homogeneous()
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }
}

impl HasVertexBufferLayout for Instance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
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
            ],
        }
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct CameraUniform {
    view_projection: [f32; 16],
}

impl CameraUniform {
    fn from_camera(camera: &Camera, transform: &GlobalTransform) -> Self {
        #[rustfmt::skip]
        pub const NALGEBRA_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        #[rustfmt::skip]
        pub const TEST: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, -1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        Self {
            view_projection: (camera.projection_matrix.as_matrix()
                * transform.model_matrix.inverse().to_homogeneous())
            .as_slice()
            .try_into()
            .unwrap(),
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            #[rustfmt::skip]
            view_projection: [
                1., 0., 0., 0.,
                0., 1., 0., 0.,
                0., 0., 1., 0.,
                0., 0., 0., 1.,
            ],
        }
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

impl DepthTexture {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    fn new(backend: &Backend, surface_size: SurfaceSize) -> Self {
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

fn convert_color_palette_to_wgpu(color: Srgba<f64>) -> wgpu::Color {
    wgpu::Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}
