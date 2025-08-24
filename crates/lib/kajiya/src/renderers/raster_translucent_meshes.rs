use std::sync::Arc;

use kajiya_backend::{
    ash::vk,
    vk_sync::AccessType,
    vulkan::{buffer::*, image::*, shader::*},
};
use kajiya_rg::{self as rg};
use rg::{IntoRenderPassPipelineBinding, RenderGraph, RenderPassBinding};

use crate::world_renderer::MeshInstance;

use super::{GbufferDepth, raster_meshes::UploadedTriMesh};

pub struct RasterTranslucentMeshesData<'a> {
    pub meshes: &'a [UploadedTriMesh],
    pub instances: &'a [MeshInstance],
    pub vertex_buffer: Arc<Buffer>,
    pub bindless_descriptor_set: vk::DescriptorSet,
}

pub fn raster_translucent_meshes(
    rg: &mut RenderGraph,
    render_pass: Arc<RenderPass>,
    color_output: &mut rg::Handle<Image>,
    gbuffer_depth: &GbufferDepth,
    mesh_data: RasterTranslucentMeshesData<'_>,
) {
    let mut pass = rg.add_pass("raster translucent");

    let pipeline = pass.register_raster_pipeline(
        &[
            PipelineShaderDesc::builder(ShaderPipelineStage::Vertex)
                .hlsl_source("/shaders/translucent_vs.hlsl")
                .build()
                .unwrap(),
            PipelineShaderDesc::builder(ShaderPipelineStage::Pixel)
                .hlsl_source("/shaders/translucent_ps.hlsl")
                .build()
                .unwrap(),
        ],
        RasterPipelineDesc::builder()
            .render_pass(render_pass.clone())
            .face_cull(false)
            .depth_write(false) // Don't write to depth for translucent objects
            .push_constants_bytes(2 * std::mem::size_of::<u32>()),
    );

    let meshes: Vec<UploadedTriMesh> = mesh_data.meshes.to_vec();
    let instances: Vec<MeshInstance> = mesh_data.instances.to_vec();

    // Read-only access to depth buffer for depth testing
    let depth_ref = pass.raster_read(&gbuffer_depth.depth, AccessType::DepthStencilAttachmentRead);
    
    // Write to color output
    let color_ref = pass.raster(color_output, AccessType::ColorAttachmentWrite);

    let vertex_buffer = mesh_data.vertex_buffer.clone();
    let bindless_descriptor_set = mesh_data.bindless_descriptor_set;

    pass.render(move |api| {
        let [width, height, _] = color_ref.desc().extent;

        api.begin_render_pass(
            &*render_pass,
            [width, height],
            &[(color_ref, &ImageViewDesc::default())],
            Some((depth_ref, &ImageViewDesc::builder()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .build()
                .unwrap())),
        )?;

        api.set_default_view_and_scissor([width, height]);

        // Create instance transforms for dynamic constants
        let instance_transforms_offset = api.dynamic_constants().push_from_iter(
            instances.iter().map(|inst| {
                let transform = [
                    inst.transform.x_axis.x,
                    inst.transform.y_axis.x,
                    inst.transform.z_axis.x,
                    inst.transform.translation.x,
                    inst.transform.x_axis.y,
                    inst.transform.y_axis.y,
                    inst.transform.z_axis.y,
                    inst.transform.translation.y,
                    inst.transform.x_axis.z,
                    inst.transform.y_axis.z,
                    inst.transform.z_axis.z,
                    inst.transform.translation.z,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    // Same for prev_transform for now
                    inst.transform.x_axis.x,
                    inst.transform.y_axis.x,
                    inst.transform.z_axis.x,
                    inst.transform.translation.x,
                    inst.transform.x_axis.y,
                    inst.transform.y_axis.y,
                    inst.transform.z_axis.y,
                    inst.transform.translation.y,
                    inst.transform.x_axis.z,
                    inst.transform.y_axis.z,
                    inst.transform.z_axis.z,
                    inst.transform.translation.z,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                ];
                (transform, [0.0f32; 32]) // Dummy prev_transform
            })
        );

        let pipeline = api.bind_raster_pipeline(
            pipeline
                .into_binding()
                .descriptor_set(
                    0,
                    &[RenderPassBinding::DynamicConstantsStorageBuffer(
                        instance_transforms_offset,
                    )],
                )
                .raw_descriptor_set(1, bindless_descriptor_set),
        )?;

        // For now, render all instances as potentially translucent
        // TODO: Filter only truly translucent instances and sort them
        for (instance_idx, instance) in instances.iter().enumerate() {
            let mesh = &meshes[instance.mesh.0];

            let push_constants: [u32; 2] = [instance_idx as u32, instance.mesh.0 as u32];

            pipeline.push_constants(
                api.cb.raw,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                unsafe {
                    std::slice::from_raw_parts(
                        push_constants.as_ptr() as *const u8,
                        push_constants.len() * std::mem::size_of::<u32>(),
                    )
                },
            );

            unsafe {
                api.device().raw.cmd_bind_index_buffer(
                    api.cb.raw,
                    vertex_buffer.raw,
                    mesh.index_buffer_offset as u64,
                    vk::IndexType::UINT32,
                );

                api.device().raw.cmd_draw_indexed(
                    api.cb.raw,
                    mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            }
        }

        api.end_render_pass();

        Ok(())
    });
}
