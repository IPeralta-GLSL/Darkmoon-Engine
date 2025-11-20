

use crate::renderers::temporal_reprojection::{TemporalReprojectionRenderer, TemporalQuality};

impl WorldRenderer {
    
    pub fn init_temporal_reprojection(&mut self) {
        
        self.temporal_reprojection = Some(TemporalReprojectionRenderer::new(
            TemporalQuality::High.to_config()
        ));
    }

    pub fn set_temporal_quality(&mut self, quality: TemporalQuality) {
        if let Some(ref mut temporal_reprojection) = self.temporal_reprojection {
            temporal_reprojection.update_config(quality.to_config());
        }
    }

    pub fn get_temporal_config(&self) -> Option<&crate::renderers::temporal_reprojection::TemporalReprojectionConfig> {
        self.temporal_reprojection.as_ref().map(|tr| tr.get_config())
    }
}

