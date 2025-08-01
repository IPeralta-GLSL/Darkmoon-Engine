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

struct InstanceConstants {
    float4x4 transform;
    float4x4 prev_transform;
};

VsOut vs_main(VsIn input) {
    VsOut output;

    ConstantBuffer<InstanceConstants> instance_constants = ResourceDescriptorHeap[frame_constants.instance_data_address + push_constants.draw_index];
    ConstantBuffer<MeshMaterial> material = ResourceDescriptorHeap[frame_constants.material_data_address + input.material_id];

    float4 ws_pos = mul(instance_constants.transform, float4(input.pos, 1.0));
    output.position = mul(frame_constants.view_proj_matrix, ws_pos);
    
    output.color = input.color;
    output.uv = input.uv;
    output.normal = normalize(mul((float3x3)instance_constants.transform, input.normal));
    output.material_id = input.material_id;
    output.tangent = normalize(mul((float3x3)instance_constants.transform, input.tangent));
    
    // Compute bitangent
    output.bitangent = cross(output.normal, output.tangent);
    
    output.ws_pos = ws_pos.xyz;
    output.depth = output.position.z / output.position.w;

    return output;
}
