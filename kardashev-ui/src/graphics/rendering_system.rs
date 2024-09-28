use std::sync::Arc;

use bytemuck::{
    Pod,
    Zeroable,
};
use palette::Srgba;
use wgpu::util::DeviceExt;

use super::{
    Backend,
    SurfaceSize,
};
use crate::{
    error::Error,
    graphics::{
        camera::{
            Camera,
            ClearColor,
        },
        material::Material,
        mesh::Mesh,
        transform::Transform,
        Surface,
    },
    world::System,
};

#[derive(Debug)]
pub struct RenderingSystem;

impl System for RenderingSystem {
    async fn run(&mut self, world: &mut hecs::World) -> Result<(), Error> {
        let mut cameras =
            world.query::<(&Camera, &Transform, Option<&ClearColor>, &RenderTarget)>();

        for (_, (camera, camera_transform, clear_color, render_target)) in &mut cameras {
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

            let camera_uniform = CameraUniform {
                view_projection: (camera.projection * camera_transform.transform.inverse())
                    .matrix()
                    .as_slice()
                    .try_into()
                    .unwrap(),
            };
            render_target.backend.queue.write_buffer(
                &render_target.camera_buffer,
                0,
                bytemuck::bytes_of(&camera_uniform),
            );

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

            let mut render_entities = world.query::<(&Transform, &Mesh, &Material)>();
            for (_entity, (transform, mesh, material)) in &mut render_entities {
                let _ = (transform, material); // todo

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                //render_pass.set_bind_group(0, &material.bind_group, &[]);
                // todo
                render_pass.set_bind_group(1, &render_target.camera_bind_group, &[]);
                render_pass.draw_indexed(0..mesh.num_indices as u32, 0, 0..1);
            }

            render_target.backend.queue.submit(Some(encoder.finish()));
            target_texture.present();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct Pipeline {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(surface: &Surface) -> Self {
        let device = &surface.backend.device;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/shader.wgsl"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        Self {
            pipeline_layout,
            pipeline,
            texture_bind_group_layout,
            camera_bind_group_layout,
        }
    }
}

#[derive(Debug)]
pub struct RenderTarget {
    surface: Arc<wgpu::Surface<'static>>,
    backend: Backend,
    pipeline: Arc<Pipeline>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: DepthTexture,
}

impl RenderTarget {
    pub fn from_surface(surface: &Surface) -> Self {
        let device = &surface.backend.device;
        let pipeline = Pipeline::new(surface);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::bytes_of(&CameraUniform::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture = DepthTexture::new(
            device,
            SurfaceSize::from_surface_configuration(&surface.surface_configuration),
        );

        Self {
            surface: surface.surface.clone(),
            backend: surface.backend.clone(),
            pipeline: Arc::new(pipeline),
            camera_buffer,
            camera_bind_group,
            depth_texture,
        }
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

#[derive(Debug)]
struct DepthTexture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

impl DepthTexture {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    fn new(device: &wgpu::Device, surface_size: SurfaceSize) -> Self {
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

        let texture = device.create_texture(&texture_descriptor);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            texture_view,
        }
    }
}
