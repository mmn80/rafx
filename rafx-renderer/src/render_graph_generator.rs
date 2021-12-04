use rafx_assets::AssetManager;
use rafx_framework::graph::PreparedRenderGraph;
use rafx_framework::render_features::{ExtractResources, RenderView};
use rafx_framework::{ImageViewResource, RafxResult, RenderResources, ResourceArc};

pub trait RenderGraphGenerator: 'static + Send {
    fn generate_render_graph(
        &self,
        asset_manager: &AssetManager,
        swapchain_image: ResourceArc<ImageViewResource>,
        rotating_frame_index: usize,
        main_view: RenderView,
        extract_resources: &ExtractResources,
        render_resources: &RenderResources,
    ) -> RafxResult<PreparedRenderGraph>;
}
