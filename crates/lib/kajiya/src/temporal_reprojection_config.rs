// Example configuration file for Enhanced Temporal Reprojection System
// Add this to your world_renderer.rs or similar configuration file

use crate::renderers::temporal_reprojection::{TemporalReprojectionRenderer, TemporalQuality};

impl WorldRenderer {
    /// Initialize temporal reprojection system
    pub fn init_temporal_reprojection(&mut self) {
        // Create temporal reprojection renderer with high quality settings
        self.temporal_reprojection = Some(TemporalReprojectionRenderer::new(
            TemporalQuality::High.to_config()
        ));
    }

    /// Update temporal reprojection quality at runtime
    pub fn set_temporal_quality(&mut self, quality: TemporalQuality) {
        if let Some(ref mut temporal_reprojection) = self.temporal_reprojection {
            temporal_reprojection.update_config(quality.to_config());
        }
    }

    /// Get current temporal reprojection configuration
    pub fn get_temporal_config(&self) -> Option<&crate::renderers::temporal_reprojection::TemporalReprojectionConfig> {
        self.temporal_reprojection.as_ref().map(|tr| tr.get_config())
    }
}

// Usage example in render loop:
/*
impl WorldRenderer {
    pub fn render_frame(&mut self, frame_desc: &WorldFrameDesc) -> RenderOutput {
        // ... existing render code ...
        
        // Apply enhanced temporal reprojection to various buffers
        if let Some(ref temporal_reprojection) = self.temporal_reprojection {
            // Apply to lighting buffer
            let temporal_lighting = temporal_reprojection.apply_temporal_reprojection(
                rg,
                &lighting_buffer,
                &mut temporal_lighting_history,
                &reprojection_map,
                &gbuffer_depth,
                "lighting_temporal"
            );
            
            // Apply to reflections
            let temporal_reflections = temporal_reprojection.apply_temporal_reprojection(
                rg,
                &rtr_buffer,
                &mut temporal_rtr_history,
                &reprojection_map,
                &gbuffer_depth,
                "rtr_temporal"
            );
            
            // Apply to global illumination
            let temporal_gi = temporal_reprojection.apply_temporal_reprojection(
                rg,
                &rtdgi_buffer,
                &mut temporal_gi_history,
                &reprojection_map,
                &gbuffer_depth,
                "gi_temporal"
            );
        }
        
        // ... continue with post-processing ...
    }
}
*/

// Performance tips:
// 1. Use TemporalQuality::Basic for low-end hardware
// 2. Use TemporalQuality::High for high-end hardware with RT support
// 3. Adjust history_alpha based on scene dynamics (lower for fast-moving scenes)
// 4. Enable variance_guided for better adaptive sampling in noisy areas
// 5. Tune motion_threshold based on camera movement speed
