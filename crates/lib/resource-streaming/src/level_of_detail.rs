use serde::{Deserialize, Serialize};

/// Nivel de detalle para un recurso
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LodLevel {
    /// Calidad baja - optimizado para distancia
    Low = 0,
    /// Calidad media - balance entre calidad y rendimiento
    Medium = 1,
    /// Calidad alta - máxima calidad disponible
    High = 2,
}

impl Default for LodLevel {
    fn default() -> Self {
        LodLevel::Medium
    }
}

impl From<u8> for LodLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => LodLevel::Low,
            1 => LodLevel::Medium,
            2 | _ => LodLevel::High,
        }
    }
}

impl From<LodLevel> for u8 {
    fn from(lod: LodLevel) -> Self {
        lod as u8
    }
}

/// Configuración de LOD para diferentes tipos de recursos
#[derive(Debug, Clone)]
pub struct LodConfig {
    pub texture_high_distance: f32,
    pub texture_medium_distance: f32,
    pub mesh_high_distance: f32,
    pub mesh_medium_distance: f32,
    pub enable_dynamic_lod: bool,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            texture_high_distance: 30.0,
            texture_medium_distance: 100.0,
            mesh_high_distance: 50.0,
            mesh_medium_distance: 150.0,
            enable_dynamic_lod: true,
        }
    }
}

/// Gestor de niveles de detalle
#[derive(Debug, Clone)]
pub struct LodManager {
    config: LodConfig,
}

impl LodManager {
    pub fn new(high_distance: f32, medium_distance: f32, _low_distance: f32) -> Self {
        let config = LodConfig {
            texture_high_distance: high_distance * 0.6,
            texture_medium_distance: medium_distance * 0.6,
            mesh_high_distance: high_distance,
            mesh_medium_distance: medium_distance,
            enable_dynamic_lod: true,
        };
        
        Self { config }
    }
    
    /// Calcula el nivel de LOD apropiado basado en la distancia y tipo de recurso
    pub fn calculate_lod_level(&self, distance: f32, resource_type: &ResourceType) -> LodLevel {
        if !self.config.enable_dynamic_lod {
            return LodLevel::High;
        }
        
        let (high_threshold, medium_threshold) = match resource_type {
            ResourceType::Texture => (self.config.texture_high_distance, self.config.texture_medium_distance),
            ResourceType::Mesh => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
            ResourceType::Audio => (self.config.mesh_high_distance * 2.0, self.config.mesh_medium_distance * 2.0), // Audio tiene rangos mayores
            _ => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
        };
        
        if distance <= high_threshold {
            LodLevel::High
        } else if distance <= medium_threshold {
            LodLevel::Medium
        } else {
            LodLevel::Low
        }
    }
    
    /// Calcula el nivel de LOD basado en múltiples factores
    pub fn calculate_lod_advanced(
        &self,
        distance: f32,
        resource_type: &ResourceType,
        screen_size_factor: f32,
        performance_factor: f32,
    ) -> LodLevel {
        let base_lod = self.calculate_lod_level(distance, resource_type);
        
        // Ajustar basándose en el tamaño en pantalla
        let screen_adjusted = match base_lod {
            LodLevel::High if screen_size_factor < 0.1 => LodLevel::Medium,
            LodLevel::Medium if screen_size_factor < 0.05 => LodLevel::Low,
            _ => base_lod,
        };
        
        // Ajustar basándose en el rendimiento del sistema
        if performance_factor < 0.5 {
            // Sistema con bajo rendimiento, reducir LOD
            match screen_adjusted {
                LodLevel::High => LodLevel::Medium,
                LodLevel::Medium => LodLevel::Low,
                LodLevel::Low => LodLevel::Low,
            }
        } else {
            screen_adjusted
        }
    }
    
    /// Obtiene la distancia de transición entre niveles de LOD
    pub fn get_transition_distance(&self, resource_type: &ResourceType, from_lod: LodLevel, to_lod: LodLevel) -> f32 {
        let (high_threshold, medium_threshold) = match resource_type {
            ResourceType::Texture => (self.config.texture_high_distance, self.config.texture_medium_distance),
            ResourceType::Mesh => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
            ResourceType::Audio => (self.config.mesh_high_distance * 2.0, self.config.mesh_medium_distance * 2.0),
            _ => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
        };
        
        match (from_lod, to_lod) {
            (LodLevel::High, LodLevel::Medium) | (LodLevel::Medium, LodLevel::High) => high_threshold,
            (LodLevel::Medium, LodLevel::Low) | (LodLevel::Low, LodLevel::Medium) => medium_threshold,
            (LodLevel::High, LodLevel::Low) | (LodLevel::Low, LodLevel::High) => (high_threshold + medium_threshold) / 2.0,
            _ => 0.0,
        }
    }
    
    /// Actualiza la configuración de LOD
    pub fn update_config(&mut self, config: LodConfig) {
        self.config = config;
    }
    
    /// Obtiene la configuración actual
    pub fn get_config(&self) -> &LodConfig {
        &self.config
    }
    
    /// Calcula el factor de calidad para un nivel de LOD específico
    pub fn get_quality_factor(&self, lod_level: LodLevel) -> f32 {
        match lod_level {
            LodLevel::High => 1.0,
            LodLevel::Medium => 0.5,
            LodLevel::Low => 0.25,
        }
    }
    
    /// Determina si se debe hacer una transición suave entre niveles de LOD
    pub fn should_use_smooth_transition(&self, distance: f32, resource_type: &ResourceType) -> bool {
        let (high_threshold, medium_threshold) = match resource_type {
            ResourceType::Texture => (self.config.texture_high_distance, self.config.texture_medium_distance),
            ResourceType::Mesh => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
            _ => (self.config.mesh_high_distance, self.config.mesh_medium_distance),
        };
        
        // Usar transición suave cerca de los umbrales
        let high_range = high_threshold * 0.2;
        let medium_range = medium_threshold * 0.2;
        
        (distance >= high_threshold - high_range && distance <= high_threshold + high_range) ||
        (distance >= medium_threshold - medium_range && distance <= medium_threshold + medium_range)
    }
}

/// Tipo de recurso para cálculos de LOD
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
    Mesh,
    Texture,
    Material,
    Audio,
    Scene,
    Other,
}

impl From<&str> for ResourceType {
    fn from(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "gltf" | "glb" | "obj" | "fbx" => ResourceType::Mesh,
            "png" | "jpg" | "jpeg" | "tga" | "dds" | "hdr" | "exr" => ResourceType::Texture,
            "mtl" | "mat" => ResourceType::Material,
            "wav" | "mp3" | "ogg" => ResourceType::Audio,
            "dmoon" | "scene" => ResourceType::Scene,
            _ => ResourceType::Other,
        }
    }
}

/// Estadísticas de uso de LOD
#[derive(Debug, Default, Clone)]
pub struct LodStats {
    pub high_lod_count: u32,
    pub medium_lod_count: u32,
    pub low_lod_count: u32,
    pub total_resources: u32,
    pub memory_saved_bytes: u64,
}

impl LodStats {
    pub fn add_resource(&mut self, lod_level: LodLevel) {
        match lod_level {
            LodLevel::High => self.high_lod_count += 1,
            LodLevel::Medium => self.medium_lod_count += 1,
            LodLevel::Low => self.low_lod_count += 1,
        }
        self.total_resources += 1;
    }
    
    pub fn get_distribution_percentage(&self, lod_level: LodLevel) -> f32 {
        if self.total_resources == 0 {
            return 0.0;
        }
        
        let count = match lod_level {
            LodLevel::High => self.high_lod_count,
            LodLevel::Medium => self.medium_lod_count,
            LodLevel::Low => self.low_lod_count,
        };
        
        (count as f32 / self.total_resources as f32) * 100.0
    }
    
    pub fn estimate_memory_savings(&self) -> f32 {
        if self.total_resources == 0 {
            return 0.0;
        }
        
        // Estimación aproximada de ahorro de memoria
        // Asumiendo que LOD medium ahorra 50% y LOD low ahorra 75%
        let medium_savings = self.medium_lod_count as f32 * 0.5;
        let low_savings = self.low_lod_count as f32 * 0.75;
        
        ((medium_savings + low_savings) / self.total_resources as f32) * 100.0
    }
}
