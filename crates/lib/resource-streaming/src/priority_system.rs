use crate::asset_loader::LoadPriority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StreamingPriority {
    Invisible = 0,
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

impl Default for StreamingPriority {
    fn default() -> Self {
        StreamingPriority::Medium
    }
}

impl From<LoadPriority> for StreamingPriority {
    fn from(load_priority: LoadPriority) -> Self {
        match load_priority {
            LoadPriority::Low => StreamingPriority::Low,
            LoadPriority::Medium => StreamingPriority::Medium,
            LoadPriority::High => StreamingPriority::High,
            LoadPriority::Critical => StreamingPriority::Critical,
        }
    }
}

impl From<StreamingPriority> for u8 {
    fn from(priority: StreamingPriority) -> Self {
        priority as u8
    }
}

#[derive(Debug, Clone)]
pub struct PriorityFactors {
    
    pub distance_factor: f32,
    
    pub view_angle_factor: f32,
    
    pub screen_size_factor: f32,
    
    pub movement_speed_factor: f32,
    
    pub recency_factor: f32,
    
    pub importance_factor: f32,
}

impl Default for PriorityFactors {
    fn default() -> Self {
        Self {
            distance_factor: 0.5,
            view_angle_factor: 0.5,
            screen_size_factor: 0.5,
            movement_speed_factor: 0.0,
            recency_factor: 0.5,
            importance_factor: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriorityConfig {
    
    pub distance_weight: f32,
    
    pub view_angle_weight: f32,
    
    pub screen_size_weight: f32,
    
    pub movement_speed_weight: f32,
    
    pub recency_weight: f32,
    
    pub importance_weight: f32,
    
    pub max_distance_threshold: f32,
    
    pub min_priority_threshold: f32,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            distance_weight: 0.3,
            view_angle_weight: 0.2,
            screen_size_weight: 0.25,
            movement_speed_weight: 0.1,
            recency_weight: 0.1,
            importance_weight: 0.05,
            max_distance_threshold: 1000.0,
            min_priority_threshold: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriorityCalculator {
    config: PriorityConfig,
}

impl PriorityCalculator {
    pub fn new() -> Self {
        Self {
            config: PriorityConfig::default(),
        }
    }
    
    pub fn with_config(config: PriorityConfig) -> Self {
        Self { config }
    }

    pub fn calculate_priority(&self, distance: f32, _camera_direction: &[f32; 3], resource_path: &str) -> StreamingPriority {

        if distance > self.config.max_distance_threshold {
            return StreamingPriority::Invisible;
        }
        
        let distance_normalized = (distance / self.config.max_distance_threshold).min(1.0);
        let distance_priority = 1.0 - distance_normalized;

        let final_priority = distance_priority * self.config.distance_weight + 
                           self.get_base_importance(resource_path) * self.config.importance_weight;
        
        self.priority_score_to_enum(final_priority)
    }

    pub fn calculate_priority_advanced(&self, factors: &PriorityFactors) -> StreamingPriority {
        let weighted_score = 
            factors.distance_factor * self.config.distance_weight +
            factors.view_angle_factor * self.config.view_angle_weight +
            factors.screen_size_factor * self.config.screen_size_weight +
            (1.0 - factors.movement_speed_factor) * self.config.movement_speed_weight +
            factors.recency_factor * self.config.recency_weight +
            factors.importance_factor * self.config.importance_weight;
        
        self.priority_score_to_enum(weighted_score)
    }

    pub fn calculate_distance_factor(&self, distance: f32) -> f32 {
        if distance >= self.config.max_distance_threshold {
            return 0.0;
        }
        
        let normalized = distance / self.config.max_distance_threshold;
        (1.0 - normalized).max(0.0)
    }

    pub fn calculate_view_angle_factor(&self, resource_direction: &[f32; 3], camera_direction: &[f32; 3]) -> f32 {
        
        let dot_product = resource_direction[0] * camera_direction[0] +
                         resource_direction[1] * camera_direction[1] +
                         resource_direction[2] * camera_direction[2];

        (dot_product + 1.0) / 2.0
    }

    pub fn calculate_screen_size_factor(&self, distance: f32, object_size: f32, fov: f32, screen_height: f32) -> f32 {
        if distance <= 0.0 {
            return 1.0;
        }

        let angular_size = (object_size / distance).atan();
        let screen_size = angular_size * screen_height / fov;
        
        (screen_size / screen_height).min(1.0)
    }

    pub fn calculate_movement_speed_factor(&self, camera_velocity: f32, max_velocity: f32) -> f32 {
        (camera_velocity / max_velocity).min(1.0)
    }

    pub fn calculate_recency_factor(&self, last_access: std::time::Instant) -> f32 {
        let elapsed = last_access.elapsed().as_secs_f32();
        let max_age = 300.0; 
        
        if elapsed >= max_age {
            0.0
        } else {
            1.0 - (elapsed / max_age)
        }
    }

    pub fn should_load_resource(&self, priority: StreamingPriority) -> bool {
        priority as u8 >= StreamingPriority::Low as u8
    }

    pub fn should_unload_resource(&self, priority: StreamingPriority, memory_pressure: f32) -> bool {
        if memory_pressure > 0.9 {
            
            priority as u8 <= StreamingPriority::Low as u8
        } else if memory_pressure > 0.7 {
            
            priority as u8 <= StreamingPriority::VeryLow as u8
        } else {
            
            priority == StreamingPriority::Invisible
        }
    }

    pub fn update_config(&mut self, config: PriorityConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &PriorityConfig {
        &self.config
    }

    fn priority_score_to_enum(&self, score: f32) -> StreamingPriority {
        if score < 0.1 {
            StreamingPriority::Invisible
        } else if score < 0.2 {
            StreamingPriority::VeryLow
        } else if score < 0.4 {
            StreamingPriority::Low
        } else if score < 0.6 {
            StreamingPriority::Medium
        } else if score < 0.8 {
            StreamingPriority::High
        } else {
            StreamingPriority::Critical
        }
    }

    fn get_base_importance(&self, resource_path: &str) -> f32 {
        
        if resource_path.contains("ui") || resource_path.contains("hud") {
            1.0 
        } else if resource_path.contains("character") || resource_path.contains("player") {
            0.9 
        } else if resource_path.contains("weapon") || resource_path.contains("item") {
            0.7 
        } else if resource_path.contains("environment") || resource_path.contains("terrain") {
            0.5 
        } else if resource_path.contains("particle") || resource_path.contains("effect") {
            0.3 
        } else {
            0.5 
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PriorityStats {
    pub invisible_count: u32,
    pub very_low_count: u32,
    pub low_count: u32,
    pub medium_count: u32,
    pub high_count: u32,
    pub critical_count: u32,
    pub total_resources: u32,
}

impl PriorityStats {
    pub fn add_resource(&mut self, priority: StreamingPriority) {
        match priority {
            StreamingPriority::Invisible => self.invisible_count += 1,
            StreamingPriority::VeryLow => self.very_low_count += 1,
            StreamingPriority::Low => self.low_count += 1,
            StreamingPriority::Medium => self.medium_count += 1,
            StreamingPriority::High => self.high_count += 1,
            StreamingPriority::Critical => self.critical_count += 1,
        }
        self.total_resources += 1;
    }
    
    pub fn get_priority_percentage(&self, priority: StreamingPriority) -> f32 {
        if self.total_resources == 0 {
            return 0.0;
        }
        
        let count = match priority {
            StreamingPriority::Invisible => self.invisible_count,
            StreamingPriority::VeryLow => self.very_low_count,
            StreamingPriority::Low => self.low_count,
            StreamingPriority::Medium => self.medium_count,
            StreamingPriority::High => self.high_count,
            StreamingPriority::Critical => self.critical_count,
        };
        
        (count as f32 / self.total_resources as f32) * 100.0
    }
    
    pub fn get_active_resources(&self) -> u32 {
        
        self.total_resources - self.invisible_count
    }
}
