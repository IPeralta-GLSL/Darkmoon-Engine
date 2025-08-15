use crate::asset_browser::{AssetBrowser, AssetAction};
use kajiya::RenderOverrideFlags;
use kajiya_simple::*;
use kajiya_backend::shader_progress::GLOBAL_SHADER_PROGRESS;  // Enhanced import
use darkmoon_icons::*;
use imgui::*;

use crate::{
    runtime::{RuntimeState, MAX_FPS_LIMIT},
    PersistedState,
};

impl RuntimeState {
    fn get_element_icon(elem: &crate::persisted::SceneElement) -> char {
        if elem.is_compound {
            ICON_OBJECT_GROUP
        } else {
            match &elem.source {
                crate::persisted::MeshSource::File(path) => {
                    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                        match extension.to_lowercase().as_str() {
                            "dmoon" => ICON_FILM,           
                            "gltf" | "glb" => ICON_CUBE,    
                            _ => ICON_CUBE,                
                        }
                    } else {
                        ICON_CUBE
                    }
                }
                crate::persisted::MeshSource::Cache(_) => ICON_GEAR, 
            }
        }
    }

    /// mesh node
    fn get_node_icon() -> char {
        ICON_SHAPES 
    }

    /// sun
    fn get_sun_icon() -> char {
        ICON_SUN 
    }

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
            log::info!("GUI toggle pressed. show_gui is now: {}", self.show_gui);
        }

        ctx.world_renderer.rg_debug_hook = self.locked_rg_debug_hook.clone();

        // Always show GUI when shaders are compiling, even if normally hidden
        let is_compiling = Self::is_shader_compilation_active() || kajiya_backend::shader_progress::is_compilation_or_heavy_work_active();
        let should_show_gui = self.show_gui || is_compiling;
        
        // Debug logging for GUI state
        static mut LAST_GUI_STATE: Option<(bool, bool, bool)> = None;
        let current_state = (self.show_gui, is_compiling, should_show_gui);
        unsafe {
            if LAST_GUI_STATE != Some(current_state) {
                log::info!("GUI state changed: show_gui={}, is_compiling={}, should_show_gui={}", 
                    self.show_gui, is_compiling, should_show_gui);
                LAST_GUI_STATE = Some(current_state);
            }
        }

        if should_show_gui || is_compiling {
            log::debug!("Starting ImGui frame with show_gui={}, is_compiling={}", self.show_gui, is_compiling);
            
            // Variable to track save requests outside the UI closure
            let mut save_scene_requested = false;
            
            if let Some(imgui_ctx) = ctx.imgui.take() {
                log::info!("ImGui context taken successfully, calling frame()");
                imgui_ctx.frame(|ui| {
                    log::debug!("Inside ImGui frame callback");
                    // --- Asset Browser Window ---
                if let Some(asset_browser) = self.ui_windows.asset_browser.as_mut() {
                    if self.ui_windows.show_asset_browser && asset_browser.open {
                        let action = asset_browser.show(ui);
                        // Handle asset browser actions
                        match action {
                            AssetAction::LoadScene(scene_path) => {
                                // Convert PathBuf to string for the load_scene_from_path method
                                if let Some(path_str) = scene_path.to_str() {
                                    if let Err(err) = self.load_scene_from_path(persisted, ctx, path_str) {
                                        log::error!("Failed to load scene from asset browser {}: {:#}", path_str, err);
                                    } else {
                                        log::info!("Successfully loaded scene from asset browser: {}", path_str);
                                    }
                                } else {
                                    log::error!("Failed to convert scene path to string: {:?}", scene_path);
                                }
                            }
                            AssetAction::None => {
                                // No action taken
                            }
                        }
                    }
                }
                // --- Hierarchy Window ---
                // Outliner window (was Hierarchy)
                static mut SELECTED_ELEMENT: Option<usize> = None;
                static mut RESET_WINDOW_POSITIONS: bool = false;
                static mut UNSAVED_CHANGES: bool = false;
                
                if self.ui_windows.show_hierarchy {
                    let reset_condition = unsafe {
                        if RESET_WINDOW_POSITIONS {
                            imgui::Condition::Always
                        } else {
                            imgui::Condition::FirstUseEver
                        }
                    };
                    
                    ui.window("Outliner")
                        .opened(&mut self.ui_windows.show_hierarchy)
                        .size([350.0, 500.0], reset_condition)
                        .position([10.0, 30.0], reset_condition)  // Posición segura con margen
                        .build(|| {
                            // Sun as a selectable item
                            let sun_selected = unsafe { SELECTED_ELEMENT == Some(usize::MAX) };
                            let sun_label = create_icon_label(Self::get_sun_icon(), "Sun Direction");
                            if ui.selectable_config(&format!("{}", sun_label))
                                .selected(sun_selected)
                                .build() {
                                unsafe { SELECTED_ELEMENT = Some(usize::MAX); }
                            }
                            for (idx, elem) in persisted.scene.elements.iter().enumerate() {
                                let element_icon = Self::get_element_icon(elem);
                                let element_name = if let Some(name) = elem.mesh_nodes.get(0).and_then(|n| n.name.as_ref()) {
                                    name.clone()
                                } else {
                                    format!("{:?}", elem.source)
                                };
                                let element_label = create_icon_label(element_icon, &element_name);
                                
                                let is_selected = unsafe { SELECTED_ELEMENT == Some(idx) };
                                if ui.selectable_config(&format!("{}##{}", element_label, idx))
                                    .selected(is_selected)
                                    .build() {
                                    unsafe { SELECTED_ELEMENT = Some(idx); }
                                }
                                if elem.is_compound && !elem.mesh_nodes.is_empty() {
                                    ui.tree_node_config(&format!("Nodes##{}", idx))
                                        .build(|| {
                                        for (nidx, node) in elem.mesh_nodes.iter().enumerate() {
                                            let node_icon = Self::get_node_icon();
                                            let node_name = if let Some(n) = &node.name {
                                                n.clone()
                                            } else {
                                                format!("Node {}", nidx)
                                            };
                                            let node_label = create_icon_label(node_icon, &node_name);
                                            ui.bullet_text(&format!("{}##{}-{}", node_label, idx, nidx));
                                        }
                                    });
                                }
                            }
                        });
                }

                // Attributes window for selected object
                let selected_idx = unsafe { SELECTED_ELEMENT };
                
                if let Some(idx) = selected_idx {
                    let reset_condition = unsafe {
                        if RESET_WINDOW_POSITIONS {
                            imgui::Condition::Always
                        } else {
                            imgui::Condition::FirstUseEver
                        }
                    };
                    
                    if idx == usize::MAX {
                        // Sun attributes
                        ui.window("Attributes")
                            .size([350.0, 200.0], reset_condition)
                            .position([370.0, 30.0], reset_condition)  // A la derecha del Outliner
                            .build(|| {
                                let controller = &mut persisted.light.sun.controller;
                                let mut dir = controller.towards_sun();
                                ui.text("Sun Direction (editable):");
                                let mut changed = false;
                                changed |= Drag::new("X").speed(0.01).range(-1.0, 1.0).build(ui, &mut dir.x);
                                changed |= Drag::new("Y").speed(0.01).range(-1.0, 1.0).build(ui, &mut dir.y);
                                changed |= Drag::new("Z").speed(0.01).range(-1.0, 1.0).build(ui, &mut dir.z);
                                if changed {
                                    if dir.length() > 1e-4 {
                                        controller.set_towards_sun(dir.normalize());
                                    }
                                }
                                ui.separator();
                                ui.text(&format!("Current: ({:.3}, {:.3}, {:.3})", dir.x, dir.y, dir.z));
                            });
                    } else if let Some(elem) = persisted.scene.elements.get_mut(idx) {
                        ui.window("Attributes")
                            .size([350.0, 400.0], reset_condition)
                            .position([370.0, 30.0], reset_condition)  // A la derecha del Outliner
                            .build(|| {
                                ui.text(&format!("Source: {:?}", elem.source));
                                ui.text(&format!("Compound: {}", elem.is_compound));
                                ui.separator();
                                
                                // Transform controls with grouping
                                ui.text("Position:");
                                ui.indent();
                                let mut pos_changed = false;
                                pos_changed |= Drag::new("X##pos").speed(0.1).range(-1000.0, 1000.0).build(ui, &mut elem.transform.position.x);
                                pos_changed |= Drag::new("Y##pos").speed(0.1).range(-1000.0, 1000.0).build(ui, &mut elem.transform.position.y);
                                pos_changed |= Drag::new("Z##pos").speed(0.1).range(-1000.0, 1000.0).build(ui, &mut elem.transform.position.z);
                                ui.unindent();
                                
                                ui.text("Rotation (degrees):");
                                ui.indent();
                                let mut rot_changed = false;
                                rot_changed |= Drag::new("X##rot").speed(1.0).range(-360.0, 360.0).build(ui, &mut elem.transform.rotation_euler_degrees.x);
                                rot_changed |= Drag::new("Y##rot").speed(1.0).range(-360.0, 360.0).build(ui, &mut elem.transform.rotation_euler_degrees.y);
                                rot_changed |= Drag::new("Z##rot").speed(1.0).range(-360.0, 360.0).build(ui, &mut elem.transform.rotation_euler_degrees.z);
                                ui.unindent();
                                
                                ui.text("Scale:");
                                ui.indent();
                                let mut scale_changed = false;
                                scale_changed |= Drag::new("X##scale").speed(0.01).range(0.001, 100.0).build(ui, &mut elem.transform.scale.x);
                                scale_changed |= Drag::new("Y##scale").speed(0.01).range(0.001, 100.0).build(ui, &mut elem.transform.scale.y);
                                scale_changed |= Drag::new("Z##scale").speed(0.01).range(0.001, 100.0).build(ui, &mut elem.transform.scale.z);
                                ui.unindent();
                                
                                let any_changed = pos_changed || rot_changed || scale_changed;
                                
                                // Apply changes to renderer immediately for real-time feedback
                                if any_changed {
                                    ctx.world_renderer.set_instance_transform(elem.instance, elem.transform.affine_transform());
                                    // Mark scene as having unsaved changes
                                    unsafe { UNSAVED_CHANGES = true; }
                                }
                                
                                ui.separator();
                                
                                // Reset transform button
                                if ui.button("Reset Transform") {
                                    elem.transform = crate::persisted::SceneElementTransform::IDENTITY;
                                    ctx.world_renderer.set_instance_transform(elem.instance, elem.transform.affine_transform());
                                    unsafe { UNSAVED_CHANGES = true; }
                                }
                                
                                ui.separator();
                                
                                // Show save status and quick save button
                                let has_unsaved = unsafe { UNSAVED_CHANGES };
                                if let Some(scene_path) = &self.current_scene_path {
                                    let scene_name = scene_path.file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or("Unknown");
                                    
                                    // Quick save button - only show if there are unsaved changes
                                    if has_unsaved {
                                        if ui.button(&format!("{} Quick Save", ICON_FLOPPY_DISK)) {
                                            save_scene_requested = true;
                                        }
                                        ui.same_line();
                                        ui.text_colored([1.0, 0.8, 0.0, 1.0], &format!("* {} has unsaved changes", scene_name));
                                    } else {
                                        ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("{} {} - All changes saved", ICON_CHECK, scene_name));
                                    }
                                    
                                    ui.text_colored([0.7, 0.7, 0.7, 1.0], "Tip: Use 'S' key or File > Save Scene for quick save");
                                } else {
                                    ui.text_colored([0.7, 0.7, 0.7, 1.0], "No scene file loaded - drag & drop a .dmoon file");
                                }
                                
                                // Show mesh node information if available
                                if !elem.mesh_nodes.is_empty() {
                                    ui.separator();
                                    ui.text(&format!("{} Mesh Nodes ({}):", ICON_SHAPES, elem.mesh_nodes.len()));
                                    ui.indent();
                                    for (nidx, node) in elem.mesh_nodes.iter().enumerate() {
                                        if let Some(name) = &node.name {
                                            ui.bullet_text(&format!("{} {}", Self::get_node_icon(), name));
                                        } else {
                                            ui.bullet_text(&format!("{} Node {}", Self::get_node_icon(), nidx));
                                        }
                                    }
                                    ui.unindent();
                                }
                            });
                    }
                }
                // --- Shader Compilation Progress Popup (always first, even if GUI is hidden) ---
                if is_compiling {
                    Self::show_shader_compilation_popup(ui);
                }

                // Only show regular GUI if user has it enabled
                if self.show_gui {
                    log::debug!("Showing regular GUI (show_gui=true)");
                            
                            // --- Menubar superior ---
                if let Some(bar) = ui.begin_main_menu_bar() {
                    if let Some(file_menu) = ui.begin_menu("File") {
                        if let Some(scene_menu) = ui.begin_menu("Load Scene") {
                            let scene_files = [
                                ("Car", "assets/scenes/car.dmoon"),
                                ("Car2", "assets/scenes/car2.dmoon"),
                                ("Conference", "assets/scenes/conference.dmoon"),
                                ("Pica", "assets/scenes/pica.dmoon"),
                                ("Viziers", "assets/scenes/viziers.dmoon"),
                                ("Gas Stations", "assets/scenes/gas_stations.dmoon"),
                                ("Battle", "assets/scenes/battle.dmoon"),
                                ("Girl", "assets/scenes/girl.dmoon"),
                                ("Tree", "assets/scenes/tree.dmoon"),
                                ("Mini Battle", "assets/scenes/mini_battle.dmoon"),
                            ];
                            
                            for (name, path) in &scene_files {
                                if ui.menu_item(name) {
                                    if let Err(err) = self.load_scene_from_path(persisted, ctx, path) {
                                        log::error!("Failed to load scene {}: {:#}", name, err);
                                    }
                                }
                            }
                            
                            ui.separator();
                            
                            if ui.menu_item_config("Custom File...").enabled(false).build() {
                            }
                            ui.text_colored([0.7, 0.7, 0.7, 1.0], "Drag & drop .dmoon files to load");
                            
                            scene_menu.end();
                        }
                        
                        ui.separator();
                        
                        // Save options with visual status
                        let has_unsaved = unsafe { UNSAVED_CHANGES };
                        if let Some(scene_path) = &self.current_scene_path {
                            let scene_name = scene_path.file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("Unknown");
                            
                            let save_label = if has_unsaved {
                                format!("{} Save Scene ({}) *", ICON_FLOPPY_DISK, scene_name)
                            } else {
                                format!("{} Save Scene ({})", ICON_FLOPPY_DISK, scene_name)
                            };
                            
                            if ui.menu_item(&save_label) {
                                if let Err(err) = self.save_current_scene(persisted) {
                                    log::error!("Failed to save current scene: {:#}", err);
                                } else {
                                    log::info!("Scene saved successfully!");
                                    unsafe { UNSAVED_CHANGES = false; }
                                }
                            }
                            
                            // Show save status
                            if has_unsaved {
                                ui.text_colored([1.0, 0.8, 0.0, 1.0], "  Unsaved changes");
                            } else {
                                ui.text_colored([0.0, 1.0, 0.0, 1.0], "  All changes saved");
                            }
                        } else {
                            ui.menu_item_config(&format!("{} Save Scene", ICON_FLOPPY_DISK)).enabled(false).build();
                            ui.text_colored([0.7, 0.7, 0.7, 1.0], "  No scene loaded");
                        }
                        
                        ui.separator();
                        ui.text_colored([0.6, 0.6, 0.6, 1.0], "Shortcut: S key for quick save");
                        
                        if ui.menu_item("Clear Scene") {
                            self.clear_scene_from_gui(persisted, ctx);
                        }
                        
                        file_menu.end();
                    }
                    if let Some(window_menu) = ui.begin_menu("Window") {
                        let show_assets = self.ui_windows.asset_browser.as_ref().map_or(false, |a| a.open && self.ui_windows.show_asset_browser);
                        if ui.menu_item_config("Assets Browser").selected(show_assets).build() {
                            if let Some(asset_browser) = self.ui_windows.asset_browser.as_mut() {
                                asset_browser.open = !asset_browser.open;
                                self.ui_windows.show_asset_browser = asset_browser.open;
                            }
                        }
                        if ui.menu_item_config("Hierarchy").selected(self.ui_windows.show_hierarchy).build() {
                            self.ui_windows.show_hierarchy = !self.ui_windows.show_hierarchy;
                        }
                        if ui.menu_item_config("Debug").selected(self.ui_windows.show_debug).build() {
                            self.ui_windows.show_debug = !self.ui_windows.show_debug;
                        }
                        
                        ui.separator();
                        if ui.menu_item("Reset Window Positions") {
                            // Reset all window positions to default
                            unsafe { RESET_WINDOW_POSITIONS = true; }
                        }
                        
                        window_menu.end();
                    }
                    if let Some(view_menu) = ui.begin_menu("View") {
                        if let Some(rendering_menu) = ui.begin_menu("Rendering Type") {
                            // Rasterization mode (RTX OFF)
                            let is_rasterization = !ctx.world_renderer.is_ray_tracing_enabled() && 
                                                  ctx.world_renderer.get_render_mode() == RenderMode::Standard;
                            if ui.menu_item_config("Rasterization").selected(is_rasterization).build() {
                                ctx.world_renderer.set_ray_tracing_enabled(false);
                                ctx.world_renderer.set_render_mode(RenderMode::Standard);
                            }
                            
                            // Ray Tracing mode
                            let is_ray_tracing = ctx.world_renderer.is_ray_tracing_enabled() && 
                                                ctx.world_renderer.get_render_mode() == RenderMode::Standard;
                            if ui.menu_item_config("Ray Tracing").selected(is_ray_tracing).build() {
                                ctx.world_renderer.set_ray_tracing_enabled(true);
                                ctx.world_renderer.set_render_mode(RenderMode::Standard);
                            }
                            
                            // Path Tracing mode (Reference)
                            let is_path_tracing = ctx.world_renderer.get_render_mode() == RenderMode::Reference;
                            if ui.menu_item_config("Path Tracing").selected(is_path_tracing).build() {
                                ctx.world_renderer.set_render_mode(RenderMode::Reference);
                            }
                            
                            ui.separator();
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], "Both Rasterization and Ray Tracing");
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], "now have full lighting & shadows!");
                            ui.text_colored([0.7, 0.7, 0.7, 1.0], "Use Debug > Shading Mode to control");
                            
                            rendering_menu.end();
                        }
                        view_menu.end();
                    }
                    bar.end();
                }

                if ui.collapsing_header("RTX", TreeNodeFlags::DEFAULT_OPEN) {
                    Drag::new("EV shift").range(-8.0, 12.0).speed(0.01).build(ui, &mut persisted.exposure.ev_shift);

                    ui.checkbox(
                        "Use dynamic exposure",
                        &mut persisted.exposure.use_dynamic_adaptation,
                    );

                    Drag::new("Adaptation speed").range(-4.0, 4.0).speed(0.01).build(ui, &mut persisted.exposure.dynamic_adaptation_speed);

                    Drag::new("Luminance histogram low clip").range(0.0, 1.0).speed(0.001).build(ui, &mut persisted.exposure.dynamic_adaptation_low_clip);
                    persisted.exposure.dynamic_adaptation_low_clip = persisted
                        .exposure
                        .dynamic_adaptation_low_clip
                        .clamp(0.0, 1.0);

                    Drag::new("Luminance histogram high clip").range(0.0, 1.0).speed(0.001).build(ui, &mut persisted.exposure.dynamic_adaptation_high_clip);
                    persisted.exposure.dynamic_adaptation_high_clip = persisted
                        .exposure
                        .dynamic_adaptation_high_clip
                        .clamp(0.0, 1.0);

                    Drag::new("Contrast").range(1.0, 1.5).speed(0.001).build(ui, &mut persisted.exposure.contrast);

                    Drag::new("Emissive multiplier").range(0.0, 10.0).speed(0.1).build(ui, &mut persisted.light.emissive_multiplier);

                    ui.checkbox(
                        "Enable emissive",
                        &mut persisted.light.enable_emissive,
                    );

                    Drag::new("Light intensity multiplier").range(0.0, 1000.0).speed(1.0).build(ui, &mut persisted.light.local_lights.multiplier);

                    Drag::new("Camera speed").range(0.0, 10.0).speed(0.025).build(ui, &mut persisted.movement.camera_speed);

                    Drag::new("Camera smoothness").range(0.0, 20.0).speed(0.1).build(ui, &mut persisted.movement.camera_smoothness);

                    Drag::new("Sun rotation smoothness").range(0.0, 20.0).speed(0.1).build(ui, &mut persisted.movement.sun_rotation_smoothness);

                    Drag::new("Field of view").range(1.0, 120.0).speed(0.25).build(ui, &mut persisted.camera.vertical_fov);

                    Drag::new("Sun size").range(0.0, 10.0).speed(0.02).build(ui, &mut persisted.light.sun.size_multiplier);

                    /*ui.checkbox(
                        "Object motion blur",
                        &mut persisted.post_process.enable_object_motion_blur,
                    );

                    ui.checkbox(
                        "TAA",
                        &mut persisted.post_process.enable_taa,
                    );

                    ui.checkbox(
                        "DOF",
                        &mut persisted.post_process.enable_dof,
                    );

                    ui.checkbox(
                        "DLSS",
                        &mut persisted.post_process.enable_dlss,
                    );

                    if persisted.post_process.enable_dlss {
                        Drag::new("DLSS ratio").range(0.1, 1.0).speed(0.01).build(ui, &mut persisted.post_process.dlss_ratio);
                    }

                    ui.checkbox(
                        "FSR",
                        &mut persisted.post_process.enable_fsr,
                    );

                    if persisted.post_process.enable_fsr {
                        Drag::new("FSR ratio").range(0.1, 1.0).speed(0.01).build(ui, &mut persisted.post_process.fsr_ratio);
                    }*/

                    /*ui.checkbox(
                        "SSGI",
                        &mut persisted.light.enable_ssgi,
                    );

                    if persisted.light.enable_ssgi {
                        Drag::new("SSGI multiplier").range(0.0, 10.0).speed(0.1).build(ui, &mut persisted.light.ssgi.multiplier);
                    }

                    ui.checkbox(
                        "RTGI",
                        &mut persisted.light.enable_rtgi,
                    );

                    if persisted.light.enable_rtgi {
                        ui.checkbox(
                            "RTGI enable",
                            &mut persisted.light.rtgi.enable,
                        );

                        Drag::new("RTGI multiplier").range(0.0, 10.0).speed(0.1).build(ui, &mut persisted.light.rtgi.multiplier);

                        ui.drag_float("RTGI rays per pixel", &mut persisted.light.rtgi.rays_per_pixel)
                            .range(1, 16)
                            .build();

                        ui.drag_float("RTGI pixel offset", &mut persisted.light.rtgi.pixel_offset)
                            .range(0.1..=5.0)
                            .speed(0.1)
                            .build();

                        ui.drag_float("RTGI ray length", &mut persisted.light.rtgi.ray_length)
                            .range(0.1..=20.0)
                            .speed(0.1)
                            .build();

                        ui.drag_float("RTGI roughness bias", &mut persisted.light.rtgi.roughness_bias)
                            .range(0.0..=0.5)
                            .speed(0.01)
                            .build();

                        ui.checkbox(
                            "RTGI show debug",
                            &mut persisted.light.rtgi.show_debug,
                        );
                    }*/

                    /*ui.checkbox(
                        "Show world radiance cache",
                        &mut ctx.world_renderer.debug_show_wrc,
                    );*/

                    /*if ui.radio_button_bool(
                        "Move sun",
                        left_click_edit_mode == LeftClickEditMode::MoveSun,
                    ) {
                        left_click_edit_mode = LeftClickEditMode::MoveSun;
                    }

                    if ui.radio_button_bool(
                        "Move local lights",
                        left_click_edit_mode == LeftClickEditMode::MoveLocalLights,
                    ) {
                        left_click_edit_mode = LeftClickEditMode::MoveLocalLights;
                    }

                    imgui::Drag::<u32>::new("Light count")
                        .range(0, 10)
                        .build(ui, &mut state.lights.count);*/

                    ui.checkbox(
                        "Scroll irradiance cache",
                        &mut ctx.world_renderer.ircache.enable_scroll,
                    );

                    Drag::new("GI spatial reuse passes").range(1, 3).build(ui, &mut ctx.world_renderer.rtdgi.spatial_reuse_pass_count);

                    ctx.world_renderer.rtdgi.spatial_reuse_pass_count = ctx
                        .world_renderer
                        .rtdgi
                        .spatial_reuse_pass_count
                        .clamp(1, 3);

                    ui.checkbox(
                        "Ray-traced reservoir visibility",
                        &mut ctx.world_renderer.rtdgi.use_raytraced_reservoir_visibility,
                    );

                    ui.checkbox(
                        "Allow diffuse ray reuse for reflections",
                        &mut ctx.world_renderer.rtr.reuse_rtdgi_rays,
                    );

                    #[cfg(feature = "dlss")]
                    {
                        ui.checkbox("Use DLSS", &mut ctx.world_renderer.use_dlss);
                    }
                }

                if ui.collapsing_header("Scene", TreeNodeFlags::DEFAULT_OPEN)
                {
                    if let Some(ibl) = persisted.scene.ibl.as_ref() {
                        ui.text(format!("IBL: {:?}", ibl));
                        if ui.button("Unload") {
                            ctx.world_renderer.ibl.unload_image();
                            persisted.scene.ibl = None;
                        }
                    } else {
                        ui.text("Drag a sphere-mapped .hdr/.exr to load as IBL");
                    }

                    // --- Hierarchy ---
                    if ui.collapsing_header("Hierarchy", TreeNodeFlags::DEFAULT_OPEN)
                    {
                        for (idx, elem) in persisted.scene.elements.iter().enumerate() {
                            let element_icon = Self::get_element_icon(elem);
                            let element_name = if let Some(name) = elem.mesh_nodes.get(0).and_then(|n| n.name.as_ref()) {
                                name.clone()
                            } else {
                                format!("{:?}", elem.source)
                            };
                            let element_label = create_icon_label(element_icon, &element_name);
                            
                            if elem.is_compound && !elem.mesh_nodes.is_empty() {
                                ui.tree_node_config(format!("{}##{}", element_label, idx))
                                    .build(|| {
                                        for (nidx, node) in elem.mesh_nodes.iter().enumerate() {
                                            let node_icon = Self::get_node_icon();
                                            let node_name = if let Some(n) = &node.name {
                                                n.clone()
                                            } else {
                                                format!("Node {}", nidx)
                                            };
                                            let node_label = create_icon_label(node_icon, &node_name);
                                            ui.bullet_text(format!("{}##{}-{}", node_label, idx, nidx));
                                        }
                                    });
                            } else {
                                ui.bullet_text(format!("{}##{}", element_label, idx));
                            }
                        }
                    }

                    let mut element_to_remove = None;
                    for (idx, elem) in persisted.scene.elements.iter_mut().enumerate() {
                        ui.dummy([0.0, 10.0]);

                        let id_token = ui.push_id_usize(idx);
                        ui.text(format!("{:?}", elem.source));

                        {
                            ui.set_next_item_width(200.0);

                            let mut scale = elem.transform.scale.x;
                            Drag::new("scale").range(0.001, 1000.0).speed(1.0).build(ui, &mut scale);
                            if scale != elem.transform.scale.x {
                                elem.transform.scale = Vec3::splat(scale);
                            }
                        }

                        ui.same_line();
                        if ui.button("Delete") {
                            element_to_remove = Some(idx);
                        }

                        // Position
                        {
                            ui.set_next_item_width(100.0);
                            Drag::new("x").speed(0.01).build(ui, &mut elem.transform.position.x);

                            ui.same_line();

                            ui.set_next_item_width(100.0);
                            Drag::new("y").speed(0.01).build(ui, &mut elem.transform.position.y);

                            ui.same_line();

                            ui.set_next_item_width(100.0);
                            Drag::new("z").speed(0.01).build(ui, &mut elem.transform.position.z);
                        }

                        // Rotation
                        {
                            ui.set_next_item_width(100.0);
                            Drag::new("rx").speed(0.1).build(ui, &mut elem.transform.rotation_euler_degrees.x);

                            ui.same_line();

                            ui.set_next_item_width(100.0);
                            Drag::new("ry").speed(0.1).build(ui, &mut elem.transform.rotation_euler_degrees.y);

                            ui.same_line();

                            ui.set_next_item_width(100.0);
                            Drag::new("rz").speed(0.1).build(ui, &mut elem.transform.rotation_euler_degrees.z);
                        }

                        id_token.pop();
                    }

                    if let Some(idx) = element_to_remove {
                        let elem = persisted.scene.elements.remove(idx);
                        ctx.world_renderer.remove_instance(elem.instance);
                    }
                }

                // Frustum Culling settings
                if ui.collapsing_header("Frustum Culling", TreeNodeFlags::DEFAULT_OPEN)
                {
                    ui.checkbox(
                        "Enable frustum culling",
                        &mut persisted.frustum_culling.enabled,
                    );

                    ui.checkbox(
                        "Debug logging",
                        &mut persisted.frustum_culling.debug_logging,
                    );

                    ui.checkbox(
                        "Use sphere culling (faster)",
                        &mut persisted.frustum_culling.use_sphere_culling,
                    );

                    // Culling method selection
                    ui.text("Culling Method:");
                    let current_method = &mut persisted.frustum_culling.culling_method;
                    
                    let mut is_emissive = matches!(current_method, crate::culling::CullingMethod::EmissiveMultiplier);
                    let mut is_move_away = matches!(current_method, crate::culling::CullingMethod::MoveAway);
                    let mut is_scale_zero = matches!(current_method, crate::culling::CullingMethod::ScaleToZero);
                    
                    if ui.checkbox("Emissive Multiplier", &mut is_emissive) && is_emissive {
                        *current_method = crate::culling::CullingMethod::EmissiveMultiplier;
                    }
                    if ui.checkbox("Move Away", &mut is_move_away) && is_move_away {
                        *current_method = crate::culling::CullingMethod::MoveAway;
                    }
                    if ui.checkbox("Scale to Zero", &mut is_scale_zero) && is_scale_zero {
                        *current_method = crate::culling::CullingMethod::ScaleToZero;
                    }
                    
                    // Show description for the selected method
                    ui.separator();
                    ui.text("Method Description:");
                    match current_method {
                        crate::culling::CullingMethod::EmissiveMultiplier => {
                            ui.text_wrapped("Makes objects invisible by setting emissive to 0. Least GPU-efficient.");
                        }
                        crate::culling::CullingMethod::MoveAway => {
                            ui.text_wrapped("Moves objects far away. More GPU-efficient as objects are naturally culled by depth.");
                        }
                        crate::culling::CullingMethod::ScaleToZero => {
                            ui.text_wrapped("Scales objects to zero size. Very GPU-efficient for triangle culling.");
                        }
                    }

                    Drag::new("Default object size").range(0.1, 10.0).speed(0.1).build(ui, &mut persisted.frustum_culling.default_object_size);

                    Drag::new("Log interval (frames)").range(30, 600).speed(10.0).build(ui, &mut persisted.frustum_culling.log_interval_frames);

                    // Display culling statistics
                    ui.text("Culling Stats:");
                    
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
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], "Status: Enabled");
                        ui.text(format!(
                            "Method: {}",
                            if persisted.frustum_culling.use_sphere_culling {
                                "Sphere"
                            } else {
                                "AABB"
                            }
                        ));
                    } else {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], "Status: Disabled");
                    }
                }

                // Occlusion Culling settings
                if imgui::CollapsingHeader::new("Occlusion Culling")
                    .default_open(false)
                    .build(ui)
                {
                    ui.checkbox(
                        "Enable occlusion culling",
                        &mut persisted.occlusion_culling.enabled,
                    );

                    ui.checkbox(
                        "Debug visualization",
                        &mut persisted.occlusion_culling.debug_visualize,
                    );

                    Drag::new("Depth buffer resolution")
                        .range(64, 512)
                        .speed(1.0)
                        .build(ui, &mut persisted.occlusion_culling.depth_buffer_resolution);

                    Drag::new("Depth bias")
                        .range(0.0, 0.1)
                        .speed(0.001)
                        .build(ui, &mut persisted.occlusion_culling.depth_bias);

                    Drag::new("Sample count per object")
                        .range(1, 8)
                        .speed(1.0)
                        .build(ui, &mut persisted.occlusion_culling.sample_count);

                    Drag::new("Max test distance")
                        .range(10.0, 5000.0)
                        .speed(10.0)
                        .build(ui, &mut persisted.occlusion_culling.max_test_distance);

                    ui.separator();
                    ui.text("Occlusion Culling Info:");
                    ui.text_wrapped("Hides objects that are blocked by other objects closer to the camera. Works in combination with frustum culling for maximum efficiency.");
                    
                    if persisted.occlusion_culling.enabled {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], "Status: Enabled");
                        ui.text(format!("Depth resolution: {}x{}", 
                            persisted.occlusion_culling.depth_buffer_resolution,
                            persisted.occlusion_culling.depth_buffer_resolution));
                    } else {
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], "Status: Disabled");
                    }
                }

                // Triangle Culling settings
                if imgui::CollapsingHeader::new("Triangle Culling")
                    .default_open(false)
                    .build(ui)
                {
                    ui.checkbox(
                        "Enable triangle culling",
                        &mut persisted.triangle_culling.enabled,
                    );
                    ui.same_line();
                    ui.text_colored([0.7, 0.7, 0.7, 1.0], "Per-triangle visibility tests");

                    if persisted.triangle_culling.enabled {
                        ui.separator();
                        
                        ui.checkbox(
                            "Debug logging",
                            &mut persisted.triangle_culling.debug_logging,
                        );
                        
                            Drag::new("Log interval (frames)")
                                .range(1, 300)
                                .speed(1.0)
                                .build(ui, &mut persisted.triangle_culling.log_interval_frames);                        ui.separator();
                        ui.text("Culling Methods:");

                        // Back-face culling checkbox
                        let mut has_backface = persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::BackFace);
                        if ui.checkbox("Back-face culling", &mut has_backface) {
                            if has_backface {
                                if !persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::BackFace) {
                                    persisted.triangle_culling.methods.push(crate::math::PrimitiveCullingMethod::BackFace);
                                }
                            } else {
                                persisted.triangle_culling.methods.retain(|m| m != &crate::math::PrimitiveCullingMethod::BackFace);
                            }
                        }
                        ui.same_line();
                        ui.text_colored([0.7, 0.7, 0.7, 1.0], "Hide faces pointing away");

                        // Small triangle culling checkbox
                        let mut has_small = persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::SmallTriangle);
                        if ui.checkbox("Small triangle culling", &mut has_small) {
                            if has_small {
                                if !persisted.triangle_culling.methods.contains(&crate::math::PrimitiveCullingMethod::SmallTriangle) {
                                    persisted.triangle_culling.methods.push(crate::math::PrimitiveCullingMethod::SmallTriangle);
                                }
                            } else {
                                persisted.triangle_culling.methods.retain(|m| m != &crate::math::PrimitiveCullingMethod::SmallTriangle);
                            }
                        }
                        ui.same_line();
                        ui.text_colored([0.7, 0.7, 0.7, 1.0], "Hide very small triangles");

                        ui.separator();
                        ui.text("Parameters:");

                        Drag::new("Min triangle area (pixels)")
                            .range(0.1, 100.0)
                            .speed(0.1)
                            .build(ui, &mut persisted.triangle_culling.min_triangle_area);

                        Drag::new("Back-face epsilon")
                            .range(0.0, 0.1)
                            .speed(0.001)
                            .build(ui, &mut persisted.triangle_culling.backface_epsilon);

                        Drag::new("Max distance")
                            .range(10.0, 5000.0)
                            .speed(10.0)
                            .build(ui, &mut persisted.triangle_culling.max_distance);
                    }

                    ui.separator();
                    ui.text("Triangle Culling Info:");
                    ui.text_wrapped("Culls individual triangles based on various criteria. Works at the finest level of detail, complementing object-level frustum and occlusion culling.");
                    
                    if persisted.triangle_culling.enabled {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], "Status: Enabled");
                        ui.text(format!("Active methods: {}", persisted.triangle_culling.methods.len()));
                        
                        // Show triangle culling statistics
                        let triangle_stats = self.get_triangle_culling_statistics();
                        if triangle_stats.triangles_tested > 0 {
                            ui.separator();
                            ui.text("Triangle Statistics:");
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
                        ui.text_colored([1.0, 0.0, 0.0, 1.0], "Status: Disabled");
                    }
                }

                // Resource Streaming Section
                if imgui::CollapsingHeader::new("Resource Streaming")
                    .default_open(false)
                    .build(ui)
                {
                    self.streaming_integration.render_gui(ui);
                }

                if imgui::CollapsingHeader::new("Overrides")
                    .default_open(false)
                    .build(ui)
                {
                    macro_rules! do_flag {
                        ($flag:path, $name:literal) => {
                            let mut is_set: bool =
                                ctx.world_renderer.render_overrides.has_flag($flag);
                            ui.checkbox($name, &mut is_set);
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

                    Drag::new("Roughness scale")
                        .range(0.0, 4.0)
                        .speed(0.001)
                        .build(
                            ui,
                            &mut ctx.world_renderer.render_overrides.material_roughness_scale,
                        );
                }

                if imgui::CollapsingHeader::new("Sequence")
                    .default_open(false)
                    .build(ui)
                {
                    if ui.button("Add key") {
                        self.add_sequence_keyframe(persisted);
                    }

                    ui.same_line();
                    if self.is_sequence_playing() {
                        if ui.button("Stop") {
                            self.stop_sequence();
                        }
                    } else if ui.button("Play") {
                        self.play_sequence(persisted);
                    }

                    ui.same_line();
                    ui.set_next_item_width(60.0);
                    Drag::new("Speed")
                        .range(0.0, 4.0)
                        .speed(0.01)
                        .build(ui, &mut self.sequence_playback_speed);

                    if self.active_camera_key.is_some() {
                        ui.same_line();
                        if ui.button("Deselect key") {
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
                            format!("-> {}:", i)
                        } else {
                            format!("{}:", i)
                        };

                        if ui.button(&label) {
                            cmd = Cmd::JumpToKey(i);
                        }

                        ui.same_line();
                        ui.set_next_item_width(60.0);
                        ui.input_float(format!("duration##{}", i), &mut item.duration);

                        ui.same_line();
                        ui.checkbox(
                            &format!("Pos##{}", i),
                            &mut item.value.camera_position.is_some,
                        );

                        ui.same_line();
                        ui.checkbox(
                            &format!("Dir##{}", i),
                            &mut item.value.camera_direction.is_some,
                        );

                        ui.same_line();
                        ui.checkbox(&format!("Sun##{}", i), &mut item.value.towards_sun.is_some);

                        ui.same_line();
                        if ui.button(&format!("Delete##{}", i)) {
                            cmd = Cmd::DeleteKey(i);
                        }

                        ui.same_line();
                        if ui.button(&format!("Replace##{}:", i)) {
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
                    if imgui::CollapsingHeader::new("Debug")
                        .default_open(false)
                        .build(ui)
                    {
                        if ui.radio_button_bool(
                            "Scene geometry",
                            ctx.world_renderer.debug_mode == RenderDebugMode::None,
                        ) {
                            ctx.world_renderer.debug_mode = RenderDebugMode::None;
                        }

                        /*if ui.radio_button_bool(
                            "World radiance cache",
                            ctx.world_renderer.debug_mode == RenderDebugMode::WorldRadianceCache,
                        ) {
                            ctx.world_renderer.debug_mode = RenderDebugMode::WorldRadianceCache;
                        }*/

                        /*ui.combo_box_simple_string(
                            "Shading",
                            &mut ctx.world_renderer.debug_shading_mode,
                            &[
                                "Default",
                                "No base color",
                                "Diffuse GI",
                                "Reflections",
                                "RTX OFF",
                                "Irradiance cache",
                            ],
                        );*/

                        // Manual shading mode control - now independent of ray tracing mode
                        ui.text("Shading Mode:");
                        if ui.radio_button_bool("Default (Full Lighting)", ctx.world_renderer.debug_shading_mode == 0) {
                            ctx.world_renderer.debug_shading_mode = 0;
                        }
                        if ui.radio_button_bool("No Base Color", ctx.world_renderer.debug_shading_mode == 1) {
                            ctx.world_renderer.debug_shading_mode = 1;
                        }
                        if ui.radio_button_bool("Diffuse GI Only", ctx.world_renderer.debug_shading_mode == 2) {
                            ctx.world_renderer.debug_shading_mode = 2;
                        }
                        if ui.radio_button_bool("Reflections Only", ctx.world_renderer.debug_shading_mode == 3) {
                            ctx.world_renderer.debug_shading_mode = 3;
                        }
                        if ui.radio_button_bool("RTX OFF (No Shadows)", ctx.world_renderer.debug_shading_mode == 4) {
                            ctx.world_renderer.debug_shading_mode = 4;
                        }
                        if ui.radio_button_bool("Irradiance Cache", ctx.world_renderer.debug_shading_mode == 5) {
                            ctx.world_renderer.debug_shading_mode = 5;
                        }
                        
                        ui.separator();

                        Drag::new("Max FPS").range(1, MAX_FPS_LIMIT).build(ui, &mut self.max_fps);

                        ui.checkbox("Allow pass overlap", unsafe {
                            &mut kajiya::rg::RG_ALLOW_PASS_OVERLAP
                        });
                    }
                }

                if imgui::CollapsingHeader::new("GPU passes")
                    .default_open(true)
                    .build(ui)
                {
                    ui.text(format!("CPU frame time: {:.3}ms", ctx.dt_filtered * 1000.0));

                    // GPU profiler is not available in this build
                    ui.text("GPU profiling disabled");
                }
                
                // Handle save request within the scope where variables are defined
                if save_scene_requested {
                    if let Err(err) = self.save_current_scene(persisted) {
                        log::error!("Failed to save scene: {:#}", err);
                    } else {
                        log::info!("Scene saved successfully!");
                        unsafe { UNSAVED_CHANGES = false; }
                    }
                }
                
                } // Close the if self.show_gui block
                
                // Reset window positions flag after frame
                unsafe {
                    if RESET_WINDOW_POSITIONS {
                        RESET_WINDOW_POSITIONS = false;
                        log::info!("Window positions reset to default");
                    }
                }
                });
                log::debug!("ImGui frame callback completed");
            } else {
                log::warn!("Failed to take ImGui context - ctx.imgui was None!");
            }
        } else {
            log::debug!("GUI skipped: show_gui={}, is_compiling={}, should_show_gui={}", 
                self.show_gui, is_compiling, should_show_gui);
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
                    
                    // Use window builder pattern for imgui 0.11
                    ui.window("Compiling Shaders")
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
                        .build(|| {
                            ui.text("Initializing rendering engine...");
                            ui.spacing();

                            // Progress bar
                            let progress_fraction = if progress.total_shaders > 0 {
                                progress.progress_percentage() / 100.0
                            } else {
                                0.0 // Indeterminate progress when no shaders registered yet
                            };
                            
                            ProgressBar::new(progress_fraction)
                                .size([450.0, 20.0])
                                .overlay_text(format!("{:.1}%", progress.progress_percentage()))
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
