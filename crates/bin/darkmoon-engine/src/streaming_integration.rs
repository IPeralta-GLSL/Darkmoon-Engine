use resource_streaming::{ResourceStreamingManager, StreamingConfig, LoadPriority};
use crate::PersistedState;
use anyhow::Result;
use log::{info, debug, error};
use std::sync::Arc;
use parking_lot::RwLock;

/// Estado de inicialización del streaming
#[derive(Debug, Clone, PartialEq)]
pub enum StreamingInitState {
    NotInitialized,
    Initializing,
    Initialized,
    Failed(String),
}

/// Resource streaming system integration with Darkmoon Engine
pub struct StreamingIntegration {
    manager: Option<ResourceStreamingManager>,
    enabled: bool,
    init_state: StreamingInitState,
    init_requested: bool,
}

impl StreamingIntegration {
    pub fn new() -> Self {
        Self {
            manager: None,
            enabled: false,
            init_state: StreamingInitState::NotInitialized,
            init_requested: false,
        }
    }
    
    /// Inicializa el sistema de streaming
    pub async fn initialize(&mut self, persisted: &PersistedState) -> Result<()> {
        if self.enabled {
            info!("Streaming system already initialized");
            return Ok(());
        }
        
        info!("Initializing resource streaming system...");
        
        let config = StreamingConfig {
            max_cache_size: self.calculate_cache_size(),
            worker_threads: self.calculate_worker_threads(),
            high_quality_distance: 50.0,
            medium_quality_distance: 150.0,
            low_quality_distance: 500.0,
            enable_predictive_loading: true,
            asset_base_path: "assets".to_string(),
        };
        
        match resource_streaming::initialize_streaming(config) {
            Ok(manager) => {
                self.manager = Some(manager);
                self.enabled = true;
                info!("Streaming system initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Error initializing streaming system: {}", e);
                Err(e)
            }
        }
    }
    
    /// Actualiza el sistema de streaming en cada frame
    pub fn update(&self, camera_position: &[f32; 3], camera_direction: &[f32; 3]) {
        if let Some(ref manager) = self.manager {
            manager.update(camera_position, camera_direction);
        }
    }
    
    /// Solicita la carga de un recurso
    pub fn request_resource(&self, path: &str, priority: LoadPriority) -> Option<u64> {
        if let Some(ref manager) = self.manager {
            Some(manager.request_resource(path, priority))
        } else {
            debug!("Streaming system not initialized, ignoring request: {}", path);
            None
        }
    }
    
    /// Obtiene el estado de un recurso
    pub fn get_resource_state(&self, handle: u64) -> Option<resource_streaming::resource_manager::ResourceState> {
        if let Some(ref manager) = self.manager {
            manager.get_resource_state(handle)
        } else {
            None
        }
    }
    
    /// Obtiene estadísticas del sistema de streaming
    pub fn get_stats(&self) -> Option<resource_streaming::resource_manager::StreamingStats> {
        if let Some(ref manager) = self.manager {
            Some(manager.get_stats())
        } else {
            None
        }
    }
    
    /// Limpia recursos no utilizados
    pub fn cleanup_unused_resources(&self) {
        if let Some(ref manager) = self.manager {
            manager.cleanup_unused_resources();
        }
    }
    
    /// Verifica si el sistema de streaming está habilitado
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.manager.is_some()
    }
    
    /// Renderiza la GUI del sistema de streaming
    pub fn render_gui(&mut self, ui: &imgui::Ui) {
        let mut initialize_clicked = false;
        
        if let Some(ref manager) = self.manager {
            let stats = manager.get_stats();
            
            // System Status
            ui.separator();
            ui.text("System Status");
            ui.separator();
            
            ui.text(format!("Status: {}", if self.enabled { "Active" } else { "Inactive" }));
            ui.text(format!("Total resources: {}", stats.total_resources));
            ui.text(format!("Loaded resources: {}", stats.loaded_resources));
            ui.text(format!("Loading resources: {}", stats.loading_resources));
            ui.text(format!("Failed resources: {}", stats.failed_resources));
            
            // Memory Usage
            ui.separator();
            ui.text("Memory Usage");
            ui.separator();
            
            let memory_pct = if stats.memory_limit > 0 {
                (stats.memory_used as f64 / stats.memory_limit as f64 * 100.0) as f32
            } else {
                0.0
            };
            
            ui.text(format!("Memory used: {:.1} MB / {:.1} MB ({:.1}%)", 
                   stats.memory_used as f32 / 1024.0 / 1024.0,
                   stats.memory_limit as f32 / 1024.0 / 1024.0,
                   memory_pct));
            
            imgui::ProgressBar::new(memory_pct / 100.0)
                .size([300.0, 20.0])
                .overlay_text(&imgui::im_str!("{:.1}%", memory_pct))
                .build(ui);
            
            // Cache Statistics
            ui.separator();
            ui.text("Cache Statistics");
            ui.separator();
            
            let hit_rate_pct = stats.cache_hit_rate * 100.0;
            ui.text(format!("Hit Rate: {:.1}%", hit_rate_pct));
            
            imgui::ProgressBar::new(stats.cache_hit_rate)
                .size([300.0, 20.0])
                .overlay_text(&imgui::im_str!("{:.1}%", hit_rate_pct))
                .build(ui);
            
            // Controls
            ui.separator();
            ui.text("Controls");
            ui.separator();
            
            if ui.button(imgui::im_str!("Clear Cache"), [120.0, 0.0]) {
                if let Some(ref manager) = self.manager {
                    manager.clear_cache();
                    info!("Cache cleared manually");
                }
            }
            
            ui.same_line(0.0);
            if ui.button(imgui::im_str!("Force GC"), [120.0, 0.0]) {
                if let Some(ref manager) = self.manager {
                    manager.force_garbage_collection();
                    info!("Garbage collection executed manually");
                }
            }
        } else {
            match &self.init_state {
                StreamingInitState::NotInitialized => {
                    ui.text("Resource streaming system not initialized");
                    ui.spacing();
                    
                    if ui.button(imgui::im_str!("Initialize Streaming"), [200.0, 0.0]) {
                        debug!("Initialize button pressed from GUI");
                        initialize_clicked = true;
                    }
                }
                StreamingInitState::Initializing => {
                    ui.text("Initializing streaming system...");
                    ui.spacing();
                    
                    imgui::ProgressBar::new(-1.0) // Indeterminate progress
                        .size([200.0, 20.0])
                        .overlay_text(&imgui::im_str!("Initializing..."))
                        .build(ui);
                }
                StreamingInitState::Initialized => {
                    ui.text("System initialized but manager not available");
                    ui.text("(This should not happen - internal bug)");
                }
                StreamingInitState::Failed(error) => {
                    ui.text_colored([1.0, 0.3, 0.3, 1.0], "Initialization error:");
                    ui.spacing();
                    ui.text_wrapped(&imgui::im_str!("{}", error));
                    ui.spacing();
                    
                    if ui.button(imgui::im_str!("Retry"), [100.0, 0.0]) {
                        self.init_state = StreamingInitState::NotInitialized;
                        self.init_requested = false;
                    }
                }
            }
        }
        
        // Handle button click outside of closure
        if initialize_clicked {
            info!("Requesting streaming initialization from GUI");
            self.request_initialization();
        }
    }

    /// Solicita la inicialización del streaming (llamado desde GUI)
    pub fn request_initialization(&mut self) {
        if self.init_state == StreamingInitState::NotInitialized {
            self.init_requested = true;
            self.init_state = StreamingInitState::Initializing;
            info!("Streaming initialization requested from GUI");
        }
    }
    
    /// Verifica si hay una solicitud de inicialización pendiente y la procesa
    pub async fn process_pending_initialization(&mut self) -> Result<()> {
        if self.init_requested && self.init_state == StreamingInitState::Initializing {
            self.init_requested = false;
            
            match self.initialize_internal().await {
                Ok(()) => {
                    self.init_state = StreamingInitState::Initialized;
                    info!("Streaming system initialized successfully from GUI");
                }
                Err(e) => {
                    let error_msg = format!("Error initializing streaming: {}", e);
                    error!("{}", error_msg);
                    self.init_state = StreamingInitState::Failed(error_msg);
                }
            }
        }
        Ok(())
    }
    
    /// Inicialización interna del sistema de streaming
    async fn initialize_internal(&mut self) -> Result<()> {
        if self.enabled {
            info!("Streaming system already initialized");
            return Ok(());
        }
        
        info!("Initializing resource streaming system...");
        
        let config = StreamingConfig {
            max_cache_size: self.calculate_cache_size(),
            worker_threads: self.calculate_worker_threads(),
            high_quality_distance: 50.0,
            medium_quality_distance: 150.0,
            low_quality_distance: 500.0,
            enable_predictive_loading: true,
            asset_base_path: "assets".to_string(),
        };
        
        match resource_streaming::initialize_streaming(config) {
            Ok(manager) => {
                self.manager = Some(manager);
                self.enabled = true;
                info!("Streaming system initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Error initializing streaming system: {}", e);
                Err(e)
            }
        }
    }

    // Métodos privados para configuración
    
    fn calculate_cache_size(&self) -> u64 {
        // Calculate cache size based on available system memory
        // For now, use a fixed value of 2GB
        2 * 1024 * 1024 * 1024
    }
    
    fn calculate_worker_threads(&self) -> usize {
        // Use half of available cores for streaming
        (num_cpus::get() / 2).max(2).min(8)
    }
}

impl Default for StreamingIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Extensions for persisted state to include streaming configuration
pub trait PersistedStateStreamingExt {
    fn get_streaming_enabled(&self) -> bool;
    fn set_streaming_enabled(&mut self, enabled: bool);
    fn get_streaming_cache_size_mb(&self) -> u32;
    fn set_streaming_cache_size_mb(&mut self, size_mb: u32);
    fn get_streaming_worker_threads(&self) -> u8;
    fn set_streaming_worker_threads(&mut self, threads: u8);
}

// Default implementation for PersistedState
#[allow(unused)]
impl PersistedStateStreamingExt for PersistedState {
    fn get_streaming_enabled(&self) -> bool {
        // Default enabled
        true
    }
    
    fn set_streaming_enabled(&mut self, enabled: bool) {
        // Implement when fields are added to PersistedState
        debug!("Setting streaming enabled: {}", enabled);
    }
    
    fn get_streaming_cache_size_mb(&self) -> u32 {
        // Default 2GB
        2048
    }
    
    fn set_streaming_cache_size_mb(&mut self, size_mb: u32) {
        debug!("Setting cache size: {} MB", size_mb);
    }
    
    fn get_streaming_worker_threads(&self) -> u8 {
        // Default 4 threads
        4
    }
    
    fn set_streaming_worker_threads(&mut self, threads: u8) {
        debug!("Setting worker threads: {}", threads);
    }
}
