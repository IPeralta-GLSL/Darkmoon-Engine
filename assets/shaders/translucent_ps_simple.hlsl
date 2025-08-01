#include "inc/frame_constants.hlsl"
#include "inc/mesh.hlsl"
#include "inc/bindless.hlsl"

struct VsOut {
    float4 position: SV_POSITION;
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

float4 ps_main(VsOut input) : SV_Target {
    ConstantBuffer<MeshMaterial> material = ResourceDescriptorHeap[frame_constants.material_data_address + input.material_id];
    
    // Simple transparency check
    if (material.transparency < 0.01 && material.transmission < 0.01) {
        discard;
    }
    
    // Basic material sampling
    float4 base_color = float4(1.0, 1.0, 1.0, 1.0);
    
    if (material.albedo_texture_index != INVALID_BINDLESS_INDEX) {
        Texture2D albedo_tex = ResourceDescriptorHeap[material.albedo_texture_index];
        base_color = albedo_tex.Sample(sampler_llr, input.uv);
    }
    
    base_color *= input.color;
    base_color *= material.base_color_factor;
    
    // Apply transparency
    base_color.a *= material.transparency;
    
    // Simple lighting (just use normal as color for now)
    float3 normal = normalize(input.normal);
    float3 light_color = abs(normal);
    
    return float4(base_color.rgb * light_color, base_color.a);
}
