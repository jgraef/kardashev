use std::sync::Arc;

use bytemuck::{
    Pod,
    Zeroable,
};
use hecs::{
    Entity,
    World,
};
use palette::{
    Srgba,
    WithAlpha,
};
use parking_lot::RwLock;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use super::{
    camera::Camera,
    material::Material,
    mesh::Mesh,
    transform::Transform,
};

#[derive(Debug)]
pub struct CreateRendererContext<'a> {
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    pub surface_size: PhysicalSize<u32>,
    pub device: &'a wgpu::Device,
}

#[derive(Debug)]
pub struct ResizeContext<'a> {
    pub device: &'a wgpu::Device,
    pub new_size: PhysicalSize<u32>,
}

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub target_texture: wgpu::SurfaceTexture,
    pub window: &'a winit::window::Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

impl<'a> RenderContext<'a> {
    pub fn present(self) {
        self.window.pre_present_notify();
        self.target_texture.present();
    }
}

pub trait RenderPlugin: 'static {
    fn create_renderer(&self, context: CreateRendererContext) -> Box<dyn Renderer>;
}

pub trait Renderer {
    fn resize(&mut self, context: ResizeContext);
    fn render(&mut self, context: RenderContext);
}

#[derive(Debug)]
pub struct Render3dPlugin;

impl RenderPlugin for Render3dPlugin {
    fn create_renderer(&self, context: CreateRendererContext) -> Box<dyn Renderer> {
        let shader = context
            .device
            .create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let camera_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::bytes_of(&CameraUniform::default()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let depth_texture = DepthTexture::create(&context.device, context.surface_size);

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.format,
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

        /*const VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.99240386],
            }, // A
            Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.56958647],
            }, // B
            Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.05060294],
            }, // C
            Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.1526709],
            }, // D
            Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.7347359],
            }, // E
        ];
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });*/

        Box::new(Render3d {
            pipeline,
            texture_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            camera_entity: None,
            depth_texture,
            world: Arc::new(RwLock::new(World::new())), // we'll have that here for now
        })
    }
}

pub struct Render3d {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_entity: Option<Entity>,
    depth_texture: DepthTexture,
    world: Arc<RwLock<World>>,
}

impl Renderer for Render3d {
    fn resize(&mut self, context: ResizeContext) {
        self.depth_texture = DepthTexture::create(&context.device, context.new_size);
    }

    fn render(&mut self, context: RenderContext) {
        let target_view = context
            .target_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        let mut world = self.world.write();

        let mut clear_color: Option<Srgba> =
            Some(palette::named::BLACK.with_alpha(1.0).into_format());
        let mut camera_exists = false;

        if let Some(camera_entity) = self.camera_entity {
            if let Ok((transform, camera)) =
                world.query_one_mut::<(&Transform, &Camera)>(camera_entity)
            {
                let camera_uniform = CameraUniform {
                    view_projection: (camera.projection * transform.transform.inverse())
                        .matrix()
                        .as_slice()
                        .try_into()
                        .unwrap(),
                };
                context.queue.write_buffer(
                    &self.camera_buffer,
                    0,
                    bytemuck::bytes_of(&camera_uniform),
                );
                clear_color = camera.clear_color;
                camera_exists = true;
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: clear_color
                            .map(|c| {
                                wgpu::LoadOp::Clear(convert_color_palette_to_wgpu(c.into_format()))
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

            if camera_exists {
                render_pass.set_pipeline(&self.pipeline);

                for (_entity, (transform, mesh, material)) in
                    world.query_mut::<(&Transform, &Mesh, &Material)>()
                {
                    let _ = (transform, material); // todo

                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    //render_pass.set_bind_group(0, &material.bind_group, &[]);
                    // todo
                    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    render_pass.draw_indexed(0..mesh.num_indices as u32, 0, 0..1);
                }
            }
        }

        context.queue.submit(Some(encoder.finish()));

        context.present();
    }
}

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
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
            ],
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [f32; 16],
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

struct DepthTexture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

impl DepthTexture {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    fn create(device: &wgpu::Device, size: PhysicalSize<u32>) -> Self {
        let size = wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
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

        let texture = device.create_texture(&texture_descriptor);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            texture_view,
        }
    }
}
