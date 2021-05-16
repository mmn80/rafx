use crate::features::internal::{DemoFramePacket, DemoSubmitPacket};
use crate::phases::OpaqueRenderPhase;
use rafx::api::{RafxBufferDef, RafxFormat, RafxPrimitiveTopology, RafxVertexBufferBinding};
use rafx::framework::render_features::RenderPhase;
use rafx::framework::{DescriptorSetBindings, VertexDataLayout, VertexDataSetLayout};
use rafx::render_feature_write_job_prelude::*;
use rafx_api::RafxResult;
use std::marker::PhantomData;

#[derive(Default, Clone, Copy)]
struct PositionColorVertex {
    position: [f32; 2],
    color: [f32; 3],
}

pub struct DemoWriteJob<'write> {
    vertex_layout: Arc<VertexDataSetLayout>,
    frame_packet: Box<DemoFramePacket>,
    _submit_packet: Box<DemoSubmitPacket>,
    phantom: PhantomData<&'write ()>,
}

impl<'write> DemoWriteJob<'write> {
    pub fn new(
        _write_context: &RenderJobWriteContext<'write>,
        frame_packet: Box<DemoFramePacket>,
        _submit_packet: Box<DemoSubmitPacket>,
    ) -> Arc<dyn RenderFeatureWriteJob<'write> + 'write> {
        //
        // The vertex format does not need to be specified up-front to create the material pass.
        // This allows a single material to be used with vertex data stored in any format. While we
        // don't need to create it just yet, we'll do it here once and put it in an arc so we can
        // easily use it later without having to reconstruct every frame.
        //
        let vertex_layout = Arc::new(
            VertexDataLayout::build_vertex_layout(
                &PositionColorVertex::default(),
                |builder, vertex| {
                    builder.add_member(&vertex.position, "POSITION", RafxFormat::R32G32_SFLOAT);
                    builder.add_member(&vertex.color, "COLOR", RafxFormat::R32G32B32_SFLOAT);
                },
            )
            .into_set(RafxPrimitiveTopology::TriangleList),
        );

        Arc::new(Self {
            vertex_layout,
            frame_packet,
            _submit_packet,
            phantom: Default::default(),
        })
    }
}

impl<'write> RenderFeatureWriteJob<'write> for DemoWriteJob<'write> {
    fn view_frame_index(
        &self,
        view: &RenderView,
    ) -> ViewFrameIndex {
        self.frame_packet.view_frame_index(view)
    }

    fn render_submit_node(
        &self,
        write_context: &mut RenderJobCommandBufferContext,
        _view_frame_index: ViewFrameIndex,
        _render_phase_index: RenderPhaseIndex,
        _submit_node_id: SubmitNodeId,
    ) -> RafxResult<()> {
        let per_frame_data = self.frame_packet.per_frame_data().get();

        //
        // Some data we will draw
        //
        #[rustfmt::skip]
        let vertex_data = [
            PositionColorVertex { position: [0.0, 0.5], color: [1.0, 0.0, 0.0] },
            PositionColorVertex { position: [-0.5 + (per_frame_data.seconds.cos() / 2. + 0.5), -0.5], color: [0.0, 1.0, 0.0] },
            PositionColorVertex { position: [0.5 - (per_frame_data.seconds.cos() / 2. + 0.5), -0.5], color: [0.0, 0.0, 1.0] },
        ];

        assert_eq!(20, std::mem::size_of::<PositionColorVertex>());

        let color = (per_frame_data.seconds.cos() + 1.0) / 2.0;
        let uniform_data = [color, 0.0, 1.0 - color, 1.0];

        //
        // Here we create a vertex buffer. Since we only use it once we won't bother putting
        // it into dedicated GPU memory.
        //
        // The vertex_buffer is ref-counted and can be kept around as long as you like. The
        // resource manager will ensure it stays allocated until enough frames are presented
        // that it's safe to delete.
        //
        // The resource allocators should be used and dropped, not kept around. They are
        // pooled/re-used.
        //

        let resource_allocator = write_context
            .resource_context
            .create_dyn_resource_allocator_set();
        let vertex_buffer = write_context
            .device_context
            .create_buffer(&RafxBufferDef::for_staging_vertex_buffer_data(&vertex_data))?;

        vertex_buffer.copy_to_host_visible_buffer(&vertex_data)?;

        let vertex_buffer = resource_allocator.insert_buffer(vertex_buffer);

        //
        // Create a descriptor set. USUALLY - you can use the autogenerated code from the shader pipeline
        // in higher level rafx crates to make this more straightforward - this is shown in the demo.
        // Also, flush_changes is automatically called when dropped, we only have to call it
        // here because we immediately use the descriptor set.
        //
        // Once the descriptor set is created, it's ref-counted and you can keep it around
        // as long as you like. The resource manager will ensure it stays allocated
        // until enough frames are presented that it's safe to delete.
        //
        // The allocator should be used and dropped, not kept around. It is pooled/re-used.
        // flush_changes is automatically called on drop.
        //
        let material_pass = per_frame_data.triangle_material.as_ref().unwrap();
        let descriptor_set_layout = material_pass.get_raw().descriptor_set_layouts[0].clone();

        let mut descriptor_set_allocator = write_context
            .resource_context
            .create_descriptor_set_allocator();
        let mut dyn_descriptor_set = descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&descriptor_set_layout)?;
        dyn_descriptor_set.set_buffer_data(0, &uniform_data);
        dyn_descriptor_set.flush(&mut descriptor_set_allocator)?;
        descriptor_set_allocator.flush_changes()?;

        // At this point if we don't intend to change the descriptor, we can grab the
        // descriptor set inside and use it as a ref-counted resource.
        let descriptor_set = dyn_descriptor_set.descriptor_set();

        //
        // Fetch the pipeline. If we have a pipeline for this material that's compatible with
        // the render target and vertex layout, we'll use it. Otherwise, we create it.
        //
        // The render phase is not really utilized to the full extent in this demo, but it
        // would normally help pair materials with render targets, ensuring newly loaded
        // materials can create pipelines ahead-of-time, off the render codepath.
        //
        let pipeline = write_context
            .resource_context
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                OpaqueRenderPhase::render_phase_index(),
                &material_pass,
                &write_context.render_target_meta,
                &self.vertex_layout,
            )?;

        //
        // We have everything needed to draw now, write instruction to the command buffer
        //
        let cmd_buffer = &write_context.command_buffer;
        cmd_buffer.cmd_bind_pipeline(&pipeline.get_raw().pipeline)?;
        cmd_buffer.cmd_bind_vertex_buffers(
            0,
            &[RafxVertexBufferBinding {
                buffer: &vertex_buffer.get_raw().buffer,
                byte_offset: 0,
            }],
        )?;

        descriptor_set.bind(&cmd_buffer)?;
        cmd_buffer.cmd_draw(3, 0)?;

        Ok(())
    }

    fn feature_debug_constants(&self) -> &'static RenderFeatureDebugConstants {
        super::render_feature_debug_constants()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        super::render_feature_index()
    }
}