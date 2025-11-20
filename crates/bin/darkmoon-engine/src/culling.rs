use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CullingMethod {
    
    EmissiveMultiplier,
    
    MoveAway,
    
    ScaleToZero,
}

impl Default for CullingMethod {
    fn default() -> Self {
        Self::MoveAway
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrustumCullingConfig {
    pub enabled: bool,
    pub debug_logging: bool,
    pub log_interval_frames: u32,
    pub default_object_size: f32,
    pub use_sphere_culling: bool, 
    pub culling_method: CullingMethod, 
}

impl Default for FrustumCullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_logging: false,
            log_interval_frames: 120, 
            default_object_size: 2.0,
            use_sphere_culling: false,
            culling_method: CullingMethod::default(),
        }
    }
}
