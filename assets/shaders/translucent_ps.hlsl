#include "inc/math.hlsl"
#include "inc/samplers.hlsl"
#include "inc/frame_constants.hlsl"
#include "inc/mesh.hlsl"
#include "inc/pack_unpack.hlsl"
#include "inc/bindless.hlsl"
#include "inc/brdf.hlsl"
#include "inc/atmosphere.hlsl"
#include "inc/sun.hlsl"

struct PsIn {
    [[vk::location(0)]] float4 color: TEXCOORD0;
    [[vk::location(1)]] float2 uv: TEXCOORD1;
    [[vk::location(2)]] float3 normal: TEXCOORD2;
    [[vk::location(3)]] nointerpolation uint material_id: TEXCOORD3;
    [[vk::location(4)]] float3 tangent: TEXCOORD4;
    [[vk::location(5)]] float3 bitangent: TEXCOORD5;
    [[vk::location(6)]] float3 ws_pos: TEXCOORD6;
    [[vk::location(7)]] float depth: TEXCOORD7;
};

[[vk::push_constant]]
struct {
    uint draw_index;
    uint mesh_index;
} push_constants;

struct InstanceTransform {
    row_major float3x4 current;
    row_major float3x4 previous;
};

[[vk::binding(0)]] StructuredBuffer<InstanceTransform> instance_transforms_dyn;
[[vk::binding(1)]] Texture2D<float4> gbuffer_tex;
[[vk::binding(2)]] Texture2D<float> depth_tex;

struct PsOut {
    float4 color: SV_TARGET0;
};

// Simple forward lighting function for translucent materials
float3 compute_lighting(float3 albedo, float3 normal, float3 view_dir, float roughness, float metalness, float3 ws_pos) {
    // Simple directional light (sun)
    float3 light_dir = frame_constants.sun_direction;
    float3 light_color = frame_constants.sun_color_multiplier * frame_constants.pre_exposure;
    
    // Basic BRDF evaluation
    float ndotl = max(0.0, dot(normal, light_dir));
    float ndotv = max(0.0, dot(normal, view_dir));
    
    // Fresnel term
    float3 f0 = lerp(0.04.xxx, albedo, metalness);
    float3 fresnel = f0 + (1.0 - f0) * pow(1.0 - ndotv, 5.0);
    
    // Diffuse term
    float3 diffuse = albedo * (1.0 - metalness) * (1.0 - fresnel) / M_PI;
    
    // Specular term (simplified)
    float3 half_vec = normalize(light_dir + view_dir);
    float ndoth = max(0.0, dot(normal, half_vec));
    float alpha = roughness * roughness;
    float alpha2 = alpha * alpha;
    float denom = ndoth * ndoth * (alpha2 - 1.0) + 1.0;
    float d = alpha2 / (M_PI * denom * denom);
    float3 specular = fresnel * d * 0.25; // Simplified geometry term
    
    return (diffuse + specular) * light_color * ndotl;
}

PsOut main(PsIn ps) {
    Mesh mesh = meshes[push_constants.mesh_index];
    MeshMaterial material = vertices.Load<MeshMaterial>(mesh.mat_data_offset + ps.material_id * sizeof(MeshMaterial));

    // Skip if material is not translucent
    if (!is_material_translucent(material)) {
        discard;
    }

    const float lod_bias = -0.5;

    // Sample albedo texture
    float2 albedo_uv = transform_material_uv(material, ps.uv, 0);
    Texture2D albedo_tex = bindless_textures[NonUniformResourceIndex(material.maps[MAP_INDEX_ALBEDO])];
    float4 albedo_texel = albedo_tex.SampleBias(sampler_llr, albedo_uv, lod_bias);
    
    float3 albedo = albedo_texel.xyz * float3(material.base_color_mult[0], material.base_color_mult[1], material.base_color_mult[2]) * ps.color.xyz;
    float alpha = get_material_alpha(material) * albedo_texel.a;
    
    // Early discard for very transparent pixels
    if (alpha < 0.01) {
        discard;
    }

    // Sample material properties
    float2 spec_uv = transform_material_uv(material, ps.uv, 2);
    Texture2D spec_tex = bindless_textures[NonUniformResourceIndex(material.maps[MAP_INDEX_SPEC])];
    const float4 metalness_roughness = spec_tex.SampleBias(sampler_llr, spec_uv, lod_bias);
    float perceptual_roughness = material.roughness_mult * metalness_roughness.x;
    float roughness = clamp(perceptual_roughness * perceptual_roughness, 1e-4, 1.0);
    float metalness = metalness_roughness.y * material.metalness_factor;

    // Process normal map
    float3 normal_ws = ps.normal;
    {
        if (!frame_constants.render_overrides.has_flag(RenderOverrideFlags::NO_NORMAL_MAPS)) {
            Texture2D normal_tex = bindless_textures[NonUniformResourceIndex(material.maps[MAP_INDEX_NORMAL])];
            float3 ts_normal = float3(normal_tex.SampleBias(sampler_llr, ps.uv, lod_bias).xy * 2.0 - 1.0, 0);
            ts_normal.z = sqrt(max(0.01, 1.0 - dot(ts_normal.xy, ts_normal.xy)));

            if (frame_constants.render_overrides.has_flag(RenderOverrideFlags::FLIP_NORMAL_MAP_YZ)) {
                ts_normal.zy *= -1;
            }

            if (dot(ps.bitangent, ps.bitangent) > 0.0) {
                float3x3 tbn = float3x3(ps.tangent, ps.bitangent, ps.normal);
                normal_ws = normalize(mul(ts_normal, tbn));
            }
        }
    }

    // Emissive
    float2 emissive_uv = transform_material_uv(material, ps.uv, 3);
    Texture2D emissive_tex = bindless_textures[NonUniformResourceIndex(material.maps[MAP_INDEX_EMISSIVE])];
    float3 emissive = emissive_tex.SampleBias(sampler_llr, emissive_uv, lod_bias).rgb
        * float3(material.emissive[0], material.emissive[1], material.emissive[2])
        * instance_dynamic_parameters_dyn[push_constants.draw_index].emissive_multiplier
        * frame_constants.pre_exposure;

    // View direction
    float3 view_dir = normalize(frame_constants.view_constants.camera_position - ps.ws_pos);
    
    // Compute forward lighting
    float3 lit_color = compute_lighting(albedo, normal_ws, view_dir, roughness, metalness, ps.ws_pos);
    
    // Add emissive
    lit_color += emissive;
    
    // Handle transmission
    if (material.transmission > 0.01) {
        // Simple transmission approximation - sample background color
        float2 screen_uv = ps.position.xy / frame_constants.view_constants.screen_size;
        // Note: In a real implementation, you'd want to sample a background buffer here
        // For now, we'll just blend with a simple environment color
        float3 transmitted_color = float3(0.1, 0.2, 0.4); // Simple sky color
        lit_color = lerp(lit_color, transmitted_color, material.transmission * (1.0 - alpha));
    }

    PsOut ps_out;
    ps_out.color = float4(lit_color, alpha);
    
    return ps_out;
}
