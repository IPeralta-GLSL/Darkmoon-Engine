[numthreads(8, 8, 1)]
void main(
    uint3 dispatch_thread_id : SV_DispatchThreadID,
    uint3 group_thread_id : SV_GroupThreadID,
    uint3 group_id : SV_GroupID
) {
    uint2 shading_rate_coord = dispatch_thread_id.xy;
    
    cbuffer push_constants : register(b0) {
        uint2 render_extent;
        uint2 tile_size;
    };
    
    Texture2D<float> depth_tex : register(t0);
    Texture2D<float2> velocity_tex : register(t1);
    RWTexture2D<uint> shading_rate_out : register(u0);
    
    SamplerState linear_sampler : register(s0);
    
    uint2 shading_rate_extent;
    shading_rate_out.GetDimensions(shading_rate_extent.x, shading_rate_extent.y);
    
    if (any(shading_rate_coord >= shading_rate_extent)) {
        return;
    }
    
    // Calculate the corresponding region in the full-resolution image
    uint2 region_start = shading_rate_coord * tile_size;
    uint2 region_end = min(region_start + tile_size, render_extent);
    
    // Sample depth and velocity in the region to determine shading rate
    float max_depth_variance = 0.0;
    float max_velocity_magnitude = 0.0;
    float avg_depth = 0.0;
    uint sample_count = 0;
    
    // Sample every 4th pixel to reduce workload
    for (uint y = region_start.y; y < region_end.y; y += 4) {
        for (uint x = region_start.x; x < region_end.x; x += 4) {
            float2 uv = (float2(x, y) + 0.5) / float2(render_extent);
            
            float depth = depth_tex.SampleLevel(linear_sampler, uv, 0).r;
            float2 velocity = velocity_tex.SampleLevel(linear_sampler, uv, 0).xy;
            
            if (depth > 0.0 && depth < 1.0) {
                avg_depth += depth;
                sample_count++;
                
                float velocity_mag = length(velocity);
                max_velocity_magnitude = max(max_velocity_magnitude, velocity_mag);
            }
        }
    }
    
    if (sample_count > 0) {
        avg_depth /= float(sample_count);
        
        // Calculate depth variance
        for (uint y = region_start.y; y < region_end.y; y += 4) {
            for (uint x = region_start.x; x < region_end.x; x += 4) {
                float2 uv = (float2(x, y) + 0.5) / float2(render_extent);
                float depth = depth_tex.SampleLevel(linear_sampler, uv, 0).r;
                
                if (depth > 0.0 && depth < 1.0) {
                    float depth_diff = abs(depth - avg_depth);
                    max_depth_variance = max(max_depth_variance, depth_diff);
                }
            }
        }
    }
    
    // Determine shading rate based on analysis
    uint shading_rate = 0; // VK_FRAGMENT_SHADING_RATE_1_INVOCATION_PER_PIXEL_NV (1x1)
    
    // High motion areas - reduce shading rate
    if (max_velocity_magnitude > 0.02) {
        shading_rate = 5; // 2x2
    }
    // Low depth variance areas (flat surfaces) - can reduce shading rate
    else if (max_depth_variance < 0.001) {
        shading_rate = 1; // 1x2
    }
    // Very flat areas with no motion - aggressive reduction
    else if (max_depth_variance < 0.0005 && max_velocity_magnitude < 0.005) {
        shading_rate = 5; // 2x2
    }
    
    shading_rate_out[shading_rate_coord] = shading_rate;
}
