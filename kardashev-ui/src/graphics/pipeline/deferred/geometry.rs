use kardashev_protocol::assets::Vertex;

use crate::graphics::{
    material::PipelineMaterial,
    pipeline::{
        deferred::gbuffer::GeometryBuffer,
        draw_world::{
            DrawMeshesWithMaterials,
            DrawMeshesWithMaterialsConfig,
        },
        forward::blinn_phong::{
            BlinnPhongMaterial,
            Instance,
        },
        globals::GlobalsUniform,
        CreatePipelineContext,
        RenderPipeline,
        RenderPipelineContext,
        RenderWorldInput,
    },
    utils::{
        HasVertexBufferLayout,
        PipelineBuilder,
        RenderPassBuilder,
        TextureBuffer,
        UniformBuffer,
    },
};

#[derive(Debug)]
pub struct GeometryPipeline {
    depth_texture: TextureBuffer,
    material_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    draw: DrawMeshesWithMaterials<BlinnPhongMaterial, Instance>,
}

impl GeometryPipeline {
    pub fn new(
        context: &CreatePipelineContext,
        globals: &UniformBuffer<GlobalsUniform>,
        geometry_buffer: &GeometryBuffer,
    ) -> Self {
        let depth_texture = TextureBuffer::new(
            context.backend,
            geometry_buffer.size,
            wgpu::TextureFormat::Depth32Float,
            Some("depth texture"),
        );

        let material_bind_group_layout =
            BlinnPhongMaterial::create_bind_group_layout(context.backend);

        let pipeline = PipelineBuilder::new(shader::SOURCE)
            .with_label("deferred geometry pipeline")
            .with_bind_group_layout(&globals.bind_group_layout)
            .with_bind_group_layout(&material_bind_group_layout)
            .with_vertex_buffer_layout(Vertex::layout())
            .with_vertex_buffer_layout(Instance::layout())
            .with_fragment_targets(geometry_buffer.fragment_targets())
            .with_depth_texture_format(depth_texture.format)
            .build(context.backend);

        Self {
            depth_texture,
            material_bind_group_layout,
            pipeline,
            draw: DrawMeshesWithMaterials::new(
                context.backend,
                DrawMeshesWithMaterialsConfig {
                    instance_buffer_slot: 1,
                    vertex_buffer_slot: 0,
                    material_bind_group_index: 1,
                },
            ),
        }
    }
}

impl RenderPipeline for GeometryPipeline {
    type Input<'a> = (RenderWorldInput<'a>, &'a UniformBuffer<GlobalsUniform>);
    type Output<'a> = &'a GeometryBuffer;

    fn render(
        &mut self,
        context: &mut RenderPipelineContext,
        (input, globals): Self::Input<'_>,
        output: Self::Output<'_>,
    ) {
        self.depth_texture.resize(context.backend, output.size);

        let mut render_pass = RenderPassBuilder::<{ GeometryBuffer::NUM_TEXTURES }>::default()
            .with_label("deferred geometry render pass")
            .with_color_attachments(output.color_attachments())
            .with_depth_attachment(&self.depth_texture.texture_view, Some(1.0))
            .begin(context.encoder);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &globals.bind_group, &[]);

        self.draw.batch(
            context.backend,
            input.world,
            input.resources,
            &self.material_bind_group_layout,
            Instance::new,
        );
        self.draw.draw(context.backend, &mut render_pass);
    }
}

#[include_wgsl_oil::include_wgsl_oil("geometry.wgsl")]
mod shader {}
