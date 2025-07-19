use crate::{ResourceId};
use std::collections::HashMap;
use bytesize::ByteSize;
use log::{debug, warn};

/// Política de desalojo del cache
#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    /// Least Recently Used - remueve los elementos menos recientemente usados
    LeastRecentlyUsed,
    /// Least Frequently Used - remueve los elementos menos frecuentemente usados
    LeastFrequentlyUsed,
    /// Basado en prioridad - remueve elementos de menor prioridad
    Priority,
}

/// Configuración del cache de streaming
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: u64,
    pub eviction_policy: EvictionPolicy,
}

/// Entrada del cache con metadatos
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    access_count: u32,
    last_accessed: std::time::Instant,
    size: u64,
    priority: u8,
}

/// Cache inteligente para recursos de streaming
pub struct StreamingCache {
    config: CacheConfig,
    entries: HashMap<ResourceId, CacheEntry>,
    current_size: u64,
    hit_count: u64,
    miss_count: u64,
}

impl StreamingCache {
    pub fn new(config: CacheConfig) -> Self {
        debug!("Inicializando cache de streaming con límite: {}", 
               ByteSize(config.max_size));
        
        Self {
            config,
            entries: HashMap::new(),
            current_size: 0,
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    /// Inserta un recurso en el cache
    pub fn insert(&mut self, resource_id: ResourceId, data: Vec<u8>) {
        let size = data.len() as u64;
        
        // Si el recurso es demasiado grande para el cache, no lo almacenamos
        if size > self.config.max_size {
            warn!("Recurso {} es demasiado grande para el cache ({} > {})", 
                  resource_id, ByteSize(size), ByteSize(self.config.max_size));
            return;
        }
        
        // Hacer espacio si es necesario
        self.make_space_for(size);
        
        let entry = CacheEntry {
            data,
            access_count: 1,
            last_accessed: std::time::Instant::now(),
            size,
            priority: 5, // Prioridad media por defecto
        };
        
        // Si el recurso ya existía, actualizar el tamaño total
        if let Some(old_entry) = self.entries.insert(resource_id.clone(), entry) {
            self.current_size -= old_entry.size;
        }
        
        self.current_size += size;
        
        debug!("Recurso {} insertado en cache. Uso actual: {}/{}", 
               resource_id, 
               ByteSize(self.current_size), 
               ByteSize(self.config.max_size));
    }
    
    /// Obtiene un recurso del cache
    pub fn get(&mut self, resource_id: &ResourceId) -> Option<&Vec<u8>> {
        if let Some(entry) = self.entries.get_mut(resource_id) {
            entry.access_count += 1;
            entry.last_accessed = std::time::Instant::now();
            self.hit_count += 1;
            Some(&entry.data)
        } else {
            self.miss_count += 1;
            None
        }
    }
    
    /// Remueve un recurso del cache
    pub fn remove(&mut self, resource_id: &ResourceId) -> bool {
        if let Some(entry) = self.entries.remove(resource_id) {
            self.current_size -= entry.size;
            debug!("Recurso {} removido del cache", resource_id);
            true
        } else {
            false
        }
    }
    
    /// Verifica si un recurso está en el cache
    pub fn contains(&self, resource_id: &ResourceId) -> bool {
        self.entries.contains_key(resource_id)
    }
    
    /// Obtiene el uso actual de memoria del cache
    pub fn current_size(&self) -> u64 {
        self.current_size
    }
    
    /// Obtiene el tamaño máximo del cache
    pub fn max_size(&self) -> u64 {
        self.config.max_size
    }
    
    /// Obtiene el porcentaje de uso del cache
    pub fn usage_percentage(&self) -> f32 {
        if self.config.max_size == 0 {
            0.0
        } else {
            (self.current_size as f32 / self.config.max_size as f32) * 100.0
        }
    }
    
    /// Obtiene la tasa de aciertos del cache
    pub fn get_hit_rate(&self) -> f32 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            (self.hit_count as f32 / total as f32) * 100.0
        }
    }
    
    /// Limpia todo el cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
        self.hit_count = 0;
        self.miss_count = 0;
        debug!("Cache completamente limpiado");
    }
    
    /// Hace espacio en el cache para un nuevo recurso de tamaño específico
    fn make_space_for(&mut self, required_size: u64) {
        while self.current_size + required_size > self.config.max_size && !self.entries.is_empty() {
            if let Some(resource_id) = self.select_victim() {
                self.remove(&resource_id);
            } else {
                break;
            }
        }
    }
    
    /// Selecciona una víctima para desalojo según la política configurada
    fn select_victim(&self) -> Option<ResourceId> {
        match self.config.eviction_policy {
            EvictionPolicy::LeastRecentlyUsed => self.select_lru_victim(),
            EvictionPolicy::LeastFrequentlyUsed => self.select_lfu_victim(),
            EvictionPolicy::Priority => self.select_priority_victim(),
        }
    }
    
    /// Selecciona la víctima LRU (Least Recently Used)
    fn select_lru_victim(&self) -> Option<ResourceId> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, _)| id.clone())
    }
    
    /// Selecciona la víctima LFU (Least Frequently Used)
    fn select_lfu_victim(&self) -> Option<ResourceId> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(id, _)| id.clone())
    }
    
    /// Selecciona la víctima basada en prioridad
    fn select_priority_victim(&self) -> Option<ResourceId> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.priority)
            .map(|(id, _)| id.clone())
    }
    
    /// Establece la prioridad de un recurso en el cache
    pub fn set_priority(&mut self, resource_id: &ResourceId, priority: u8) {
        if let Some(entry) = self.entries.get_mut(resource_id) {
            entry.priority = priority;
        }
    }
    
    /// Obtiene estadísticas detalladas del cache
    pub fn get_detailed_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            current_size: self.current_size,
            max_size: self.config.max_size,
            usage_percentage: self.usage_percentage(),
            hit_rate: self.get_hit_rate(),
            hit_count: self.hit_count,
            miss_count: self.miss_count,
        }
    }
    
    /// Ejecuta limpieza del cache (elimina entradas antiguas)
    pub fn cleanup(&mut self) {
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(300); // 5 minutos
        let mut to_remove = Vec::new();
        
        for (id, entry) in &self.entries {
            if entry.last_accessed < cutoff && entry.access_count <= 1 {
                to_remove.push(id.clone());
            }
        }
        
        for id in to_remove {
            self.remove(&id);
        }
    }
    
    /// Obtiene el uso actual de memoria del cache
    pub fn get_memory_usage(&self) -> u64 {
        self.current_size
    }
}

/// Estadísticas detalladas del cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub current_size: u64,
    pub max_size: u64,
    pub usage_percentage: f32,
    pub hit_rate: f32,
    pub hit_count: u64,
    pub miss_count: u64,
}
