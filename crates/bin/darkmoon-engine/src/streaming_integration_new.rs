use kajiya_streaming::{ResourceStreamingManager, StreamingConfig, LoadPriority};
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

/// Integración del sistema de streaming de recursos con Darkmoon Engine
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
            info!("Sistema de streaming ya inicializado");
            return Ok(());
        }
        
        info!("Inicializando sistema de resource streaming...");
        
        let config = StreamingConfig {
            max_cache_size: self.calculate_cache_size(),
            worker_threads: self.calculate_worker_threads(),
            high_quality_distance: 50.0,
            medium_quality_distance: 150.0,
            low_quality_distance: 500.0,
            enable_predictive_loading: true,
            asset_base_path: "assets".to_string(),
        };
        
        match kajiya_streaming::initialize_streaming(config) {
            Ok(manager) => {
                self.manager = Some(manager);
                self.enabled = true;
                info!("Sistema de streaming inicializado correctamente");
                Ok(())
            }
            Err(e) => {
                error!("Error inicializando sistema de streaming: {}", e);
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
            debug!("Sistema de streaming no inicializado, ignorando solicitud: {}", path);
            None
        }
    }
    
    /// Obtiene el estado de un recurso
    pub fn get_resource_state(&self, handle: u64) -> Option<kajiya_streaming::resource_manager::ResourceState> {
        if let Some(ref manager) = self.manager {
            manager.get_resource_state(handle)
        } else {
            None
        }
    }
    
    /// Obtiene estadísticas del sistema de streaming
    pub fn get_stats(&self) -> Option<kajiya_streaming::resource_manager::StreamingStats> {
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
            
            imgui::Window::new(imgui::im_str!("Resource Streaming"))
                .size([400.0, 300.0], imgui::Condition::FirstUseEver)
                .build(ui, || {
                    // Estado general
                    ui.separator();
                    ui.text("Estado del Sistema");
                    ui.separator();
                    
                    ui.text(format!("Estado: {}", if self.enabled { "Activo" } else { "Inactivo" }));
                    ui.text(format!("Total recursos: {}", stats.total_resources));
                    ui.text(format!("Recursos cargados: {}", stats.loaded_resources));
                    ui.text(format!("Recursos cargando: {}", stats.loading_resources));
                    ui.text(format!("Recursos fallidos: {}", stats.failed_resources));
                    
                    // Estadísticas de memoria
                    ui.separator();
                    ui.text("Uso de Memoria");
                    ui.separator();
                    
                    let memory_pct = (stats.memory_used as f64 / stats.memory_limit as f64 * 100.0) as f32;
                    ui.text(format!("Memoria usada: {:.1} MB / {:.1} MB ({:.1}%)", 
                           stats.memory_used as f32 / 1024.0 / 1024.0,
                           stats.memory_limit as f32 / 1024.0 / 1024.0,
                           memory_pct));
                    
                    imgui::ProgressBar::new(memory_pct / 100.0)
                        .size([300.0, 20.0])
                        .overlay_text(&imgui::im_str!("{:.1}%", memory_pct))
                        .build(ui);
                    
                    // Estadísticas de cache
                    ui.separator();
                    ui.text("Estadísticas de Cache");
                    ui.separator();
                    
                    ui.text(format!("Hit Rate: {:.1}%", stats.cache_hit_rate * 100.0));
                    
                    imgui::ProgressBar::new(stats.cache_hit_rate)
                        .size([300.0, 20.0])
                        .overlay_text(&imgui::im_str!("{:.1}%", stats.cache_hit_rate * 100.0))
                        .build(ui);
                    
                    // Controles
                    ui.separator();
                    ui.text("Controles");
                    ui.separator();
                    
                    if ui.button(imgui::im_str!("Limpiar Cache"), [120.0, 0.0]) {
                        if let Some(ref manager) = self.manager {
                            manager.clear_cache();
                            info!("Cache limpiado manualmente");
                        }
                    }
                    
                    ui.same_line(0.0);
                    if ui.button(imgui::im_str!("Forzar GC"), [120.0, 0.0]) {
                        if let Some(ref manager) = self.manager {
                            manager.force_garbage_collection();
                            info!("Garbage collection ejecutado manualmente");
                        }
                    }
                });
        } else {
            imgui::Window::new(imgui::im_str!("Resource Streaming"))
                .size([300.0, 180.0], imgui::Condition::FirstUseEver)
                .build(ui, || {
                    match &self.init_state {
                        StreamingInitState::NotInitialized => {
                            ui.text("Sistema de streaming no inicializado");
                            ui.spacing();
                            
                            if ui.button(imgui::im_str!("Inicializar Streaming"), [200.0, 0.0]) {
                                debug!("Botón inicializar presionado desde GUI");
                                initialize_clicked = true;
                            }
                        }
                        StreamingInitState::Initializing => {
                            ui.text("Inicializando sistema de streaming...");
                            ui.spacing();
                            
                            imgui::ProgressBar::new(-1.0) // Progreso indeterminado
                                .size([200.0, 20.0])
                                .overlay_text(&imgui::im_str!("Inicializando..."))
                                .build(ui);
                        }
                        StreamingInitState::Initialized => {
                            ui.text("Sistema inicializado pero manager no disponible");
                            ui.text("(Esto no debería suceder - bug interno)");
                        }
                        StreamingInitState::Failed(error) => {
                            ui.text_colored([1.0, 0.3, 0.3, 1.0], "Error en inicialización:");
                            ui.spacing();
                            ui.text_wrapped(&imgui::im_str!("{}", error));
                            ui.spacing();
                            
                            if ui.button(imgui::im_str!("Reintentar"), [100.0, 0.0]) {
                                self.init_state = StreamingInitState::NotInitialized;
                                self.init_requested = false;
                            }
                        }
                    }
                });
        }
        
        // Manejar el click del botón fuera de la closure
        if initialize_clicked {
            info!("Solicitando inicialización de streaming desde GUI");
            self.request_initialization();
        }
    }

    /// Solicita la inicialización del streaming (llamado desde GUI)
    pub fn request_initialization(&mut self) {
        if self.init_state == StreamingInitState::NotInitialized {
            self.init_requested = true;
            self.init_state = StreamingInitState::Initializing;
            info!("Inicialización de streaming solicitada desde GUI");
        }
    }
    
    /// Verifica si hay una solicitud de inicialización pendiente y la procesa
    pub async fn process_pending_initialization(&mut self) -> Result<()> {
        if self.init_requested && self.init_state == StreamingInitState::Initializing {
            self.init_requested = false;
            
            match self.initialize_internal().await {
                Ok(()) => {
                    self.init_state = StreamingInitState::Initialized;
                    info!("Sistema de streaming inicializado exitosamente desde GUI");
                }
                Err(e) => {
                    let error_msg = format!("Error inicializando streaming: {}", e);
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
            info!("Sistema de streaming ya inicializado");
            return Ok(());
        }
        
        info!("Inicializando sistema de resource streaming...");
        
        let config = StreamingConfig {
            max_cache_size: self.calculate_cache_size(),
            worker_threads: self.calculate_worker_threads(),
            high_quality_distance: 50.0,
            medium_quality_distance: 150.0,
            low_quality_distance: 500.0,
            enable_predictive_loading: true,
            asset_base_path: "assets".to_string(),
        };
        
        match kajiya_streaming::initialize_streaming(config) {
            Ok(manager) => {
                self.manager = Some(manager);
                self.enabled = true;
                info!("Sistema de streaming inicializado correctamente");
                Ok(())
            }
            Err(e) => {
                error!("Error inicializando sistema de streaming: {}", e);
                Err(e)
            }
        }
    }

    // Métodos privados para configuración
    
    fn calculate_cache_size(&self) -> u64 {
        // Calcular tamaño de cache basándose en la memoria disponible del sistema
        // Por ahora, usar un valor fijo de 2GB
        2 * 1024 * 1024 * 1024
    }
    
    fn calculate_worker_threads(&self) -> usize {
        // Usar la mitad de los cores disponibles para streaming
        (num_cpus::get() / 2).max(2).min(8)
    }
}

impl Default for StreamingIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Extensiones para el estado persistido para incluir configuración de streaming
pub trait PersistedStateStreamingExt {
    fn get_streaming_enabled(&self) -> bool;
    fn set_streaming_enabled(&mut self, enabled: bool);
    fn get_streaming_cache_size_mb(&self) -> u32;
    fn set_streaming_cache_size_mb(&mut self, size_mb: u32);
    fn get_streaming_worker_threads(&self) -> u8;
    fn set_streaming_worker_threads(&mut self, threads: u8);
}

// Implementación por defecto para PersistedState
#[allow(unused)]
impl PersistedStateStreamingExt for PersistedState {
    fn get_streaming_enabled(&self) -> bool {
        // Por defecto habilitado
        true
    }
    
    fn set_streaming_enabled(&mut self, enabled: bool) {
        // Implementar cuando se añadan campos al PersistedState
        debug!("Configurando streaming enabled: {}", enabled);
    }
    
    fn get_streaming_cache_size_mb(&self) -> u32 {
        // Por defecto 2GB
        2048
    }
    
    fn set_streaming_cache_size_mb(&mut self, size_mb: u32) {
        debug!("Configurando cache size: {} MB", size_mb);
    }
    
    fn get_streaming_worker_threads(&self) -> u8 {
        // Por defecto 4 threads
        4
    }
    
    fn set_streaming_worker_threads(&mut self, threads: u8) {
        debug!("Configurando worker threads: {}", threads);
    }
}
