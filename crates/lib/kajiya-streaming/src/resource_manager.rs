use crate::{StreamingConfig, ResourceId, ResourceHandle};
use crate::streaming_cache::{StreamingCache, CacheConfig};
use crate::asset_loader::{AssetLoader, LoadRequest, LoadPriority};
use crate::level_of_detail::{LodManager, LodLevel};
use crate::priority_system::{PriorityCalculator, StreamingPriority};

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{info, debug, warn};

/// Estado de un recurso en el sistema de streaming
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState {
    /// Recurso no cargado
    NotLoaded,
    /// Recurso en proceso de carga
    Loading,
    /// Recurso cargado con nivel de detalle específico
    Loaded(LodLevel),
    /// Error al cargar el recurso
    Failed(String),
}

/// Información de un recurso gestionado
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub id: ResourceId,
    pub handle: ResourceHandle,
    pub path: String,
    pub state: ResourceState,
    pub priority: StreamingPriority,
    pub last_accessed: std::time::Instant,
    pub memory_usage: u64,
}

/// Gestor principal del sistema de streaming de recursos
pub struct ResourceStreamingManager {
    config: StreamingConfig,
    cache: Arc<RwLock<StreamingCache>>,
    asset_loader: AssetLoader,
    lod_manager: LodManager,
    priority_calculator: PriorityCalculator,
    
    // Estado interno
    resources: Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
    load_queue: Arc<RwLock<Vec<LoadRequest>>>,
    
    // Canal de comunicación para solicitudes de carga
    load_sender: Sender<LoadRequest>,
    load_receiver: Arc<parking_lot::Mutex<Option<Receiver<LoadRequest>>>>,
    
    // Control del background worker
    worker_shutdown: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
    
    // Estadísticas
    stats: Arc<RwLock<StreamingStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct StreamingStats {
    pub total_resources: usize,
    pub loaded_resources: usize,
    pub loading_resources: usize,
    pub failed_resources: usize,
    pub cache_hit_rate: f32,
    pub memory_used: u64,
    pub memory_limit: u64,
}

impl ResourceStreamingManager {
    pub fn new(config: StreamingConfig) -> Result<Self> {
        info!("Inicializando sistema de streaming de recursos...");
        
        let cache_config = CacheConfig {
            max_size: config.max_cache_size,
            eviction_policy: crate::streaming_cache::EvictionPolicy::LeastRecentlyUsed,
        };
        
        let cache = Arc::new(RwLock::new(StreamingCache::new(cache_config)));
        let asset_loader = AssetLoader::new(config.worker_threads, &config.asset_base_path)?;
        let lod_manager = LodManager::new(
            config.high_quality_distance,
            config.medium_quality_distance,
            config.low_quality_distance,
        );
        let priority_calculator = PriorityCalculator::new();
        
        let (load_sender, load_receiver) = unbounded::<LoadRequest>();
        
        let resources = Arc::new(RwLock::new(HashMap::new()));
        let load_queue = Arc::new(RwLock::new(Vec::new()));
        let stats = Arc::new(RwLock::new(StreamingStats::default()));
        let worker_shutdown = Arc::new(AtomicBool::new(false));
        
        // Crear el gestor
        let mut manager = Self {
            config: config.clone(),
            cache,
            asset_loader,
            lod_manager,
            priority_calculator,
            resources: resources.clone(),
            load_queue: load_queue.clone(),
            load_sender,
            load_receiver: Arc::new(parking_lot::Mutex::new(Some(load_receiver))),
            worker_shutdown: worker_shutdown.clone(),
            worker_handle: None,
            stats: stats.clone(),
        };
        
        // Iniciar el worker en background
        manager.start_background_worker()?;
        
        info!("Sistema de streaming inicializado con éxito");
        Ok(manager)
    }
    
    /// Inicia el worker en background para procesamiento de carga
    fn start_background_worker(&mut self) -> Result<()> {
        let load_receiver = self.load_receiver
            .lock()
            .take()
            .ok_or_else(|| anyhow::anyhow!("Load receiver already taken"))?;
            
        let resources = self.resources.clone();
        let cache = self.cache.clone();
        let asset_loader = self.asset_loader.clone();
        let lod_manager = self.lod_manager.clone();
        let stats = self.stats.clone();
        let shutdown = self.worker_shutdown.clone();
        
        let handle = std::thread::spawn(move || {
            info!("Background streaming worker iniciado");
            
            while !shutdown.load(Ordering::Relaxed) {
                // Procesar solicitudes de carga con timeout
                match load_receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(load_request) => {
                        futures::executor::block_on(Self::process_load_request(
                            load_request,
                            &resources,
                            &cache,
                            &asset_loader,
                            &lod_manager,
                            &stats,
                        ));
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Timeout - realizar tareas de mantenimiento
                        futures::executor::block_on(Self::perform_maintenance(&resources, &cache, &stats));
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        debug!("Load receiver closed, shutting down worker");
                        break;
                    }
                }
            }
            
            info!("Background streaming worker terminado");
        });
        
        self.worker_handle = Some(handle);
        Ok(())
    }
    
    /// Procesa una solicitud de carga de recurso en background
    async fn process_load_request(
        request: LoadRequest,
        resources: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        cache: &Arc<RwLock<StreamingCache>>,
        asset_loader: &AssetLoader,
        lod_manager: &LodManager,
        stats: &Arc<RwLock<StreamingStats>>,
    ) {
        debug!("Procesando solicitud de carga: {:?}", request.path);
        
        // Verificar si ya está en cache
        {
            let cache_read = cache.read();
            if cache_read.contains(&request.path) {
                debug!("Recurso encontrado en cache: {}", request.path);
                Self::update_resource_state(
                    &request.path,
                    ResourceState::Loaded(request.lod_level),
                    resources,
                );
                Self::update_stats(stats, 1, 0, 0, 0);
                return;
            }
        }
        
        // Marcar como loading
        Self::update_resource_state(&request.path, ResourceState::Loading, resources);
        
        // Intentar cargar el recurso
        match asset_loader.load_asset(&request).await {
            Ok(asset_data) => {
                // Cargar exitosamente - agregar al cache
                {
                    let mut cache_write = cache.write();
                    cache_write.insert(request.path.clone(), asset_data.data);
                }
                
                Self::update_resource_state(
                    &request.path,
                    ResourceState::Loaded(request.lod_level),
                    resources,
                );
                Self::update_stats(stats, 1, -1, 0, 0);
                
                info!("Recurso cargado exitosamente: {}", request.path);
            }
            Err(err) => {
                let error_msg = format!("Error cargando {}: {}", request.path, err);
                warn!("{}", error_msg);
                
                Self::update_resource_state(
                    &request.path,
                    ResourceState::Failed(error_msg),
                    resources,
                );
                Self::update_stats(stats, 0, -1, 1, 0);
            }
        }
    }
    
    /// Actualiza el estado de un recurso
    fn update_resource_state(
        resource_id: &str,
        new_state: ResourceState,
        resources: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
    ) {
        let mut resources_write = resources.write();
        if let Some(info) = resources_write.get_mut(resource_id) {
            info.state = new_state;
            info.last_accessed = std::time::Instant::now();
        }
    }
    
    /// Actualiza las estadísticas
    fn update_stats(
        stats: &Arc<RwLock<StreamingStats>>,
        loaded_delta: i32,
        loading_delta: i32,
        failed_delta: i32,
        memory_delta: i64,
    ) {
        let mut stats_write = stats.write();
        stats_write.loaded_resources = (stats_write.loaded_resources as i32 + loaded_delta).max(0) as usize;
        stats_write.loading_resources = (stats_write.loading_resources as i32 + loading_delta).max(0) as usize;
        stats_write.failed_resources = (stats_write.failed_resources as i32 + failed_delta).max(0) as usize;
        stats_write.memory_used = (stats_write.memory_used as i64 + memory_delta).max(0) as u64;
    }
    
    /// Realiza tareas de mantenimiento periódico
    async fn perform_maintenance(
        _resources: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        cache: &Arc<RwLock<StreamingCache>>,
        stats: &Arc<RwLock<StreamingStats>>,
    ) {
        // Ejecutar limpieza del cache
        {
            let mut cache_write = cache.write();
            cache_write.cleanup();
        }
        
        // Actualizar estadísticas del cache
        {
            let cache_read = cache.read();
            let mut stats_write = stats.write();
            stats_write.memory_used = cache_read.get_memory_usage();
        }
    }
    
    /// Solicita la carga de un recurso con prioridad específica
    pub fn request_resource(&self, path: &str, priority: LoadPriority) -> ResourceHandle {
        let resource_id = path.to_string();
        let handle = self.generate_handle(&resource_id);
        
        let mut resources = self.resources.write();
        
        // Si el recurso ya existe, actualizar prioridad si es mayor
        if let Some(info) = resources.get_mut(&resource_id) {
            info.last_accessed = std::time::Instant::now();
            if priority as u8 > info.priority as u8 {
                info.priority = priority.into();
            }
            return info.handle;
        }
        
        // Crear nueva información del recurso
        let resource_info = ResourceInfo {
            id: resource_id.clone(),
            handle,
            path: path.to_string(),
            state: ResourceState::Loading,
            priority: priority.into(),
            last_accessed: std::time::Instant::now(),
            memory_usage: 0,
        };
        
        resources.insert(resource_id.clone(), resource_info);
        
        // Enviar solicitud de carga al worker en background
        let load_request = LoadRequest {
            resource_id: resource_id.clone(),
            path: path.to_string(),
            priority,
            lod_level: self.lod_manager.calculate_lod_level(100.0, &crate::level_of_detail::ResourceType::Other), // TODO: usar posición real y tipo correcto
        };
        
        if let Err(e) = self.load_sender.send(load_request) {
            warn!("Error enviando solicitud de carga para {}: {}", path, e);
            // Actualizar estado a fallido
            if let Some(info) = resources.get_mut(&resource_id) {
                info.state = ResourceState::Failed(format!("Error enviando solicitud: {}", e));
            }
        }
        
        handle
    }
    
    /// Actualiza el sistema de streaming basado en la posición de la cámara
    pub fn update(&self, camera_position: &[f32; 3], camera_direction: &[f32; 3]) {
        debug!("Actualizando sistema de streaming desde posición {:?}", camera_position);
        
        // Calcular prioridades basadas en distancia y dirección de la cámara
        let mut resources = self.resources.write();
        for (_, resource_info) in resources.iter_mut() {
            // Aquí calcularías la distancia del recurso a la cámara
            // Por ahora usamos un placeholder
            let distance = self.calculate_resource_distance(&resource_info.path, camera_position);
            let new_priority = self.priority_calculator.calculate_priority(
                distance,
                camera_direction,
                &resource_info.path,
            );
            
            resource_info.priority = new_priority;
            resource_info.last_accessed = std::time::Instant::now();
        }
        
        // Actualizar estadísticas
        self.update_instance_stats();
        
        // Limpiar recursos no utilizados si es necesario
        self.cleanup_unused_resources();
    }
    
    /// Obtiene el estado de un recurso
    pub fn get_resource_state(&self, handle: ResourceHandle) -> Option<ResourceState> {
        let resources = self.resources.read();
        resources.values()
            .find(|info| info.handle == handle)
            .map(|info| info.state.clone())
    }
    
    /// Obtiene las estadísticas actuales del streaming
    pub fn get_stats(&self) -> StreamingStats {
        (*self.stats.read()).clone()
    }
    
    /// Limpia recursos no utilizados del cache
    pub fn cleanup_unused_resources(&self) {
        let now = std::time::Instant::now();
        let mut resources = self.resources.write();
        let mut cache = self.cache.write();
        
        let mut to_remove = Vec::new();
        for (id, info) in resources.iter() {
            // Remover recursos no accedidos en los últimos 5 minutos
            if now.duration_since(info.last_accessed).as_secs() > 300 {
                to_remove.push(id.clone());
            }
        }
        
        for id in to_remove {
            debug!("Removiendo recurso no utilizado: {}", id);
            resources.remove(&id);
            cache.remove(&id);
        }
    }
    
    /// Cierra el sistema de streaming y limpia recursos
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Cerrando sistema de streaming...");
        
        // Señalar al worker que se cierre
        self.worker_shutdown.store(true, Ordering::Relaxed);
        
        // Esperar a que el worker termine
        if let Some(handle) = self.worker_handle.take() {
            if let Err(e) = handle.join() {
                warn!("Error esperando el worker: {:?}", e);
            }
        }
        
        info!("Sistema de streaming cerrado");
        Ok(())
    }

    /// Limpia el cache manualmente
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        info!("Cache limpiado manualmente");
    }
    
    /// Fuerza la recolección de basura en el cache
    pub fn force_garbage_collection(&self) {
        let mut cache = self.cache.write();
        cache.cleanup();
        info!("Garbage collection ejecutado manualmente");
    }

    // Métodos privados
    
    fn generate_handle(&self, resource_id: &str) -> ResourceHandle {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        resource_id.hash(&mut hasher);
        hasher.finish()
    }
    
    fn calculate_resource_distance(&self, _resource_path: &str, _camera_position: &[f32; 3]) -> f32 {
        // Placeholder - en una implementación real, calcularías la distancia
        // basada en la posición del recurso en el mundo
        100.0
    }
    
    fn update_instance_stats(&self) {
        let resources = self.resources.read();
        let mut stats = self.stats.write();
        
        stats.total_resources = resources.len();
        stats.loaded_resources = resources.values().filter(|r| matches!(r.state, ResourceState::Loaded(_))).count();
        stats.loading_resources = resources.values().filter(|r| matches!(r.state, ResourceState::Loading)).count();
        stats.failed_resources = resources.values().filter(|r| matches!(r.state, ResourceState::Failed(_))).count();
        stats.memory_used = resources.values().map(|r| r.memory_usage).sum();
        stats.memory_limit = self.config.max_cache_size;
        
        // Calcular hit rate del cache
        let cache = self.cache.read();
        stats.cache_hit_rate = cache.get_hit_rate();
    }
}

impl Clone for ResourceStreamingManager {
    fn clone(&self) -> Self {
        // Para clonado, no incluimos el worker handle ni receiver
        Self {
            config: self.config.clone(),
            cache: self.cache.clone(),
            asset_loader: self.asset_loader.clone(),
            lod_manager: self.lod_manager.clone(),
            priority_calculator: self.priority_calculator.clone(),
            resources: self.resources.clone(),
            load_queue: self.load_queue.clone(),
            load_sender: self.load_sender.clone(),
            load_receiver: Arc::new(parking_lot::Mutex::new(None)),
            worker_shutdown: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
            stats: self.stats.clone(),
        }
    }
}
