
use std::sync::Arc;
use kajiya_backend::{
    vulkan::{device::Device, shader::*},
    lazy_cache::LazyCache,
    pipeline_cache::PipelineCache,
};

#[test]
fn test_translucent_shader_compilation() {
    
    let lazy_cache = Arc::new(LazyCache::new());
    let mut pipeline_cache = PipelineCache::new(&lazy_cache);
    
    
    let shader_descs = vec![
        PipelineShaderDesc::builder(ShaderPipelineStage::Vertex)
            .hlsl_source("/shaders/translucent_vs.hlsl")
            .build()
            .unwrap(),
        PipelineShaderDesc::builder(ShaderPipelineStage::Pixel)
            .hlsl_source("/shaders/translucent_ps.hlsl")
            .build()
            .unwrap(),
    ];
    

    println!("Attempting to register translucent pipeline...");
    
    let handle = pipeline_cache.register_raster_pipeline(
        &shader_descs,
        RasterPipelineDesc::builder()
            .face_cull(false)
            .depth_write(false)
            .build()
            .unwrap(),
    );
    
    println!("Pipeline registered successfully: {:?}", handle);
}
