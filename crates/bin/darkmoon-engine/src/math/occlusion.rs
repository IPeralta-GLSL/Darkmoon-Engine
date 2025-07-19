use dolly::glam::{Mat4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

use crate::math::Aabb;

/// Represents a simple depth buffer for occlusion testing
#[derive(Clone, Debug)]
pub struct DepthBuffer {
    pub width: u32,
    pub height: u32,
    pub depths: Vec<f32>,
}

impl DepthBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depths: vec![f32::INFINITY; (width * height) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.depths.fill(f32::INFINITY);
    }

    pub fn get_depth(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width && y < self.height {
            Some(self.depths[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set_depth(&mut self, x: u32, y: u32, depth: f32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if depth < self.depths[index] {
                self.depths[index] = depth;
            }
        }
    }
}

/// Configuration for occlusion culling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OcclusionCullingConfig {
    pub enabled: bool,
    pub depth_buffer_resolution: u32, // Resolution for occlusion depth buffer (e.g., 256x256)
    pub depth_bias: f32,              // Bias to prevent self-occlusion
    pub sample_count: u32,            // Number of samples per object for occlusion testing
    pub debug_visualize: bool,        // Visualize occlusion results
    pub max_test_distance: f32,       // Maximum distance for occlusion testing
}

impl Default for OcclusionCullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            depth_buffer_resolution: 128, // Start with low-res for performance
            depth_bias: 0.01,
            sample_count: 4, // Test 4 points per object
            debug_visualize: false,
            max_test_distance: 1000.0,
        }
    }
}

/// Occlusion culling system
pub struct OcclusionCuller {
    depth_buffer: DepthBuffer,
    config: OcclusionCullingConfig,
    occluder_bounds: Vec<Aabb>, // Bounding boxes of potential occluders
}

impl OcclusionCuller {
    pub fn new(config: OcclusionCullingConfig) -> Self {
        let res = config.depth_buffer_resolution;
        Self {
            depth_buffer: DepthBuffer::new(res, res),
            config,
            occluder_bounds: Vec::new(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: OcclusionCullingConfig) {
        if config.depth_buffer_resolution != self.config.depth_buffer_resolution {
            let res = config.depth_buffer_resolution;
            self.depth_buffer = DepthBuffer::new(res, res);
        }
        self.config = config;
    }

    /// Clear and prepare for new frame
    pub fn prepare_frame(&mut self) {
        self.depth_buffer.clear();
        self.occluder_bounds.clear();
    }

    /// Add a potential occluder (object that can block other objects)
    pub fn add_occluder(&mut self, bounds: Aabb, view_proj_matrix: &Mat4) {
        // Only add objects that are close enough and large enough to be effective occluders
        let center = bounds.center();
        let size = bounds.size();
        
        // Project to screen space to check size
        let screen_pos = self.project_to_screen(&center, view_proj_matrix);
        if let Some((_x, _y, depth)) = screen_pos {
            if depth < self.config.max_test_distance && size.length() > 1.0 {
                self.occluder_bounds.push(bounds);
                
                // Rasterize this occluder into the depth buffer
                self.rasterize_occluder(&bounds, view_proj_matrix);
            }
        }
    }

    /// Test if an object is occluded by previously added occluders
    pub fn is_occluded(&self, bounds: &Aabb, view_proj_matrix: &Mat4) -> bool {
        if !self.config.enabled {
            return false;
        }

        let center = bounds.center();
        let size = bounds.size();

        // Generate sample points around the object
        let sample_points = self.generate_sample_points(&center, &size);
        
        let mut visible_samples = 0;
        
        for point in sample_points {
            if let Some((x, y, depth)) = self.project_to_screen(&point, view_proj_matrix) {
                // Check if this point is visible in the depth buffer
                if let Some(buffer_depth) = self.depth_buffer.get_depth(x, y) {
                    if depth < buffer_depth + self.config.depth_bias {
                        visible_samples += 1;
                    }
                }
            }
        }

        // Object is occluded if no samples are visible
        visible_samples == 0
    }

    /// Project a world space point to screen coordinates
    fn project_to_screen(&self, point: &Vec3, view_proj_matrix: &Mat4) -> Option<(u32, u32, f32)> {
        let homogeneous = *view_proj_matrix * Vec4::new(point.x, point.y, point.z, 1.0);
        
        if homogeneous.w <= 0.0 {
            return None; // Behind camera
        }
        
        let ndc = homogeneous / homogeneous.w;
        
        // Check if within NDC bounds
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
            return None;
        }
        
        // Convert to screen coordinates
        let x = ((ndc.x + 1.0) * 0.5 * self.depth_buffer.width as f32) as u32;
        let y = ((1.0 - ndc.y) * 0.5 * self.depth_buffer.height as f32) as u32; // Flip Y
        let depth = ndc.z;
        
        Some((x, y, depth))
    }

    /// Rasterize an occluder's bounding box into the depth buffer
    fn rasterize_occluder(&mut self, bounds: &Aabb, view_proj_matrix: &Mat4) {
        // Get all 8 corners of the bounding box
        let corners = [
            Vec3::new(bounds.min.x, bounds.min.y, bounds.min.z),
            Vec3::new(bounds.max.x, bounds.min.y, bounds.min.z),
            Vec3::new(bounds.min.x, bounds.max.y, bounds.min.z),
            Vec3::new(bounds.max.x, bounds.max.y, bounds.min.z),
            Vec3::new(bounds.min.x, bounds.min.y, bounds.max.z),
            Vec3::new(bounds.max.x, bounds.min.y, bounds.max.z),
            Vec3::new(bounds.min.x, bounds.max.y, bounds.max.z),
            Vec3::new(bounds.max.x, bounds.max.y, bounds.max.z),
        ];

        // Find screen-space bounding box
        let mut min_x = self.depth_buffer.width;
        let mut max_x = 0;
        let mut min_y = self.depth_buffer.height;
        let mut max_y = 0;
        let mut min_depth = f32::INFINITY;

        for corner in &corners {
            if let Some((x, y, depth)) = self.project_to_screen(corner, view_proj_matrix) {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                min_depth = min_depth.min(depth);
            }
        }

        // Rasterize a simple rectangular region
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.depth_buffer.set_depth(x, y, min_depth);
            }
        }
    }

    /// Generate sample points for occlusion testing
    fn generate_sample_points(&self, center: &Vec3, size: &Vec3) -> Vec<Vec3> {
        let mut points = Vec::new();
        
        // Always test the center point
        points.push(*center);
        
        if self.config.sample_count <= 1 {
            return points;
        }
        
        // Generate additional sample points around the object
        let half_size = *size * 0.5;
        let sample_count = self.config.sample_count.min(8); // Limit to 8 samples max
        
        match sample_count {
            2..=4 => {
                // Sample corners
                points.push(*center + Vec3::new(half_size.x, half_size.y, 0.0));
                points.push(*center + Vec3::new(-half_size.x, half_size.y, 0.0));
                if sample_count >= 4 {
                    points.push(*center + Vec3::new(half_size.x, -half_size.y, 0.0));
                    points.push(*center + Vec3::new(-half_size.x, -half_size.y, 0.0));
                }
            }
            5..=8 => {
                // Sample all 6 face centers
                points.push(*center + Vec3::new(half_size.x, 0.0, 0.0));
                points.push(*center + Vec3::new(-half_size.x, 0.0, 0.0));
                points.push(*center + Vec3::new(0.0, half_size.y, 0.0));
                points.push(*center + Vec3::new(0.0, -half_size.y, 0.0));
                if sample_count >= 6 {
                    points.push(*center + Vec3::new(0.0, 0.0, half_size.z));
                    points.push(*center + Vec3::new(0.0, 0.0, -half_size.z));
                }
            }
            _ => {}
        }
        
        points.truncate(sample_count as usize);
        points
    }

    /// Get statistics for debugging
    pub fn get_statistics(&self) -> OcclusionCullingStatistics {
        let total_pixels = (self.depth_buffer.width * self.depth_buffer.height) as usize;
        let filled_pixels = self.depth_buffer.depths.iter()
            .filter(|&&depth| depth < f32::INFINITY)
            .count();
        
        OcclusionCullingStatistics {
            total_occluders: self.occluder_bounds.len(),
            depth_buffer_resolution: self.depth_buffer.width,
            depth_buffer_usage: (filled_pixels as f32 / total_pixels as f32) * 100.0,
        }
    }
}

/// Statistics for occlusion culling debugging
#[derive(Clone, Debug)]
pub struct OcclusionCullingStatistics {
    pub total_occluders: usize,
    pub depth_buffer_resolution: u32,
    pub depth_buffer_usage: f32, // Percentage of depth buffer filled
}
