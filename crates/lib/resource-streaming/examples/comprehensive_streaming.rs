//! Enhanced Resource Streaming Example
//! 
//! This example demonstrates basic usage of the resource streaming system.

use resource_streaming::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Basic Resource Streaming Example");

    // Create streaming configuration
    let config = StreamingConfig {
        max_cache_size: 256 * 1024 * 1024, // 256 MB cache
        worker_threads: 4,
        high_quality_distance: 100.0,
        medium_quality_distance: 300.0,
        low_quality_distance: 1000.0,
        enable_predictive_loading: true,
        asset_base_path: "assets".to_string(),
    };

    println!("Initializing resource streaming system...");
    let mut manager = initialize_streaming(config).await?;

    // Simulate basic resource loading
    simulate_basic_loading(&manager).await;

    // Demonstrate manual cache management
    demonstrate_cache_management(&manager);

    // Show statistics
    display_statistics(&manager);

    // Graceful shutdown
    println!("Shutting down streaming system...");
    manager.shutdown().await?;

    Ok(())
}

/// Simulates basic resource loading
async fn simulate_basic_loading(manager: &ResourceStreamingManager) {
    println!("\n=== Basic Resource Loading Demo ===");

    // Request various resources with different priorities
    let resources = vec![
        ("meshes/character.gltf", LoadPriority::Critical),
        ("textures/terrain.png", LoadPriority::High),
        ("audio/ambient.ogg", LoadPriority::Medium),
        ("meshes/building.gltf", LoadPriority::Low),
    ];

    // Request all resources
    let mut handles = Vec::new();
    for (path, priority) in &resources {
        let handle = manager.request_resource(path, *priority);
        handles.push((handle, *path));
        println!("Requested resource: {} with priority {:?}", path, priority);
    }

    // Simulate a few update cycles
    let camera_positions = [
        [0.0, 0.0, 0.0],
        [25.0, 5.0, 12.0],
        [50.0, 10.0, 25.0],
    ];

    for (i, &camera_pos) in camera_positions.iter().enumerate() {
        println!("\n--- Update {} - Camera at {:?} ---", i + 1, camera_pos);
        
        // Update streaming system
        let camera_direction = [0.0, 0.0, 1.0]; // Looking forward
        manager.update(&camera_pos, &camera_direction);

        // Check resource states
        for (handle, path) in &handles {
            if let Some(state) = manager.get_resource_state(*handle) {
                println!("  {}: {:?}", path, state);
            } else {
                println!("  {}: Not tracked", path);
            }
        }

        // Show streaming statistics
        let stats = manager.get_stats();
        println!("  Cache: {}/{} MB ({:.1}% hit rate)", 
                 stats.memory_used / 1024 / 1024,
                 stats.memory_limit / 1024 / 1024,
                 stats.cache_hit_rate * 100.0);

        // Simulate frame time
        sleep(Duration::from_millis(100)).await;
    }
}

/// Demonstrates manual cache management features
fn demonstrate_cache_management(manager: &ResourceStreamingManager) {
    println!("\n=== Cache Management Demo ===");

    let stats_before = manager.get_stats();
    println!("Cache before cleanup:");
    println!("  Resources: {} loaded, {} loading, {} failed",
             stats_before.loaded_resources,
             stats_before.loading_resources,
             stats_before.failed_resources);
    println!("  Memory: {} MB used", stats_before.memory_used / 1024 / 1024);

    // Force garbage collection
    manager.force_garbage_collection();
    println!("Executed garbage collection");

    // Show cache statistics after cleanup
    let stats_after = manager.get_stats();
    println!("Cache after cleanup:");
    println!("  Memory: {} MB used (saved {} MB)",
             stats_after.memory_used / 1024 / 1024,
             (stats_before.memory_used.saturating_sub(stats_after.memory_used)) / 1024 / 1024);

    // Demonstrate full cache clear
    manager.clear_cache();
    println!("Cleared entire cache");

    let stats_final = manager.get_stats();
    println!("Final cache state:");
    println!("  Memory: {} MB used", stats_final.memory_used / 1024 / 1024);
}

/// Displays comprehensive streaming statistics
fn display_statistics(manager: &ResourceStreamingManager) {
    println!("\n=== Final Statistics ===");
    
    let stats = manager.get_stats();
    
    println!("Resource counts:");
    println!("  Total: {}", stats.total_resources);
    println!("  Loaded: {}", stats.loaded_resources);
    println!("  Loading: {}", stats.loading_resources);
    println!("  Failed: {}", stats.failed_resources);
    
    println!("\nMemory usage:");
    println!("  Used: {:.2} MB", stats.memory_used as f64 / 1024.0 / 1024.0);
    println!("  Limit: {:.2} MB", stats.memory_limit as f64 / 1024.0 / 1024.0);
    println!("  Utilization: {:.1}%", 
             stats.memory_used as f64 / stats.memory_limit as f64 * 100.0);
    
    println!("\nCache performance:");
    println!("  Hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
    
    if stats.cache_hit_rate > 0.9 {
        println!("  ✅ Excellent cache performance!");
    } else if stats.cache_hit_rate > 0.7 {
        println!("  ✅ Good cache performance");
    } else {
        println!("  ⚠️  Cache performance could be improved");
    }
}


