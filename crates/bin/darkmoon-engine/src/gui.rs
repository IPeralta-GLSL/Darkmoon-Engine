use imgui::im_str;
use crate::asset_browser::AssetBrowser;
use kajiya::RenderOverrideFlags;
use kajiya_simple::*;
use kajiya_backend::shader_progress::GLOBAL_SHADER_PROGRESS;  // Enhanced import

use crate::{
    runtime::{RuntimeState, MAX_FPS_LIMIT},
    PersistedState,
};

use crate::runtime::UiWindowsState;

impl RuntimeState {
    pub fn do_gui(&mut self, persisted: &mut PersistedState, ctx: &mut FrameContext) {
        // --- Asset Browser State ---
        if self.ui_windows.asset_browser.is_none() {
            self.ui_windows.asset_browser = Some(AssetBrowser::new());
        }
        // Update shader progress tracking each frame 
        // Pipeline compilation counts are automatically reported by the pipeline cache
        kajiya_backend::shader_progress::update_pipeline_compilation_frame(0);

        if self.keyboard.was_just_pressed(self.keymap_config.ui.toggle) {
            self.show_gui = !self.show_gui;
        }

        ctx.world_renderer.rg_debug_hook = self.locked_rg_debug_hook.clone();

        // Always show GUI when shaders are compiling, even if normally hidden
        let is_compiling = Self::is_shader_compilation_active() || kajiya_backend::shader_progress::is_compilation_or_heavy_work_active();
        let should_show_gui = self.show_gui || is_compiling;

        if should_show_gui || is_compiling {
            ctx.imgui.take().unwrap().frame(|ui| {
                // --- Asset Browser Window ---
                if let Some(asset_browser) = self.ui_windows.asset_browser.as_mut() {
                    if self.ui_windows.show_asset_browser && asset_browser.open {
                        asset_browser.show(ui);
                    }
                }
                // --- Hierarchy Window ---
                if self.ui_windows.show_hierarchy {
                    imgui::Window::new(im_str!("Hierarchy")).opened(&mut self.ui_windows.show_hierarchy)
                        .size([350.0, 500.0], imgui::Condition::FirstUseEver)
                        .build(ui, || {
                            for (idx, elem) in persisted.scene.elements.iter().enumerate() {
                                let label = if let Some(name) = elem.mesh_nodes.get(0).and_then(|n| n.name.as_ref()) {
                                    im_str!("{}##{}", name, idx)
                                } else {
                                    im_str!("{:?}##{}", elem.source, idx)
                                };
                                if elem.is_compound && !elem.mesh_nodes.is_empty() {
                                    imgui::TreeNode::new(&label).build(ui, || {
                                        for (nidx, node) in elem.mesh_nodes.iter().enumerate() {
                                            let node_label = if let Some(n) = &node.name {
                                                im_str!("{}##{}-{}", n, idx, nidx)
                                            } else {
                                                im_str!("Node {}##{}-{}", nidx, idx, nidx)
                                            };
                                            ui.bullet_text(&node_label);
                                        }
                                    });
                                } else {
                                    ui.bullet_text(&label);
                                }
                            }
                        });
                }
                // --- Shader Compilation Progress Popup (always first, even if GUI is hidden) ---
                if is_compiling {
                    Self::show_shader_compilation_popup(ui);
                }

                // Only show regular GUI if user has it enabled
                if self.show_gui {
                    // --- Menubar superior ---
                if let Some(bar) = ui.begin_main_menu_bar() {
                    if let Some(file_menu) = ui.begin_menu(im_str!("File"), true) {
                        // Opciones de File
                        file_menu.end(&ui);
                    }
                    if let Some(window_menu) = ui.begin_menu(im_str!("Window"), true) {
                        let show_assets = self.ui_windows.asset_browser.as_ref().map_or(false, |a| a.open && self.ui_windows.show_asset_browser);
                        if imgui::MenuItem::new(im_str!("Assets Browser")).selected(show_assets).build(ui) {
                            if let Some(asset_browser) = self.ui_windows.asset_browser.as_mut() {
                                asset_browser.open = !asset_browser.open;
                                self.ui_windows.show_asset_browser = asset_browser.open;
                            }
                        }
                        if imgui::MenuItem::new(im_str!("Hierarchy")).selected(self.ui_windows.show_hierarchy).build(ui) {
                            self.ui_windows.show_hierarchy = !self.ui_windows.show_hierarchy;
                        }
                        if imgui::MenuItem::new(im_str!("Debug")).selected(self.ui_windows.show_debug).build(ui) {
                            self.ui_windows.show_debug = !self.ui_windows.show_debug;
                        }
                        window_menu.end(&ui);
                    }
                    if let Some(view_menu) = ui.begin_menu(im_str!("View"), true) {
                        let mut checked = ctx.world_renderer.is_ray_tracing_enabled();
                        if ui.checkbox(im_str!("Ray tracing"), &mut checked) {
                            ctx.world_renderer.set_ray_tracing_enabled(checked);
                        }
                        view_menu.end(&ui);
                    }
                    bar.end(&ui);
                }

                if imgui::CollapsingHeader::new(im_str!("Tweaks"))
                    .default_open(true)
                    .build(ui)
                {
                    imgui::Drag::<f32>::new(im_str!("EV shift"))
                        .range(-8.0..=12.0)
                        .speed(0.01)
                        .build(ui, &mut persisted.exposure.ev_shift);

                    ui.checkbox(
                        im_str!("Use dynamic exposure"),
                        &mut persisted.exposure.use_dynamic_adaptation,
                    );

                    imgui::Drag::<f32>::new(im_str!("Adaptation speed"))
                        .range(-4.0..=4.0)
                        .speed(0.01)
                        .build(ui, &mut persisted.exposure.dynamic_adaptation_speed);

                    imgui::Drag::<f32>::new(im_str!("Luminance histogram low clip"))
                        .range(0.0..=1.0)
                        .speed(0.001)
                        .build(ui, &mut persisted.exposure.dynamic_adaptation_low_clip);
                    persisted.exposure.dynamic_adaptation_low_clip = persisted
                        .exposure
                        .dynamic_adaptation_low_clip
                        .clamp(0.0, 1.0);

                    imgui::Drag::<f32>::new(im_str!("Luminance histogram high clip"))
                        .range(0.0..=1.0)
                        .speed(0.001)
                        .build(ui, &mut persisted.exposure.dynamic_adaptation_high_clip);
                    persisted.exposure.dynamic_adaptation_high_clip = persisted
                        .exposure
                        .dynamic_adaptation_high_clip
                        .clamp(0.0, 1.0);

                    imgui::Drag::<f32>::new(im_str!("Contrast"))
                        .range(1.0..=1.5)
                        .speed(0.001)
                        .build(ui, &mut persisted.exposure.contrast);

                    imgui::Drag::<f32>::new(im_str!("Emissive multiplier"))
                        .range(0.0..=10.0)
                        .speed(0.1)
                        .build(ui, &mut persisted.light.emissive_multiplier);

                    ui.checkbox(
                        im_str!("Enable emissive"),
                        &mut persisted.light.enable_emissive,
                    );

                    imgui::Drag::<f32>::new(im_str!("Light intensity multiplier"))
                        .range(0.0..=1000.0)
                        .speed(1.0)
                        .build(ui, &mut persisted.light.local_lights.multiplier);

                    imgui::Drag::<f32>::new(im_str!("Camera speed"))
                        .range(0.0..=10.0)
                        .speed(0.025)
                        .build(ui, &mut persisted.movement.camera_speed);

                    imgui::Drag::<f32>::new(im_str!("Camera smoothness"))
                        .range(0.0..=20.0)
                        .speed(0.1)
                        .build(ui, &mut persisted.movement.camera_smoothness);

                    imgui::Drag::<f32>::new(im_str!("Sun rotation smoothness"))
                        .range(0.0..=20.0)
                        .speed(0.1)
                        .build(ui, &mut persisted.movement.sun_rotation_smoothness);

                    imgui::Drag::<f32>::new(im_str!("Field of view"))
                        .range(1.0..=120.0)
                        .speed(0.25)
                        .build(ui, &mut persisted.camera.vertical_fov);

                    imgui::Drag::<f32>::new(im_str!("Sun size"))
                        .range(0.0..=10.0)
                        .speed(0.02)
                        .build(ui, &mut persisted.light.sun.size_multiplier);

                    /*ui.checkbox(
                        im_str!("Show world radiance cache"),
                        &mut ctx.world_renderer.debug_show_wrc,
                    );*/

                    /*if ui.radio_button_bool(
                        im_str!("Move sun"),
                        left_click_edit_mode == LeftClickEditMode::MoveSun,
                    ) {
                        left_click_edit_mode = LeftClickEditMode::MoveSun;
                    }

                    if ui.radio_button_bool(
                        im_str!("Move local lights"),
                        left_click_edit_mode == LeftClickEditMode::MoveLocalLights,
                    ) {
                        left_click_edit_mode = LeftClickEditMode::MoveLocalLights;
                    }

                    imgui::Drag::<u32>::new(im_str!("Light count"))
                        .range(0..=10)
                        .build(ui, &mut state.lights.count);*/

                    ui.checkbox(
                        im_str!("Scroll irradiance cache"),
                        &mut ctx.world_renderer.ircache.enable_scroll,
                    );

                    imgui::Drag::<u32>::new(im_str!("GI spatial reuse passes"))
                        .range(1..=3)
                        .build(ui, &mut ctx.world_renderer.rtdgi.spatial_reuse_pass_count);

                    ctx.world_renderer.rtdgi.spatial_reuse_pass_count = ctx
                        .world_renderer
                        .rtdgi
                        .spatial_reuse_pass_count
                        .clamp(1, 3);

                    ui.checkbox(
                        im_str!("Ray-traced reservoir visibility"),
                        &mut ctx.world_renderer.rtdgi.use_raytraced_reservoir_visibility,
                    );

                    ui.checkbox(
                        im_str!("Allow diffuse ray reuse for reflections"),
                        &mut ctx.world_renderer.rtr.reuse_rtdgi_rays,
                    );

                    #[cfg(feature = "dlss")]
                    {
                        ui.checkbox(im_str!("Use DLSS"), &mut ctx.world_renderer.use_dlss);
                    }
                }

                if imgui::CollapsingHeader::new(im_str!("Scene"))
                    .default_open(true)
                    .build(ui)
                {
                    if let Some(ibl) = persisted.scene.ibl.as_ref() {
                        ui.text(im_str!("IBL: {:?}", ibl));
                        if ui.button(im_str!("Unload"), [0.0, 0.0]) {
                            ctx.world_renderer.ibl.unload_image();
                            persisted.scene.ibl = None;
                        }
                    } else {
                        ui.text(im_str!("Drag a sphere-mapped .hdr/.exr to load as IBL"));
                    }

                    // --- Hierarchy ---
                    if imgui::CollapsingHeader::new(im_str!("Hierarchy"))
                        .default_open(true)
                        .build(ui)
                    {
                        for (idx, elem) in persisted.scene.elements.iter().enumerate() {
                            let label = if let Some(name) = elem.mesh_nodes.get(0).and_then(|n| n.name.as_ref()) {
                                im_str!("{}##{}", name, idx)
                            } else {
                                im_str!("{:?}##{}", elem.source, idx)
                            };
                            if elem.is_compound && !elem.mesh_nodes.is_empty() {
                                imgui::TreeNode::new(&label).build(ui, || {
                                    for (nidx, node) in elem.mesh_nodes.iter().enumerate() {
                                        let node_label = if let Some(n) = &node.name {
                                            im_str!("{}##{}-{}", n, idx, nidx)
                                        } else {
                                            im_str!("Node {}##{}-{}", nidx, idx, nidx)
                                        };
                                        ui.bullet_text(&node_label);
                                    }
                                });
                            } else {
                                ui.bullet_text(&label);
                            }
                        }
                    }

                    let mut element_to_remove = None;
                    for (idx, elem) in persisted.scene.elements.iter_mut().enumerate() {
                        ui.dummy([0.0, 10.0]);

                        let id_token = ui.push_id(idx as i32);
                        ui.text(im_str!("{:?}", elem.source));

                        {
                            ui.set_next_item_width(200.0);

                            let mut scale = elem.transform.scale.x;
                            imgui::Drag::<f32>::new(im_str!("scale"))
                                .range(0.001..=1000.0)
                                .speed(1.0)
                                .flags(imgui::SliderFlags::LOGARITHMIC)
                                .build(ui, &mut scale);
                            if scale != elem.transform.scale.x {
                                elem.transform.scale = Vec3::splat(scale);
                            }
                        }

                        ui.same_line(0.0);
                        if ui.button(im_str!("Delete"), [0.0, 0.0]) {
                            element_to_remove = Some(idx);
                        }

                        // Position
                        {
                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("x"))
                                .speed(0.01)
                                .build(ui, &mut elem.transform.position.x);

                            ui.same_line(0.0);

                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("y"))
                                .speed(0.01)
                                .build(ui, &mut elem.transform.position.y);

                            ui.same_line(0.0);

                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("z"))
                                .speed(0.01)
                                .build(ui, &mut elem.transform.position.z);
                        }

                        // Rotation
                        {
                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("rx"))
                                .speed(0.1)
                                .build(ui, &mut elem.transform.rotation_euler_degrees.x);

                            ui.same_line(0.0);

                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("ry"))
                                .speed(0.1)
                                .build(ui, &mut elem.transform.rotation_euler_degrees.y);

                            ui.same_line(0.0);

                            ui.set_next_item_width(100.0);
                            imgui::Drag::<f32>::new(im_str!("rz"))
                                .speed(0.1)
                                .build(ui, &mut elem.transform.rotation_euler_degrees.z);
                        }

                        id_token.pop(ui);
                    }

                    if let Some(idx) = element_to_remove {
                        let elem = persisted.scene.elements.remove(idx);
                        ctx.world_renderer.remove_instance(elem.instance);
                    }
                }

                // Frustum Culling settings
                if imgui::CollapsingHeader::new(im_str!("Frustum Culling"))
                    .default_open(false)
                    .build(ui)
                {
                    ui.checkbox(
                        im_str!("Enable frustum culling"),
                        &mut persisted.frustum_culling.enabled,
                    );

                    ui.checkbox(
                        im_str!("Debug logging"),
                        &mut persisted.frustum_culling.debug_logging,
                    );

                    ui.checkbox(
                        im_str!("Use sphere culling (faster)"),
                        &mut persisted.frustum_culling.use_sphere_culling,
                    );

                    // Culling method selection
                    ui.text(im_str!("Culling Method:"));
                    let current_method = &mut persisted.frustum_culling.culling_method;
                    
                    let mut is_emissive = matches!(current_method, crate::culling::CullingMethod::EmissiveMultiplier);
                    let mut is_move_away = matches!(current_method, crate::culling::CullingMethod::MoveAway);
                    let mut is_scale_zero = matches!(current_method, crate::culling::CullingMethod::ScaleToZero);
                    
                    if ui.checkbox(im_str!("Emissive Multiplier"), &mut is_emissive) && is_emissive {
                        *current_method = crate::culling::CullingMethod::EmissiveMultiplier;
                    }
                    if ui.checkbox(im_str!("Move Away"), &mut is_move_away) && is_move_away {
                        *current_method = crate::culling::CullingMethod::MoveAway;
                    }
                    if ui.checkbox(im_str!("Scale to Zero"), &mut is_scale_zero) && is_scale_zero {
                        *current_method = crate::culling::CullingMethod::ScaleToZero;
                    }
                    
                    // Show description for the selected method
                    ui.separator();
                    ui.text(im_str!("Method Description:"));
                    match current_method {
                        crate::culling::CullingMethod::EmissiveMultiplier => {
                            ui.text_wrapped(im_str!("Makes objects invisible by setting emissive to 0. Least GPU-efficient."));
                        }
                        crate::culling::CullingMethod::MoveAway => {
                            ui.text_wrapped(im_str!("Moves objects far away. More GPU-efficient as objects are naturally culled by depth."));
                        }
                        crate::culling::CullingMethod::ScaleToZero => {
                            ui.text_wrapped(im_str!("Scales objects to zero size. Very GPU-efficient for triangle culling."));
                        }
                    }

                    imgui::Drag::<f32>::new(im_str!("Default object size"))
                        .range(0.1..=10.0)
                        .speed(0.1)
                        .build(ui, &mut persisted.frustum_culling.default_object_size);

                    imgui::Drag::<u32>::new(im_str!("Log interval (frames)"))
                        .range(30..=600)
                        .speed(10.0)
                        .build(ui, &mut persisted.frustum_culling.log_interval_frames);

                    // Display culling statistics
                    ui.text(im_str!("Culling Stats:"));
                    
                    let total_elements = persisted.scene.elements.len();
                    let total_nodes: usize = persisted.scene.elements.iter()
                        .map(|elem| if elem.is_compound { elem.mesh_nodes.len().max(1) } else { 1 })
                        .sum();
                    let compound_elements = persisted.scene.elements.iter()
                        .filter(|elem| elem.is_compound)
                        .count();
                        
                    ui.text(format!("Scene elements: {}", total_elements));
                    ui.text(format!("Total mesh nodes: {}", total_nodes));
                    ui.text(format!("GLTF compound objects: {}", compound_elements));
                    
                    if persisted.frustum_culling.enabled {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Status: Enabled"));
                        ui.text(format!(
                            "Method: {}",
                            if persisted.frustum_culling.use_sphere_culling {
                                "Sphere"
                            } else {
                                "AABB"
                            }
                        ));
                    } else {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], im_str!("Status: Disabled"));
                    }
                }

                // Occlusion Culling settings
                if imgui::CollapsingHeader::new(im_str!("Occlusion Culling"))
                    .default_open(false)
                    .build(ui)
                {
                    ui.checkbox(
                        im_str!("Enable occlusion culling"),
                        &mut persisted.occlusion_culling.enabled,
                    );

                    ui.checkbox(
                        im_str!("Debug visualization"),
                        &mut persisted.occlusion_culling.debug_visualize,
                    );

                    imgui::Drag::<u32>::new(im_str!("Depth buffer resolution"))
                        .range(64..=512)
                        .speed(1.0)
                        .build(ui, &mut persisted.occlusion_culling.depth_buffer_resolution);

                    imgui::Drag::<f32>::new(im_str!("Depth bias"))
                        .range(0.0..=0.1)
                        .speed(0.001)
                        .build(ui, &mut persisted.occlusion_culling.depth_bias);

                    imgui::Drag::<u32>::new(im_str!("Sample count per object"))
                        .range(1..=8)
                        .speed(1.0)
                        .build(ui, &mut persisted.occlusion_culling.sample_count);

                    imgui::Drag::<f32>::new(im_str!("Max test distance"))
                        .range(10.0..=5000.0)
                        .speed(10.0)
                        .build(ui, &mut persisted.occlusion_culling.max_test_distance);

                    ui.separator();
                    ui.text(im_str!("Occlusion Culling Info:"));
                    ui.text_wrapped(im_str!("Hides objects that are blocked by other objects closer to the camera. Works in combination with frustum culling for maximum efficiency."));
                    
                    if persisted.occlusion_culling.enabled {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Status: Enabled"));
                        ui.text(format!("Depth resolution: {}x{}", 
                            persisted.occlusion_culling.depth_buffer_resolution,
                            persisted.occlusion_culling.depth_buffer_resolution));
                    } else {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], im_str!("Status: Disabled"));
                    }
                }

                // Triangle Culling settings
                if imgui::CollapsingHeader::new(im_str!("Triangle Culling"))
                    .default_open(false)
                    .build(ui)
                {
                    ui.checkbox(
                        im_str!("Enable triangle culling"),
                        &mut persisted.triangle_culling.enabled,
                    );
                    ui.same_line(0.0);
                    ui.text_colored([0.7, 0.7, 0.7, 1.0], im_str!("Per-triangle visibility tests"));

                    if persisted.triangle_culling.enabled {
                        ui.separator();
                        
                        ui.checkbox(
                            im_str!("Debug logging"),
                            &mut persisted.triangle_culling.debug_logging,
                        );
                        
                        imgui::Drag::<u32>::new(im_str!("Log interval (frames)"))
                            .range(1..=300)
                            .speed(1.0)
                            .build(ui, &mut persisted.triangle_culling.log_interval_frames);

                        ui.separator();
                        ui.text(im_str!("Culling Methods:"));

                        // Back-face culling checkbox
                        let mut has_backface = persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::BackFace);
                        if ui.checkbox(im_str!("Back-face culling"), &mut has_backface) {
                            if has_backface {
                                if !persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::BackFace) {
                                    persisted.triangle_culling.methods.push(crate::math::PrimitiveCullingMethod::BackFace);
                                }
                            } else {
                                persisted.triangle_culling.methods.retain(|m| m != &crate::math::PrimitiveCullingMethod::BackFace);
                            }
                        }
                        ui.same_line(0.0);
                        ui.text_colored([0.7, 0.7, 0.7, 1.0], im_str!("Hide faces pointing away"));

                        // Small triangle culling checkbox
                        let mut has_small = persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::SmallTriangle);
                        if ui.checkbox(im_str!("Small triangle culling"), &mut has_small) {
                            if has_small {
                                if !persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::SmallTriangle) {
                                    persisted.triangle_culling.methods.push(crate::math::PrimitiveCullingMethod::SmallTriangle);
                                }
                            } else {
                                persisted.triangle_culling.methods.retain(|m| m != &crate::math::PrimitiveCullingMethod::SmallTriangle);
                            }
                        }
                        ui.same_line(0.0);
                        ui.text_colored([0.7, 0.7, 0.7, 1.0], im_str!("Hide very small triangles"));

                        ui.separator();
                        ui.text(im_str!("Parameters:"));

                        imgui::Drag::<f32>::new(im_str!("Min triangle area (pixels)"))
                            .range(0.1..=100.0)
                            .speed(0.1)
                            .build(ui, &mut persisted.triangle_culling.min_triangle_area);

                        imgui::Drag::<f32>::new(im_str!("Back-face epsilon"))
                            .range(0.0..=0.1)
                            .speed(0.001)
                            .build(ui, &mut persisted.triangle_culling.backface_epsilon);

                        imgui::Drag::<f32>::new(im_str!("Max distance"))
                            .range(10.0..=5000.0)
                            .speed(10.0)
                            .build(ui, &mut persisted.triangle_culling.max_distance);
                    }

                    ui.separator();
                    ui.text(im_str!("Triangle Culling Info:"));
                    ui.text_wrapped(im_str!("Culls individual triangles based on various criteria. Works at the finest level of detail, complementing object-level frustum and occlusion culling."));
                    
                    if persisted.triangle_culling.enabled {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], im_str!("Status: Enabled"));
                        ui.text(format!("Active methods: {}", persisted.triangle_culling.methods.len()));
                        
                        // Show triangle culling statistics
                        let triangle_stats = self.get_triangle_culling_statistics();
                        if triangle_stats.triangles_tested > 0 {
                            ui.separator();
                            ui.text(im_str!("Triangle Statistics:"));
                            ui.text(format!("Triangles tested: {}", triangle_stats.triangles_tested));
                            ui.text(format!("Triangles rendered: {}", triangle_stats.triangles_rendered));
                            ui.text(format!("Culling efficiency: {:.1}%", triangle_stats.culling_efficiency()));
                            
                            if triangle_stats.total_culled > 0 {
                                ui.text(format!("  Backface: {}", triangle_stats.backface_culled));
                                ui.text(format!("  Degenerate: {}", triangle_stats.degenerate_culled));
                                ui.text(format!("  Small: {}", triangle_stats.small_triangle_culled));
                                ui.text(format!("  View-dependent: {}", triangle_stats.view_dependent_culled));
                            }
                        }
                    } else {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], im_str!("Status: Disabled"));
                    }
                }

                // Resource Streaming Section
                if imgui::CollapsingHeader::new(im_str!("Resource Streaming"))
                    .default_open(false)
                    .build(ui)
                {
                    self.streaming_integration.render_gui(ui);
                }

                if imgui::CollapsingHeader::new(im_str!("Overrides"))
                    .default_open(false)
                    .build(ui)
                {
                    macro_rules! do_flag {
                        ($flag:path, $name:literal) => {
                            let mut is_set: bool =
                                ctx.world_renderer.render_overrides.has_flag($flag);
                            ui.checkbox(im_str!($name), &mut is_set);
                            ctx.world_renderer.render_overrides.set_flag($flag, is_set);
                        };
                    }

                    do_flag!(
                        RenderOverrideFlags::FORCE_FACE_NORMALS,
                        "Force face normals"
                    );
                    do_flag!(RenderOverrideFlags::NO_NORMAL_MAPS, "No normal maps");
                    do_flag!(
                        RenderOverrideFlags::FLIP_NORMAL_MAP_YZ,
                        "Flip normal map YZ"
                    );
                    do_flag!(RenderOverrideFlags::NO_METAL, "No metal");

                    imgui::Drag::<f32>::new(im_str!("Roughness scale"))
                        .range(0.0..=4.0)
                        .speed(0.001)
                        .build(
                            ui,
                            &mut ctx.world_renderer.render_overrides.material_roughness_scale,
                        );
                }

                if imgui::CollapsingHeader::new(im_str!("Sequence"))
                    .default_open(false)
                    .build(ui)
                {
                    if ui.button(im_str!("Add key"), [0.0, 0.0]) {
                        self.add_sequence_keyframe(persisted);
                    }

                    ui.same_line(0.0);
                    if self.is_sequence_playing() {
                        if ui.button(im_str!("Stop"), [0.0, 0.0]) {
                            self.stop_sequence();
                        }
                    } else if ui.button(im_str!("Play"), [0.0, 0.0]) {
                        self.play_sequence(persisted);
                    }

                    ui.same_line(0.0);
                    ui.set_next_item_width(60.0);
                    imgui::Drag::<f32>::new(im_str!("Speed"))
                        .range(0.0..=4.0)
                        .speed(0.01)
                        .build(ui, &mut self.sequence_playback_speed);

                    if self.active_camera_key.is_some() {
                        ui.same_line(0.0);
                        if ui.button(im_str!("Deselect key"), [0.0, 0.0]) {
                            self.active_camera_key = None;
                        }
                    }

                    enum Cmd {
                        JumpToKey(usize),
                        DeleteKey(usize),
                        ReplaceKey(usize),
                        None,
                    }
                    let mut cmd = Cmd::None;

                    persisted.sequence.each_key(|i, item| {
                        let active = Some(i) == self.active_camera_key;

                        let label = if active {
                            im_str!("-> {}:", i)
                        } else {
                            im_str!("{}:", i)
                        };

                        if ui.button(&label, [0.0, 0.0]) {
                            cmd = Cmd::JumpToKey(i);
                        }

                        ui.same_line(0.0);
                        ui.set_next_item_width(60.0);
                        imgui::InputFloat::new(ui, &im_str!("duration##{}", i), &mut item.duration)
                            .build();

                        ui.same_line(0.0);
                        ui.checkbox(
                            &im_str!("Pos##{}", i),
                            &mut item.value.camera_position.is_some,
                        );

                        ui.same_line(0.0);
                        ui.checkbox(
                            &im_str!("Dir##{}", i),
                            &mut item.value.camera_direction.is_some,
                        );

                        ui.same_line(0.0);
                        ui.checkbox(&im_str!("Sun##{}", i), &mut item.value.towards_sun.is_some);

                        ui.same_line(0.0);
                        if ui.button(&im_str!("Delete##{}", i), [0.0, 0.0]) {
                            cmd = Cmd::DeleteKey(i);
                        }

                        ui.same_line(0.0);
                        if ui.button(&im_str!("Replace##{}:", i), [0.0, 0.0]) {
                            cmd = Cmd::ReplaceKey(i);
                        }
                    });

                    match cmd {
                        Cmd::JumpToKey(i) => self.jump_to_sequence_key(persisted, i),
                        Cmd::DeleteKey(i) => self.delete_camera_sequence_key(persisted, i),
                        Cmd::ReplaceKey(i) => self.replace_camera_sequence_key(persisted, i),
                        Cmd::None => {}
                    }
                }

                if self.ui_windows.show_debug {
                    if imgui::CollapsingHeader::new(im_str!("Debug"))
                        .default_open(false)
                        .build(ui)
                    {
                        if ui.radio_button_bool(
                            im_str!("Scene geometry"),
                            ctx.world_renderer.debug_mode == RenderDebugMode::None,
                        ) {
                            ctx.world_renderer.debug_mode = RenderDebugMode::None;
                        }

                        /*if ui.radio_button_bool(
                            im_str!("World radiance cache"),
                            ctx.world_renderer.debug_mode == RenderDebugMode::WorldRadianceCache,
                        ) {
                            ctx.world_renderer.debug_mode = RenderDebugMode::WorldRadianceCache;
                        }*/

                        imgui::ComboBox::new(im_str!("Shading")).build_simple_string(
                            ui,
                            &mut ctx.world_renderer.debug_shading_mode,
                            &[
                                im_str!("Default"),
                                im_str!("No base color"),
                                im_str!("Diffuse GI"),
                                im_str!("Reflections"),
                                im_str!("RTX OFF"),
                                im_str!("Irradiance cache"),
                            ],
                        );

                        imgui::Drag::<u32>::new(im_str!("Max FPS"))
                            .range(1..=MAX_FPS_LIMIT)
                            .build(ui, &mut self.max_fps);

                        ui.checkbox(im_str!("Allow pass overlap"), unsafe {
                            &mut kajiya::rg::RG_ALLOW_PASS_OVERLAP
                        });
                    }
                }

                if imgui::CollapsingHeader::new(im_str!("GPU passes"))
                    .default_open(true)
                    .build(ui)
                {
                    ui.text(format!("CPU frame time: {:.3}ms", ctx.dt_filtered * 1000.0));

                    if let Some(report) = gpu_profiler::profiler().last_report() {
                        let ordered_scopes = report.scopes.as_slice();
                        let gpu_time_ms: f64 =
                            ordered_scopes.iter().map(|scope| scope.duration.ms()).sum();

                        ui.text(format!("GPU frame time: {:.3}ms", gpu_time_ms));

                        for (scope_index, scope) in ordered_scopes.iter().enumerate() {
                            if scope.name == "debug" || scope.name.starts_with('_') {
                                continue;
                            }

                            let render_debug_hook = kajiya::rg::RenderDebugHook {
                                name: scope.name.clone(),
                                id: scope_index as u64,
                            };

                            let style = self.locked_rg_debug_hook.as_ref().and_then(|hook| {
                                if hook.render_debug_hook == render_debug_hook {
                                    Some(ui.push_style_color(
                                        imgui::StyleColor::Text,
                                        [1.0, 1.0, 0.1, 1.0],
                                    ))
                                } else {
                                    None
                                }
                            });

                            ui.text(format!("{}: {:.3}ms", scope.name, scope.duration.ms()));

                            if let Some(style) = style {
                                style.pop(ui);
                            }

                            if ui.is_item_hovered() {
                                ctx.world_renderer.rg_debug_hook =
                                    Some(kajiya::rg::GraphDebugHook { render_debug_hook });

                                if ui.is_item_clicked(imgui::MouseButton::Left) {
                                    if self.locked_rg_debug_hook == ctx.world_renderer.rg_debug_hook
                                    {
                                        self.locked_rg_debug_hook = None;
                                    } else {
                                        self.locked_rg_debug_hook =
                                            ctx.world_renderer.rg_debug_hook.clone();
                                    }
                                }
                            }
                        }
                    }
                }
                } // Close the if self.show_gui block
            });
        }
    }

    /// Check if shader compilation is currently active
    fn is_shader_compilation_active() -> bool {
        if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
            if let Ok(progress) = tracker.get_progress().lock() {
                // Show if there are registered shaders and they're not complete
                // OR if pipeline compilation is explicitly active
                let has_active_compilation = (progress.total_shaders > 0 && !progress.is_complete) 
                    || tracker.is_pipeline_compilation_active();
                    
                if has_active_compilation {
                    log::debug!("Shader compilation active: total={}, completed={}, is_complete={}, pipeline_active={}", 
                        progress.total_shaders, progress.completed_shaders, progress.is_complete, tracker.is_pipeline_compilation_active());
                }
                    
                return has_active_compilation;
            }
        }
        false
    }

    /// For testing - simulate shader compilation on startup (only if no real compilation is happening)
    pub fn simulate_shader_compilation() {
        // Enable simulation in debug builds to help with testing
        const ENABLE_SIMULATION: bool = true; // Always enabled for now

        if !ENABLE_SIMULATION {
            return;
        }

        std::thread::spawn(move || {
            // Wait a bit to ensure the GUI loop is ready
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
            // Check if real compilation is already happening
            if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
                if let Ok(progress) = tracker.get_progress().lock() {
                    if progress.total_shaders > 0 && !progress.is_simulation_mode {
                        log::info!("Real shader compilation already in progress, skipping simulation");
                        return;
                    }
                }
            }
            
            log::info!("Starting shader compilation simulation");
            
            if let Ok(mut tracker) = GLOBAL_SHADER_PROGRESS.lock() {
                tracker.set_simulation_mode(true);
                
                // Simulate some typical shaders being compiled (more realistic number)
                let test_shaders = vec![
                    "/shaders/rt/gbuffer.rchit.hlsl",
                    "/shaders/rt/reference_path_trace.rgen.hlsl", 
                    "/shaders/light_gbuffer.hlsl",
                    "/shaders/sky/comp_cube.hlsl",
                    "/shaders/dof/coc.hlsl",
                    "/shaders/taa/reproject_history.hlsl",
                    "/shaders/tonemap/luminance_histogram.hlsl",
                    "/shaders/post/post_combine.hlsl",
                    "/shaders/rt/shadow.rchit.hlsl",
                    "/shaders/atmosphere/comp_transmittance.hlsl",
                    "rust::gbuffer_cs",
                    "rust::ssgi_cs",
                    "rust::reflection_cs",
                    "rust::temporal_upsampling_cs",
                    "rust::bloom_downsample_cs",
                ];

                for shader in &test_shaders {
                    tracker.register_shader(shader);
                }
            }
            
            // Wait a bit more to show the initial state
            std::thread::sleep(std::time::Duration::from_millis(1500));
            
            // Simulate compilation progress with more realistic timing
            let test_shaders = vec![
                "/shaders/rt/gbuffer.rchit.hlsl",
                "/shaders/rt/reference_path_trace.rgen.hlsl", 
                "/shaders/light_gbuffer.hlsl",
                "/shaders/sky/comp_cube.hlsl",
                "/shaders/dof/coc.hlsl",
                "/shaders/taa/reproject_history.hlsl",
                "/shaders/tonemap/luminance_histogram.hlsl",
                "/shaders/post/post_combine.hlsl",
                "/shaders/rt/shadow.rchit.hlsl",
                "/shaders/atmosphere/comp_transmittance.hlsl",
                "rust::gbuffer_cs",
                "rust::ssgi_cs",
                "rust::reflection_cs",
                "rust::temporal_upsampling_cs",
                "rust::bloom_downsample_cs",
            ];

            for (i, shader) in test_shaders.iter().enumerate() {
                if let Ok(mut tracker) = GLOBAL_SHADER_PROGRESS.lock() {
                    tracker.start_compiling_shader(shader);
                }
                
                // Simulate more realistic compilation time (1-3 seconds per shader)
                let compilation_time = 1000 + (i * 200) as u64 + ((i * 123) % 1000) as u64;
                std::thread::sleep(std::time::Duration::from_millis(compilation_time));
                
                if let Ok(mut tracker) = GLOBAL_SHADER_PROGRESS.lock() {
                    tracker.finish_compiling_shader(shader, true);
                }
            }
            
            // When simulation finishes, keep it alive until real compilation starts or we're sure none is needed
            log::info!("Shader compilation simulation complete. Keeping window active until real compilation starts...");
            
            // Keep the simulation "complete" state visible but stay active for longer
            let mut monitoring_iterations = 0;
            let max_monitoring_time = 30; // 30 * 500ms = 15 seconds
            
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                monitoring_iterations += 1;
                
                let should_exit = if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
                    if let Ok(progress) = tracker.get_progress().lock() {
                        // If real compilation has taken over, stop monitoring
                        if !progress.is_simulation_mode {
                            log::info!("Real shader compilation detected, ending simulation monitoring");
                            true
                        } else if monitoring_iterations >= max_monitoring_time {
                            log::info!("Simulation monitoring timeout, assuming no real compilation needed");
                            // Mark as truly complete after timeout
                            drop(progress);
                            if let Ok(mut tracker_mut) = GLOBAL_SHADER_PROGRESS.lock() {
                                tracker_mut.set_pipeline_compilation_active(false);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };
                
                if should_exit {
                    break;
                }
            }
        });
    }

    /// Show shader compilation progress popup
    fn show_shader_compilation_popup(ui: &imgui::Ui) {
        if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
            if let Ok(progress) = tracker.get_progress().lock() {
                // Show popup if:
                // 1. There are shaders registered AND compilation is not complete
                // 2. OR pipeline compilation is explicitly active (even if no shaders registered yet)
                let should_show = (progress.total_shaders > 0 && !progress.is_complete) 
                    || (progress.total_shaders == 0 && tracker.is_pipeline_compilation_active());
                
                if should_show {
                    // Create a centered window
                    let [display_width, display_height] = ui.io().display_size;
                    let window_width = 500.0;
                    let window_height = 200.0;
                    
                    // Use window builder pattern for imgui 0.7
                    imgui::Window::new(im_str!("Compiling Shaders"))
                        .position(
                            [
                                (display_width - window_width) * 0.5,
                                (display_height - window_height) * 0.5,
                            ],
                            imgui::Condition::Always,
                        )
                        .size([window_width, window_height], imgui::Condition::Always)
                        .resizable(false)
                        .movable(false)
                        .collapsible(false)
                        .build(ui, || {
                            ui.text("Initializing rendering engine...");
                            ui.spacing();

                            // Progress bar
                            let progress_fraction = if progress.total_shaders > 0 {
                                progress.progress_percentage() / 100.0
                            } else {
                                0.0 // Indeterminate progress when no shaders registered yet
                            };
                            
                            imgui::ProgressBar::new(progress_fraction)
                                .size([450.0, 20.0])
                                .overlay_text(&im_str!("{:.1}%", progress.progress_percentage()))
                                .build(ui);

                            ui.spacing();

                            // Status text
                            let status = if progress.total_shaders > 0 {
                                progress.status_text()
                            } else if tracker.is_pipeline_compilation_active() {
                                "Preparing shader compilation...".to_string()
                            } else {
                                "Waiting for shader compilation to start...".to_string()
                            };
                            ui.text(status);

                            // Additional info about compilation type
                            if progress.is_simulation_mode {
                                ui.spacing();
                                ui.text_colored([0.8, 0.8, 0.3, 1.0], "Note: This is a simulation. Real compilation may follow.");
                            } else if !progress.is_simulation_mode && progress.total_shaders > 0 {
                                ui.spacing();
                                ui.text_colored([0.3, 0.8, 0.3, 1.0], "Real shader compilation in progress...");
                            }

                            if !progress.failed_shaders.is_empty() {
                                ui.spacing();
                                ui.text_colored([1.0, 0.3, 0.3, 1.0], "Some shaders failed to compile:");
                                for failed in &progress.failed_shaders {
                                    ui.text_colored([1.0, 0.6, 0.6, 1.0], failed);
                                }
                            }
                        });
                }
            }
        }
    }
}
