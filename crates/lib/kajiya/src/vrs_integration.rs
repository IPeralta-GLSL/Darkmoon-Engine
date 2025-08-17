use crate::world_renderer::WorldRenderer;
use kajiya_backend::{
    ash::vk,
    vulkan::{image::*, vrs::*},
};
use kajiya_rg::{self as rg};

impl WorldRenderer {
    pub(crate) fn create_vrs_shading_rate_image(
        &mut self,
        rg: &mut rg::TemporalRenderGraph,
        render_extent: [u32; 2],
        _gbuffer_depth: &rg::Handle<Image>,
        _velocity_img: &rg::Handle<Image>,
        config: &VrsConfig,
    ) -> Option<rg::Handle<Image>> {
        if !rg.device().vrs_enabled() || !config.enabled {
            return None;
        }

        let tile_size = rg.device().vrs_tile_size().unwrap_or([16, 16]);
        let shading_rate_extent = [
            (render_extent[0] + tile_size[0] - 1) / tile_size[0],
            (render_extent[1] + tile_size[1] - 1) / tile_size[1],
        ];

        let shading_rate_img = rg.create(
            ImageDesc::new_2d(vk::Format::R8_UINT, shading_rate_extent)
                .usage(vk::ImageUsageFlags::FRAGMENT_SHADING_RATE_ATTACHMENT_KHR | vk::ImageUsageFlags::STORAGE),
        );

        Some(shading_rate_img)
    }

    pub(crate) fn get_vrs_config() -> VrsConfig {
        VrsConfig {
            enabled: true,
            adaptive: true,
            ..Default::default()
        }
    }
}
