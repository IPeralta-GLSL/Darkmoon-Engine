#include "inc/math.hlsl"
#include "inc/frame_constants.hlsl"
#include "inc/mesh.hlsl"
#include "inc/bindless.hlsl"

struct VsIn {
    [[vk::location(0)]] float3 pos: POSITION;
    [[vk::location(1)]] float2 uv: TEXCOORD0;
    [[vk::location(2)]] float3 normal: NORMAL;
    [[vk::location(3)]] uint material_id: TEXCOORD1;
    [[vk::location(4)]] float3 tangent: TANGENT;
    [[vk::location(5)]] float4 color: COLOR;
};

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

struct InstanceTransform {
    row_major float3x4 current;
    row_major float3x4 previous;
};

[[vk::binding(0)]] StructuredBuffer<InstanceTransform> instance_transforms_dyn;

VsOut main(VsIn input) {
    float3x4 instance_transform = instance_transforms_dyn[push_constants.draw_index].current;
    
    float4 ws_pos = mul(instance_transform, float4(input.pos, 1.0));
    float4 vs_pos = mul(frame_constants.view_constants.view_to_clip, mul(frame_constants.view_constants.world_to_view, ws_pos));
    
    // Transform normal and tangent to world space
    float3 ws_normal = normalize(mul(instance_transform, float4(input.normal, 0.0)).xyz);
    float3 ws_tangent = normalize(mul(instance_transform, float4(input.tangent, 0.0)).xyz);
    float3 ws_bitangent = cross(ws_normal, ws_tangent);

    VsOut output;
    output.position = vs_pos;
    output.color = input.color;
    output.uv = input.uv;
    output.normal = ws_normal;
    output.material_id = input.material_id;
    output.tangent = ws_tangent;
    output.bitangent = ws_bitangent;
    output.ws_pos = ws_pos.xyz;
    output.depth = vs_pos.z / vs_pos.w;
    
    return output;
}
