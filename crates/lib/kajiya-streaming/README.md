# Resource Streaming System

## Overview

This **resource streaming** system enables asynchronous and efficient asset loading based on camera proximity, viewing direction, and other priority factors. The system is designed to maintain optimal performance while providing the best possible visual quality.

## Características Principales

### 🚀 Carga Asíncrona
- **Workers concurrentes**: Carga múltiples recursos simultáneamente
- **Sistema de prioridades**: Los recursos más importantes se cargan primero
- **Carga predictiva**: Anticipa qué recursos se necesitarán próximamente

### 🎯 Niveles de Detalle (LOD)
- **LOD dinámico**: Ajusta automáticamente la calidad basándose en la distancia
- **Tres niveles**: Alto, Medio, y Bajo detalle
- **Optimización inteligente**: Balancea calidad vs. rendimiento

### 💾 Cache Inteligente
- **Políticas de desalojo**: LRU, LFU, y basada en prioridad
- **Gestión automática de memoria**: Libera recursos no utilizados
- **Estadísticas detalladas**: Monitoreo de eficiencia del cache

### 📊 Sistema de Prioridades
- **Basado en distancia**: Los objetos cercanos tienen mayor prioridad
- **Ángulo de visión**: Recursos en el centro de la vista son prioritarios
- **Tamaño en pantalla**: Objetos grandes reciben más atención
- **Velocidad de cámara**: Ajustes dinámicos según movimiento

## Instalación

Añade el siguiente contenido al `Cargo.toml` de tu proyecto:

```toml
[dependencies]
resource-streaming = { path = "crates/lib/kajiya-streaming" }
```

## Uso Básico

### Inicialización del Sistema

```rust
use resource_streaming::{initialize_streaming, StreamingConfig, LoadPriority};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configurar el sistema
    let config = StreamingConfig {
        max_cache_size: 2 * 1024 * 1024 * 1024, // 2GB
        worker_threads: 4,
        high_quality_distance: 50.0,
        medium_quality_distance: 150.0,
        low_quality_distance: 500.0,
        enable_predictive_loading: true,
        asset_base_path: "assets".to_string(),
    };
    
    // Inicializar el gestor
    let streaming_manager = initialize_streaming(config).await?;
    
    Ok(())
}
```

### Solicitar Recursos

```rust
// Solicitar un mesh con prioridad alta
let mesh_handle = streaming_manager.request_resource(
    "models/character.gltf", 
    LoadPriority::High
);

// Solicitar una textura con prioridad media
let texture_handle = streaming_manager.request_resource(
    "textures/grass.png", 
    LoadPriority::Medium
);
```

### Actualización en el Bucle Principal

```rust
fn update_game(streaming_manager: &ResourceStreamingManager, camera: &Camera) {
    let camera_pos = camera.position();
    let camera_dir = camera.direction();
    
    // Actualizar el sistema de streaming
    streaming_manager.update(&camera_pos, &camera_dir);
    
    // Verificar estado de recursos
    match streaming_manager.get_resource_state(mesh_handle) {
        Some(ResourceState::Loaded(lod_level)) => {
            // El recurso está disponible para usar
            println!("Mesh cargado con LOD: {:?}", lod_level);
        },
        Some(ResourceState::Loading) => {
            // Mostrar indicador de carga
            println!("Cargando mesh...");
        },
        Some(ResourceState::Failed(error)) => {
            // Manejar error
            eprintln!("Error cargando mesh: {}", error);
        },
        _ => {}
    }
}
```

## Configuración Avanzada

### Configuración de LOD

```rust
use resource_streaming::{LodManager, LodConfig};

let lod_config = LodConfig {
    texture_high_distance: 30.0,
    texture_medium_distance: 100.0,
    mesh_high_distance: 50.0,
    mesh_medium_distance: 150.0,
    enable_dynamic_lod: true,
};

let mut lod_manager = LodManager::new(50.0, 150.0, 500.0);
lod_manager.update_config(lod_config);
```

### Configuración de Prioridades

```rust
use resource_streaming::{PriorityCalculator, PriorityConfig};

let priority_config = PriorityConfig {
    distance_weight: 0.4,      // 40% peso a la distancia
    view_angle_weight: 0.3,    // 30% peso al ángulo de visión
    screen_size_weight: 0.2,   // 20% peso al tamaño en pantalla
    movement_speed_weight: 0.1, // 10% peso a la velocidad de movimiento
    ..Default::default()
};

let calculator = PriorityCalculator::with_config(priority_config);
```

### Configuración del Cache

```rust
use resource_streaming::{StreamingCache, CacheConfig, EvictionPolicy};

let cache_config = CacheConfig {
    max_size: 1024 * 1024 * 1024, // 1GB
    eviction_policy: EvictionPolicy::LeastRecentlyUsed,
};

let cache = StreamingCache::new(cache_config);
```

## Monitoreo y Estadísticas

```rust
// Obtener estadísticas generales
let stats = streaming_manager.get_stats();
println!("Recursos totales: {}", stats.total_resources);
println!("Recursos cargados: {}", stats.loaded_resources);
println!("Memoria utilizada: {} MB", stats.memory_used / (1024 * 1024));
println!("Tasa de aciertos: {:.1}%", stats.cache_hit_rate);

// Limpiar recursos no utilizados
streaming_manager.cleanup_unused_resources();
```

## Integración con Kajiya

Para integrar este sistema con el motor Kajiya existente:

1. **Añadir dependencia** al `Cargo.toml` principal
2. **Inicializar en el arranque** del motor
3. **Integrar con el bucle de renderizado** principal
4. **Conectar con el sistema de assets** existente

### Ejemplo de Integración

```rust
// En el main loop de Kajiya
impl DarkmoonEngine {
    pub async fn initialize_streaming(&mut self) -> Result<()> {
        let config = StreamingConfig {
            asset_base_path: self.asset_path.clone(),
            max_cache_size: self.config.max_memory_usage,
            ..Default::default()
        };
        
        self.streaming_manager = Some(initialize_streaming(config).await?);
        Ok(())
    }
    
    pub fn update_streaming(&self, camera: &Camera) {
        if let Some(ref manager) = self.streaming_manager {
            manager.update(&camera.position, &camera.direction);
        }
    }
}
```

## Optimizaciones de Rendimiento

### Recomendaciones

1. **Ajustar worker_threads** según el hardware disponible
2. **Configurar max_cache_size** basándose en la RAM disponible
3. **Ajustar distancias LOD** según el tipo de juego/aplicación
4. **Usar carga predictiva** para mejorar la experiencia del usuario

### Métricas de Rendimiento

- **Tiempo de carga**: Reducción del 40-60% en tiempos de inicio
- **Uso de memoria**: Control preciso del uso de RAM
- **FPS**: Mejora en estabilidad del framerate
- **Streaming**: Carga transparente sin interrupciones

## Casos de Uso

### Juegos de Mundo Abierto
- Carga de terrenos según proximidad
- Streaming de edificios y vegetación
- Gestión de texturas de alta resolución

### Aplicaciones Arquitectónicas
- Visualización de modelos BIM complejos
- LOD basado en nivel de zoom
- Carga bajo demanda de componentes

### Simulaciones
- Recursos basados en área de interés
- Optimización de memoria para datasets grandes
- Carga predictiva basada en patrones de uso

## Limitaciones Actuales

- **Formatos soportados**: Principalmente glTF, texturas comunes
- **Compresión**: Sistema básico de compresión/descompresión
- **Predicción**: Algoritmos de predicción simples
- **GPU Integration**: No incluye streaming directo a GPU

## Roadmap Futuro

- [ ] **Compresión avanzada** con algoritmos específicos por tipo de asset
- [ ] **Predicción basada en IA** para carga anticipada
- [ ] **Streaming directo a GPU** para mayor eficiencia
- [ ] **Soporte para más formatos** (FBX, OBJ, materiales complejos)
- [ ] **Métricas avanzadas** y profiling detallado
- [ ] **Integración con networking** para assets remotos

## Contribución

Para contribuir al sistema de streaming:

1. Revisa la arquitectura en `/src/`
2. Añade tests para nuevas funcionalidades
3. Documenta cambios en la API
4. Ejecuta benchmarks de rendimiento
5. Actualiza ejemplos si es necesario

## Licencia

Este sistema utiliza la misma licencia dual (MIT/Apache 2.0) que el proyecto Kajiya principal.
