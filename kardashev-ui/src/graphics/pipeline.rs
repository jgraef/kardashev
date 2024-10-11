use std::sync::Arc;

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
use wgpu::util::DeviceExt;

use crate::{
    graphics::{
        camera::Camera,
        draw_batch::DrawBatcher,
        render_frame::{
            HasVertexBufferLayout,
            Instance,
        },
        texture::LoadedTexture,
        transform::GlobalTransform,
        util::{
            color_to_array,
            wgpu_buffer_size,
        },
        Backend,
        Surface,
        SurfaceSize,
        SurfaceSizeListener,
        SurfaceVisibilityListener,
    },
    utils::thread_local_cell::ThreadLocalCell,
};

#[derive(Debug)]
pub struct Pipeline {
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    pub default_sampler: Arc<wgpu::Sampler>,
    pub fallback_texture: LoadedTexture,
}

impl Pipeline {
    pub fn new(surface: &Surface) -> Self {
        let device = &surface.backend.device;

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../../assets/shader/shader.wgsl"));

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

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let white = palette::named::WHITE.into_format();
        let light_uniform = LightUniform::new(Point3::new(0., -2., 5.))
            .with_ambient_color(white.with_alpha(0.1))
            .with_diffuse_color(white.with_alpha(1.0))
            .with_specular_color(white.with_alpha(1.0));

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light buffer"),
            contents: bytemuck::bytes_of(&light_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &material_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
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
            light_bind_group_layout,
            light_bind_group,
            light_buffer,
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

        let camera_buffer = surface
            .backend
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("camera buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
                size: wgpu_buffer_size::<CameraUniform>(),
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

    pub fn write_camera(&self, camera: &Camera, transform: &GlobalTransform) {
        let camera_uniform = CameraUniform::from_camera(camera, transform);
        self.backend.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera_uniform),
        );
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
}

impl DepthTexture {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
    fn from_camera(camera: &Camera, transform: &GlobalTransform) -> Self {
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
