use kajiya_backend::{ash::vk, vulkan::image::*};
use kajiya_rg::{self as rg, GetOrCreateTemporal};

#[derive(Clone, Debug)]
pub struct TemporalReprojectionConfig {
    pub history_alpha: f32,
    pub max_temporal_samples: u32,
    pub motion_threshold: f32,
    pub enable_variance_guided: bool,
    pub enable_depth_validation: bool,
    pub enable_normal_validation: bool,
}

impl Default for TemporalReprojectionConfig {
    fn default() -> Self {
        Self {
            history_alpha: 0.95,
            max_temporal_samples: 64,
            motion_threshold: 0.01,
            enable_variance_guided: true,
            enable_depth_validation: true,
            enable_normal_validation: true,
        }
    }
}

pub struct TemporalReprojectionRenderer {
    config: TemporalReprojectionConfig,
}

impl TemporalReprojectionRenderer {
    pub fn new(config: TemporalReprojectionConfig) -> Self {
        Self { config }
    }

    pub fn create_variance_buffer(
        &self,
        rg: &mut rg::TemporalRenderGraph,
        name: &str,
        extent: [u32; 2],
    ) -> rg::Handle<Image> {
        rg.get_or_create_temporal(
            name,
            ImageDesc::new_2d(vk::Format::R16G16B16A16_SFLOAT, extent).usage(
                vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::STORAGE
                    | vk::ImageUsageFlags::TRANSFER_DST,
            ),
        )
        .unwrap()
    }

    pub fn update_config(&mut self, config: TemporalReprojectionConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &TemporalReprojectionConfig {
        &self.config
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TemporalQuality {
    Basic,
    Enhanced,
    High,
    Ultra,
}

impl TemporalQuality {
    pub fn to_config(self) -> TemporalReprojectionConfig {
        match self {
            TemporalQuality::Basic => TemporalReprojectionConfig {
                history_alpha: 0.85,
                max_temporal_samples: 16,
                motion_threshold: 0.02,
                enable_variance_guided: false,
                enable_depth_validation: true,
                enable_normal_validation: false,
            },
            TemporalQuality::Enhanced => TemporalReprojectionConfig {
                history_alpha: 0.90,
                max_temporal_samples: 32,
                motion_threshold: 0.015,
                enable_variance_guided: false,
                enable_depth_validation: true,
                enable_normal_validation: true,
            },
            TemporalQuality::High => TemporalReprojectionConfig {
                history_alpha: 0.95,
                max_temporal_samples: 64,
                motion_threshold: 0.01,
                enable_variance_guided: true,
                enable_depth_validation: true,
                enable_normal_validation: true,
            },
            TemporalQuality::Ultra => TemporalReprojectionConfig {
                history_alpha: 0.98,
                max_temporal_samples: 128,
                motion_threshold: 0.005,
                enable_variance_guided: true,
                enable_depth_validation: true,
                enable_normal_validation: true,
            },
        }
    }
}

pub mod temporal_utils {
    use super::*;

    pub fn calculate_temporal_weight(
        motion_magnitude: f32,
        depth_similarity: f32,
        normal_similarity: f32,
        config: &TemporalReprojectionConfig,
    ) -> f32 {
        let mut weight = config.history_alpha;

        if motion_magnitude > config.motion_threshold {
            weight *= (1.0 - (motion_magnitude - config.motion_threshold) * 10.0).max(0.0);
        }

        if config.enable_depth_validation {
            weight *= depth_similarity.powf(2.0);
        }

        if config.enable_normal_validation {
            weight *= normal_similarity.powf(1.5);
        }

        weight.clamp(0.0, 1.0)
    }

    pub fn create_temporal_desc(extent: [u32; 2], format: vk::Format) -> ImageDesc {
        ImageDesc::new_2d(format, extent).usage(
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::TRANSFER_DST,
        )
    }
}
