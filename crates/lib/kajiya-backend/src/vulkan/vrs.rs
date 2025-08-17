use ash::{prelude::VkResult, vk, Device};
use glam::{uvec2, UVec2};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ShadingRateFragment {
    Rate1x1 = 0,
    Rate1x2 = 1,
    Rate2x1 = 4,
    Rate2x2 = 5,
    Rate2x4 = 6,
    Rate4x2 = 9,
    Rate4x4 = 10,
}

impl ShadingRateFragment {
    pub fn from_performance_target(performance_factor: f32) -> Self {
        match performance_factor {
            x if x >= 4.0 => Self::Rate4x4,
            x if x >= 2.0 => Self::Rate2x2,
            x if x >= 1.5 => Self::Rate1x2,
            _ => Self::Rate1x1,
        }
    }

    pub fn texel_size(self) -> UVec2 {
        match self {
            Self::Rate1x1 => uvec2(1, 1),
            Self::Rate1x2 => uvec2(1, 2),
            Self::Rate2x1 => uvec2(2, 1),
            Self::Rate2x2 => uvec2(2, 2),
            Self::Rate2x4 => uvec2(2, 4),
            Self::Rate4x2 => uvec2(4, 2),
            Self::Rate4x4 => uvec2(4, 4),
        }
    }

    pub fn performance_gain(self) -> f32 {
        let size = self.texel_size();
        (size.x * size.y) as f32
    }

    pub fn to_vk_rate(self) -> u32 {
        self as u32
    }
}

pub struct VrsManager {
    device: Device,
    shading_rate_image: Option<vk::Image>,
    shading_rate_image_view: Option<vk::ImageView>,
    shading_rate_memory: Option<vk::DeviceMemory>,
    extent: UVec2,
    tile_size: UVec2,
}

impl VrsManager {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            shading_rate_image: None,
            shading_rate_image_view: None,
            shading_rate_memory: None,
            extent: UVec2::ZERO,
            tile_size: UVec2::ZERO,
        }
    }

    pub fn create_shading_rate_image(
        &mut self,
        extent: UVec2,
        tile_size: UVec2,
    ) -> VkResult<()> {
        self.cleanup_resources();

        let image_extent = vk::Extent3D {
            width: (extent.x + tile_size.x - 1) / tile_size.x,
            height: (extent.y + tile_size.y - 1) / tile_size.y,
            depth: 1,
        };

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8_UINT)
            .extent(image_extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::FRAGMENT_SHADING_RATE_ATTACHMENT_KHR 
                | vk::ImageUsageFlags::TRANSFER_DST
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        unsafe {
            let image = self.device.create_image(&image_create_info, None)?;
            
            let memory_requirements = self.device.get_image_memory_requirements(image);
            
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(0);

            let memory = self.device.allocate_memory(&allocate_info, None)?;
            self.device.bind_image_memory(image, memory, 0)?;

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8_UINT)
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build()
                );

            let image_view = self.device.create_image_view(&image_view_create_info, None)?;

            self.shading_rate_image = Some(image);
            self.shading_rate_image_view = Some(image_view);
            self.shading_rate_memory = Some(memory);
            self.extent = extent;
            self.tile_size = tile_size;
        }

        Ok(())
    }

    pub fn update_shading_rates(
        &self,
        cmd_buffer: vk::CommandBuffer,
        rates: &[ShadingRateFragment],
    ) {
        if self.shading_rate_image.is_none() {
            return;
        }

        let image_extent = UVec2::new(
            (self.extent.x + self.tile_size.x - 1) / self.tile_size.x,
            (self.extent.y + self.tile_size.y - 1) / self.tile_size.y,
        );

        let mut rate_data = vec![ShadingRateFragment::Rate1x1.to_vk_rate() as u8; 
                                (image_extent.x * image_extent.y) as usize];

        for (i, &rate) in rates.iter().enumerate() {
            if i < rate_data.len() {
                rate_data[i] = rate.to_vk_rate() as u8;
            }
        }

        // Para simplificar, asumimos que tenemos un buffer staging disponible
        // En implementación real, necesitarías un staging buffer apropiado
    }

    pub fn get_shading_rate_image_view(&self) -> Option<vk::ImageView> {
        self.shading_rate_image_view
    }

    pub fn cleanup_resources(&mut self) {
        unsafe {
            if let Some(image_view) = self.shading_rate_image_view.take() {
                self.device.destroy_image_view(image_view, None);
            }
            if let Some(image) = self.shading_rate_image.take() {
                self.device.destroy_image(image, None);
            }
            if let Some(memory) = self.shading_rate_memory.take() {
                self.device.free_memory(memory, None);
            }
        }
    }
}

impl Drop for VrsManager {
    fn drop(&mut self) {
        self.cleanup_resources();
    }
}

pub struct VrsConfig {
    pub enabled: bool,
    pub adaptive: bool,
    pub base_rate: ShadingRateFragment,
    pub max_rate: ShadingRateFragment,
    pub performance_threshold: f32,
}

impl Default for VrsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            adaptive: true,
            base_rate: ShadingRateFragment::Rate1x1,
            max_rate: ShadingRateFragment::Rate2x2,
            performance_threshold: 60.0,
        }
    }
}

pub fn calculate_adaptive_shading_rate(
    frame_time_ms: f32,
    target_frame_time_ms: f32,
    config: &VrsConfig,
) -> ShadingRateFragment {
    if !config.adaptive {
        return config.base_rate;
    }

    let performance_ratio = frame_time_ms / target_frame_time_ms;
    
    if performance_ratio > 1.2 {
        config.max_rate
    } else if performance_ratio > 1.1 {
        ShadingRateFragment::Rate2x1
    } else {
        config.base_rate
    }
}
