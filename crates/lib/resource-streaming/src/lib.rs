pub mod resource_manager;
pub mod streaming_cache;
pub mod asset_loader;
pub mod level_of_detail;
pub mod priority_system;

pub use resource_manager::ResourceStreamingManager;
pub use streaming_cache::{StreamingCache, CacheConfig};
pub use asset_loader::{AssetLoader, LoadRequest, LoadPriority};
pub use level_of_detail::{LodLevel, LodManager};
pub use priority_system::{PriorityCalculator, StreamingPriority};

use anyhow::Result;

/// Configuración principal del sistema de streaming
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Tamaño máximo del cache en bytes
    pub max_cache_size: u64,
    /// Número de workers asíncronos para carga
    pub worker_threads: usize,
    /// Distancia máxima para cargar recursos de alta calidad
    pub high_quality_distance: f32,
    /// Distancia máxima para cargar recursos de calidad media
    pub medium_quality_distance: f32,
    /// Distancia máxima para cargar recursos de baja calidad
    pub low_quality_distance: f32,
    /// Habilitar precarga predictiva
    pub enable_predictive_loading: bool,
    /// Directorio base para assets
    pub asset_base_path: String,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 2 * 1024 * 1024 * 1024, // 2GB
            worker_threads: 4,
            high_quality_distance: 50.0,
            medium_quality_distance: 150.0,
            low_quality_distance: 500.0,
            enable_predictive_loading: true,
            asset_base_path: "assets".to_string(),
        }
    }
}

/// Inicializa el sistema de streaming de recursos
pub fn initialize_streaming(config: StreamingConfig) -> Result<ResourceStreamingManager> {
    ResourceStreamingManager::new(config)
}

/// Re-exportación de tipos comunes
pub type ResourceId = String;
pub type ResourceHandle = u64;
