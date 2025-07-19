use resource_streaming::{
    ResourceStreamingManager, StreamingConfig, LoadPriority,
    initialize_streaming
};
use anyhow::Result;

/// Ejemplo de uso del sistema de streaming de recursos
#[tokio::main]
async fn main() -> Result<()> {
    println!("Iniciando ejemplo de resource streaming...");
    
    // Configurar el sistema de streaming
    let config = StreamingConfig {
        max_cache_size: 1024 * 1024 * 1024, // 1GB
        worker_threads: 6,
        high_quality_distance: 30.0,
        medium_quality_distance: 100.0,
        low_quality_distance: 300.0,
        enable_predictive_loading: true,
        asset_base_path: "assets".to_string(),
    };
    
    // Inicializar el gestor de streaming
    let streaming_manager = initialize_streaming(config).await?;
    
    // Simular el bucle principal del juego
    simulate_game_loop(streaming_manager).await?;
    
    Ok(())
}

async fn simulate_game_loop(streaming_manager: ResourceStreamingManager) -> Result<()> {
    println!("Iniciando simulación del bucle principal del juego...");
    
    // Solicitar carga de varios recursos
    let _mesh_handle = streaming_manager.request_resource("meshes/character.gltf", LoadPriority::High);
    let _texture_handle = streaming_manager.request_resource("textures/character_diffuse.png", LoadPriority::High);
    let _environment_handle = streaming_manager.request_resource("environments/forest.gltf", LoadPriority::Medium);
    
    // Simular posición de cámara
    let mut camera_position = [0.0, 0.0, 0.0];
    let camera_direction = [0.0, 0.0, 1.0];
    
    // Simular 10 frames del juego
    for frame in 0..10 {
        println!("Frame {}", frame);
        
        // Mover la cámara hacia adelante
        camera_position[2] += 5.0;
        
        // Actualizar el sistema de streaming
        streaming_manager.update(&camera_position, &camera_direction);
        
        // Obtener y mostrar estadísticas
        let stats = streaming_manager.get_stats();
        println!("Estadísticas de streaming:");
        println!("  Total de recursos: {}", stats.total_resources);
        println!("  Recursos cargados: {}", stats.loaded_resources);
        println!("  Recursos cargando: {}", stats.loading_resources);
        println!("  Memoria utilizada: {} MB", stats.memory_used / (1024 * 1024));
        println!("  Tasa de aciertos del cache: {:.1}%", stats.cache_hit_rate);
        
        // Esperar un poco para simular el tiempo de frame
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60 FPS
    }
    
    println!("Simulación completada");
    Ok(())
}
