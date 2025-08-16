#include "inc/frame_constants.hlsl"
#include "inc/uv.hlsl"
#include "inc/samplers.hlsl"
#include "inc/bilinear.hlsl"
#include "inc/color.hlsl"

[[vk::binding(0)]] Texture2D<float4> current_frame_tex;
[[vk::binding(1)]] Texture2D<float4> history_tex;
[[vk::binding(2)]] Texture2D<float4> reprojection_tex;
[[vk::binding(3)]] Texture2D<float> depth_tex;
[[vk::binding(4)]] Texture2D<float3> normal_tex;
[[vk::binding(5)]] Texture2D<float4> variance_tex;
[[vk::binding(6)]] RWTexture2D<float4> output_tex;
[[vk::binding(7)]] RWTexture2D<float4> variance_output_tex;
[[vk::binding(8)]] cbuffer TemporalReprojectionConstants {
    float4 output_tex_size;
    float history_alpha;
    float motion_threshold;
    float depth_threshold;
    float normal_threshold;
    uint max_temporal_samples;
    uint enable_variance_guided;
    uint enable_depth_validation;
    uint enable_normal_validation;
};

// Calculate temporal weight based on motion, depth, and normal similarity
float calculate_temporal_weight(
    float2 motion,
    float current_depth,
    float history_depth,
    float3 current_normal,
    float3 history_normal
) {
    float weight = history_alpha;
    
    // Motion-based rejection
    float motion_magnitude = length(motion);
    if (motion_magnitude > motion_threshold) {
        weight *= saturate(1.0 - (motion_magnitude - motion_threshold) * 10.0);
    }
    
    // Depth-based validation
    if (enable_depth_validation) {
        float depth_similarity = 1.0 - abs(current_depth - history_depth) / max(current_depth, history_depth);
        depth_similarity = saturate((depth_similarity - depth_threshold) / (1.0 - depth_threshold));
        weight *= depth_similarity * depth_similarity;
    }
    
    // Normal-based validation
    if (enable_normal_validation) {
        float normal_similarity = saturate(dot(current_normal, history_normal));
        normal_similarity = saturate((normal_similarity - normal_threshold) / (1.0 - normal_threshold));
        weight *= pow(normal_similarity, 1.5);
    }
    
    return saturate(weight);
}

// Enhanced bilateral filter for variance estimation
float4 calculate_variance(uint2 px, float4 current_color) {
    float4 mean = 0;
    float4 variance = 0;
    float total_weight = 0;
    
    const int radius = 2;
    for (int y = -radius; y <= radius; ++y) {
        for (int x = -radius; x <= radius; ++x) {
            uint2 sample_px = px + int2(x, y);
            if (all(sample_px >= 0) && all(sample_px < uint2(output_tex_size.xy))) {
                float4 sample_color = current_frame_tex[sample_px];
                float weight = exp(-0.5 * (x*x + y*y) / (radius * radius * 0.5));
                
                mean += sample_color * weight;
                variance += sample_color * sample_color * weight;
                total_weight += weight;
            }
        }
    }
    
    mean /= total_weight;
    variance = variance / total_weight - mean * mean;
    
    return variance;
}

[numthreads(8, 8, 1)]
void main(uint2 px : SV_DispatchThreadID) {
    if (any(px >= uint2(output_tex_size.xy))) {
        return;
    }
    
    float2 uv = get_uv(px, output_tex_size);
    
    // Current frame data
    float4 current_color = current_frame_tex[px];
    float current_depth = depth_tex[px];
    float3 current_normal = normalize(normal_tex[px] * 2.0 - 1.0);
    
    // Skip background pixels
    if (current_depth == 0.0) {
        output_tex[px] = current_color;
        variance_output_tex[px] = float4(0, 0, 0, 1);
        return;
    }
    
    // Reprojection data
    float4 reproj_data = reprojection_tex[px];
    float2 motion = reproj_data.xy;
    float reproj_validity = reproj_data.z;
    float2 history_uv = uv + motion;
    
    // Default output
    float4 final_color = current_color;
    float4 output_variance = float4(0, 0, 0, 1);
    
    // Temporal reprojection if valid
    if (reproj_validity > 0.5 && all(history_uv >= 0.0) && all(history_uv <= 1.0)) {
        // Sample history with bilinear filtering
        float4 history_color = history_tex.SampleLevel(sampler_lnc, history_uv, 0);
        
        // Get history depth and normal for validation
        float history_depth = depth_tex.SampleLevel(sampler_nnc, history_uv, 0);
        float3 history_normal = normalize(normal_tex.SampleLevel(sampler_lnc, history_uv, 0) * 2.0 - 1.0);
        
        // Calculate temporal weight
        float temporal_weight = calculate_temporal_weight(
            motion, current_depth, history_depth, current_normal, history_normal
        );
        
        // Variance-guided adaptive blending
        if (enable_variance_guided) {
            float4 current_variance = calculate_variance(px, current_color);
            float4 history_variance = variance_tex.SampleLevel(sampler_lnc, history_uv, 0);
            
            // Reduce temporal weight in high-variance areas
            float variance_factor = 1.0 / (1.0 + length(current_variance.rgb) * 10.0);
            temporal_weight *= variance_factor;
            
            // Update variance estimation
            output_variance = lerp(current_variance, history_variance, temporal_weight);
        }
        
        // Temporal blend
        final_color = lerp(current_color, history_color, temporal_weight);
        
        // Clamp to neighborhood to prevent ghosting
        float4 neighborhood_min = current_color;
        float4 neighborhood_max = current_color;
        
        const int clamp_radius = 1;
        for (int y = -clamp_radius; y <= clamp_radius; ++y) {
            for (int x = -clamp_radius; x <= clamp_radius; ++x) {
                if (x == 0 && y == 0) continue;
                
                uint2 neighbor_px = px + int2(x, y);
                if (all(neighbor_px >= 0) && all(neighbor_px < uint2(output_tex_size.xy))) {
                    float4 neighbor_color = current_frame_tex[neighbor_px];
                    neighborhood_min = min(neighborhood_min, neighbor_color);
                    neighborhood_max = max(neighborhood_max, neighbor_color);
                }
            }
        }
        
        // Clamp history to neighborhood bounds
        final_color = clamp(final_color, neighborhood_min, neighborhood_max);
    }
    
    output_tex[px] = final_color;
    variance_output_tex[px] = output_variance;
}
