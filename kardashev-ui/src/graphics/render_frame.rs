use std::task::Poll;

use bytemuck::{
    Pod,
    Zeroable,
};
use kardashev_protocol::assets::Vertex;

use crate::{
    graphics::{
        camera::{
            Camera,
            ClearColor,
        },
        loading::OnGpu,
        material::Material,
        mesh::Mesh,
        pipeline::{
            DepthTexture,
            RenderTarget,
        },
        transform::GlobalTransform,
        util::color_to_wgpu,
        Error,
    },
    world::{
        system::{
            System,
            SystemContext,
        },
        Label,
    },
};

#[derive(Debug)]
pub struct RenderFrame;

impl System for RenderFrame {
    type Error = Error;

    fn label(&self) -> &'static str {
        "rendering"
    }

    fn poll_system(
        &mut self,
        _task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut cameras = system_context.world.query::<(
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

            tracing::debug!(?label, "rendering camera");

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

            tracing::trace!("begin render pass");

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
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

            render_target.write_camera(camera, camera_transform);
            render_pass.set_bind_group(1, &render_target.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &render_target.pipeline.light_bind_group, &[]);

            tracing::trace!("batching");

            let mut render_entities =
                system_context
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

            render_target
                .draw_batcher
                .draw(&render_target.backend, &mut render_pass);

            tracing::trace!("submit command encoder");
            drop(render_pass);
            render_target.backend.queue.submit(Some(encoder.finish()));
            target_texture.present();
        }

        Poll::Ready(Ok(()))
    }
}

pub trait HasVertexBufferLayout {
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
    pub model_transform: [f32; 16],
    // note: we're using the trick mentioned here[1] to rotate the vertex normal by the rotation of
    // the model matrix [1]: https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-normal-matrix
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
