use resource_streaming::{
    ResourceStreamingManager, StreamingConfig, LoadPriority,
    initialize_streaming
};
use anyhow::Result;


fn main() -> Result<()> {
    println!("Starting resource streaming example...");
    

    let config = StreamingConfig {
        max_cache_size: 1024 * 1024 * 1024, // 1GB
        worker_threads: 6,
        high_quality_distance: 30.0,
        medium_quality_distance: 100.0,
        low_quality_distance: 300.0,
        enable_predictive_loading: true,
        asset_base_path: "assets".to_string(),
    };
    

    let streaming_manager = initialize_streaming(config)?;
    

    simulate_game_loop(streaming_manager)?;
    
    Ok(())
}

fn simulate_game_loop(streaming_manager: ResourceStreamingManager) -> Result<()> {
    println!("Starting game loop simulation...");
    

    let _mesh_handle = streaming_manager.request_resource("meshes/character.gltf", LoadPriority::High);
    let _texture_handle = streaming_manager.request_resource("textures/character_diffuse.png", LoadPriority::High);
    let _environment_handle = streaming_manager.request_resource("environments/forest.gltf", LoadPriority::Medium);
    

    let mut camera_position = [0.0, 0.0, 0.0];
    let camera_direction = [0.0, 0.0, 1.0];
    

    for frame in 0..10 {
        println!("Frame {}", frame);
        

        camera_position[2] += 5.0;
        
        streaming_manager.update(&camera_position, &camera_direction);
        
        let stats = streaming_manager.get_stats();
        println!("Streaming statistics:");
        println!("  Total resources: {}", stats.total_resources);
        println!("  Loaded resources: {}", stats.loaded_resources);
        println!("  Loading resources: {}", stats.loading_resources);
        println!("  Memory used: {} MB", stats.memory_used / (1024 * 1024));
        println!("  Cache hit rate: {:.1}%", stats.cache_hit_rate);
        

        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
    
    println!("Simulation completed");
    Ok(())
}
