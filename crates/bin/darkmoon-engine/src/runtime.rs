#![allow(clippy::single_match)]

use anyhow::Context;

use dolly::prelude::*;
use gltf;
use dolly::glam::{Mat4, Vec3};
use kajiya::{
    rg::GraphDebugHook,
    world_renderer::{AddMeshOptions, MeshHandle, WorldRenderer},
};
use kajiya_simple::*;

use crate::{
    opt::Opt,
    persisted::{MeshSource, SceneElement, SceneElementTransform, MeshNode, ShouldResetPathTracer as _},
    scene::SceneDesc,
    sequence::{CameraPlaybackSequence, MemOption, SequenceValue},
    PersistedState,
    math::{Aabb, Frustum, OcclusionCuller, TriangleCuller},
    culling::CullingMethod,
};

use crate::keymap::KeymapConfig;
use log::{info, warn};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::File,
    hash::{Hash, Hasher},
    path::PathBuf,
};

pub const MAX_FPS_LIMIT: u32 = 256;

pub struct UiWindowsState {
    pub show_asset_browser: bool,
    pub show_hierarchy: bool,
    pub show_debug: bool,
    pub asset_browser: Option<crate::asset_browser::AssetBrowser>,
}

impl Default for UiWindowsState {
    fn default() -> Self {
        Self {
            show_asset_browser: true,
            show_hierarchy: true,
            show_debug: true,
            asset_browser: None,
        }
    }
}

pub struct RuntimeState {
    pub camera: CameraRig,
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
    pub keymap_config: KeymapConfig,
    pub movement_map: KeyboardMap,

    pub show_gui: bool,
    pub sun_direction_interp: Vec3,
    pub left_click_edit_mode: LeftClickEditMode,

    pub max_fps: u32,
    pub locked_rg_debug_hook: Option<GraphDebugHook>,
    pub grab_cursor_pos: winit::dpi::PhysicalPosition<f64>,

    pub reset_path_tracer: bool,

    pub active_camera_key: Option<usize>,
    sequence_playback_state: SequencePlaybackState,
    pub sequence_playback_speed: f32,

    known_meshes: HashMap<PathBuf, MeshHandle>,
    occlusion_culler: OcclusionCuller,
    triangle_culler: TriangleCuller,
    pub streaming_integration: crate::streaming_integration::StreamingIntegration,
    pub ui_windows: UiWindowsState,
}

enum SequencePlaybackState {
    NotPlaying,
    Playing {
        t: f32,
        sequence: CameraPlaybackSequence,
    },
}

impl RuntimeState {
    pub fn new(
        persisted: &mut PersistedState,
        world_renderer: &mut WorldRenderer,
        opt: &Opt,
    ) -> Self {
        let camera: CameraRig = CameraRig::builder()
            .with(Position::new(persisted.camera.position))
            .with(YawPitch::new().rotation_quat(persisted.camera.rotation))
            .with(Smooth::default())
            .build();

        // Mitsuba match
        /*let mut camera = camera::FirstPersonCamera::new(Vec3::new(-2.0, 4.0, 8.0));
        camera.fov = 35.0 * 9.0 / 16.0;
        camera.look_at(Vec3::new(0.0, 0.75, 0.0));*/

        let mouse: MouseState = Default::default();
        let keyboard: KeyboardState = Default::default();

        let keymap_config = KeymapConfig::load(&opt.keymap).unwrap_or_else(|err| {
            warn!("Failed to load keymap: {}", err);
            info!("Using default keymap");
            KeymapConfig::default()
        });

        let sun_direction_interp = persisted.light.sun.controller.towards_sun();

        let mut res = Self {
            camera,
            mouse,
            keyboard,
            keymap_config: keymap_config.clone(),
            movement_map: keymap_config.movement.into(),

            show_gui: true,
            sun_direction_interp,
            left_click_edit_mode: LeftClickEditMode::MoveSun,

            max_fps: MAX_FPS_LIMIT,
            locked_rg_debug_hook: None,
            grab_cursor_pos: Default::default(),

            reset_path_tracer: false,

            active_camera_key: None,
            sequence_playback_state: SequencePlaybackState::NotPlaying,
            sequence_playback_speed: 1.0,

            known_meshes: Default::default(),
            occlusion_culler: OcclusionCuller::new(persisted.occlusion_culling.clone()),
            triangle_culler: TriangleCuller::new(persisted.triangle_culling.clone()),
            streaming_integration: crate::streaming_integration::StreamingIntegration::new(),
            ui_windows: UiWindowsState::default(),
        };

        // Load meshes that the persisted scene was referring to
        persisted.scene.elements.retain_mut(|elem| {
            match res.load_mesh(world_renderer, &elem.source) {
                Ok(mesh) => {
                    elem.instance =
                        world_renderer.add_instance(mesh, elem.transform.affine_transform());
                    true
                }
                Err(err) => {
                    log::error!("Failed to load mesh {:?}: {:#}", elem.source, err);
                    false
                }
            }
        });

        // Load the IBL too
        if let Some(ibl) = persisted.scene.ibl.as_ref() {
            if world_renderer.ibl.load_image(ibl).is_err() {
                persisted.scene.ibl = None;
            }
        }

        // Initialize streaming system automatically
        res.streaming_integration.request_initialization();
        log::info!("Resource streaming system initialized automatically at startup");

        res
    }

    pub fn clear_scene(
        &mut self,
        persisted: &mut PersistedState,
        world_renderer: &mut WorldRenderer,
    ) {
        for elem in persisted.scene.elements.drain(..) {
            world_renderer.remove_instance(elem.instance);
        }
    }

    /// Convenience method for clearing scene from GUI (takes FrameContext)
    pub fn clear_scene_from_gui(
        &mut self,
        persisted: &mut PersistedState,
        ctx: &mut FrameContext,
    ) {
        for elem in persisted.scene.elements.drain(..) {
            ctx.world_renderer.remove_instance(elem.instance);
        }
    }

    pub fn load_scene(
        &mut self,
        persisted: &mut PersistedState,
        world_renderer: &mut WorldRenderer,
        scene_path: impl Into<PathBuf>,
    ) -> anyhow::Result<()> {
        let scene_path = scene_path.into();
        let scene_desc: SceneDesc = ron::de::from_reader(
            File::open(&scene_path)
                .with_context(|| format!("Opening scene file {:?}", scene_path))?,
        )?;

        self.clear_scene(persisted, world_renderer);

        for instance in scene_desc.instances {
            let mesh_path = canonical_path_from_vfs(&instance.mesh)
                .with_context(|| format!("Mesh path: {:?}", instance.mesh))
                .expect("valid mesh path");

            let mesh = self
                .load_mesh(world_renderer, &MeshSource::File(mesh_path.clone()))
                .with_context(|| format!("Mesh path: {:?}", instance.mesh))
                .expect("valid mesh");

            let transform = SceneElementTransform {
                position: instance.position.into(),
                rotation_euler_degrees: instance.rotation.into(),
                scale: instance.scale.into(),
            };

            let render_instance = world_renderer.add_instance(mesh, transform.affine_transform());

            persisted.scene.elements.push(SceneElement {
                source: MeshSource::File(mesh_path),
                instance: render_instance,
                transform,
                bounding_box: None, // Will be calculated later when mesh data is available
                mesh_nodes: Vec::new(),
                is_compound: false,
            });
        }

        Ok(())
    }

    /// Convenience method for loading a scene from a path string (used by the GUI)
    pub fn load_scene_from_path(
        &mut self,
        persisted: &mut PersistedState,
        ctx: &mut FrameContext,
        path: &str,
    ) -> anyhow::Result<()> {
        self.load_scene(persisted, &mut ctx.world_renderer, path)
    }

    fn update_camera(&mut self, persisted: &mut PersistedState, ctx: &FrameContext) {
        let smooth = self.camera.driver_mut::<Smooth>();
        if ctx.world_renderer.render_mode == RenderMode::Reference {
            smooth.position_smoothness = 0.0;
            smooth.rotation_smoothness = 0.0;
        } else {
            smooth.position_smoothness = persisted.movement.camera_smoothness;
            smooth.rotation_smoothness = persisted.movement.camera_smoothness;
        }

        // When starting camera rotation, hide the mouse cursor, and capture it to the window.
        if (self.mouse.buttons_pressed & (1 << 2)) != 0 {
            let _ = ctx.window.set_cursor_grab(true);
            self.grab_cursor_pos = self.mouse.physical_position;
            ctx.window.set_cursor_visible(false);
        }

        // When ending camera rotation, release the cursor.
        if (self.mouse.buttons_released & (1 << 2)) != 0 {
            let _ = ctx.window.set_cursor_grab(false);
            ctx.window.set_cursor_visible(true);
        }

        let input = self.movement_map.map(&self.keyboard, ctx.dt_filtered);
        let move_vec = self.camera.final_transform.rotation
            * Vec3::new(input["move_right"], input["move_up"], -input["move_fwd"])
                .clamp_length_max(1.0)
            * 4.0f32.powf(input["boost"]);

        if (self.mouse.buttons_held & (1 << 2)) != 0 {
            // While we're rotating, the cursor should not move, so that upon revealing it,
            // it will be where we started the rotation motion at.
            let _ = ctx
                .window
                .set_cursor_position(winit::dpi::PhysicalPosition::new(
                    self.grab_cursor_pos.x,
                    self.grab_cursor_pos.y,
                ));

            let sensitivity = 0.1;
            self.camera.driver_mut::<YawPitch>().rotate_yaw_pitch(
                -sensitivity * self.mouse.delta.x,
                -sensitivity * self.mouse.delta.y,
            );
        }

        self.camera
            .driver_mut::<Position>()
            .translate(move_vec * ctx.dt_filtered * persisted.movement.camera_speed);

        if let SequencePlaybackState::Playing { t, sequence } = &mut self.sequence_playback_state {
            let smooth = self.camera.driver_mut::<Smooth>();
            if *t <= 0.0 {
                smooth.position_smoothness = 0.0;
                smooth.rotation_smoothness = 0.0;
            } else {
                smooth.position_smoothness = persisted.movement.camera_smoothness;
                smooth.rotation_smoothness = persisted.movement.camera_smoothness;
            }

            if let Some(value) = sequence.sample(t.max(0.0)) {
                self.camera.driver_mut::<Position>().position = value.camera_position;
                self.camera
                    .driver_mut::<YawPitch>()
                    .set_rotation_quat(dolly::util::look_at::<dolly::handedness::RightHanded>(
                        value.camera_direction,
                    ));
                persisted
                    .light
                    .sun
                    .controller
                    .set_towards_sun(value.towards_sun);

                *t += ctx.dt_filtered * self.sequence_playback_speed;
            } else {
                self.sequence_playback_state = SequencePlaybackState::NotPlaying;
            }
        }

        self.camera.update(ctx.dt_filtered);

        persisted.camera.position = self.camera.final_transform.position;
        persisted.camera.rotation = self.camera.final_transform.rotation;

        if self
            .keyboard
            .was_just_pressed(self.keymap_config.misc.print_camera_transform)
        {
            println!(
                "position: {}, look_at: {}",
                persisted.camera.position,
                persisted.camera.position + persisted.camera.rotation * -Vec3::Z,
            );
        }
    }

    fn update_sun(&mut self, persisted: &mut PersistedState, ctx: &mut FrameContext) {
        if self.mouse.buttons_held & 1 != 0 {
            let delta_x =
                (self.mouse.delta.x / ctx.render_extent[0] as f32) * std::f32::consts::TAU;
            let delta_y = (self.mouse.delta.y / ctx.render_extent[1] as f32) * std::f32::consts::PI;

            match self.left_click_edit_mode {
                LeftClickEditMode::MoveSun => {
                    let ref_frame = Quat::from_xyzw(
                        0.0,
                        persisted.camera.rotation.y,
                        0.0,
                        persisted.camera.rotation.w,
                    )
                    .normalize();

                    persisted
                        .light
                        .sun
                        .controller
                        .view_space_rotate(&ref_frame, delta_x, delta_y);
                } /*LeftClickEditMode::MoveLocalLights => {
                      persisted.light.lights.theta += theta_delta;
                      persisted.light.lights.phi += phi_delta;
                  }*/
            }
        }

        //state.sun.phi += dt;
        //state.sun.phi %= std::f32::consts::TAU;

        let sun_direction = persisted.light.sun.controller.towards_sun();
        if (sun_direction.dot(self.sun_direction_interp) - 1.0).abs() > 1e-5 {
            self.reset_path_tracer = true;
        }

        let sun_interp_t = if ctx.world_renderer.render_mode == RenderMode::Reference {
            1.0
        } else {
            (-1.0 * persisted.movement.sun_rotation_smoothness).exp2()
        };

        self.sun_direction_interp =
            Vec3::lerp(self.sun_direction_interp, sun_direction, sun_interp_t).normalize();

        ctx.world_renderer.sun_size_multiplier = persisted.light.sun.size_multiplier;
    }

    fn update_lights(&mut self, persisted: &mut PersistedState, ctx: &mut FrameContext) {
        if self.keyboard.was_just_pressed(
            self.keymap_config
                .rendering
                .switch_to_reference_path_tracing,
        ) {
            match ctx.world_renderer.render_mode {
                RenderMode::Standard => {
                    //camera.convergence_sensitivity = 1.0;
                    ctx.world_renderer.render_mode = RenderMode::Reference;
                }
                RenderMode::Reference => {
                    //camera.convergence_sensitivity = 0.0;
                    ctx.world_renderer.render_mode = RenderMode::Standard;
                }
            };
        }

        if self
            .keyboard
            .was_just_pressed(self.keymap_config.rendering.light_enable_emissive)
        {
            persisted.light.enable_emissive = !persisted.light.enable_emissive;
        }

        /*if self.keyboard.is_down(VirtualKeyCode::Z) {
            persisted.light.local_lights.distance /= 0.99;
        }
        if self.keyboard.is_down(VirtualKeyCode::X) {
            persisted.light.local_lights.distance *= 0.99;
        }*/

        /*#[allow(clippy::comparison_chain)]
        if light_instances.len() > state.lights.count as usize {
            for extra_light in light_instances.drain(state.lights.count as usize..) {
                ctx.world_renderer.remove_instance(extra_light);
            }
        } else if light_instances.len() < state.lights.count as usize {
            light_instances.extend(
                (0..(state.lights.count as usize - light_instances.len())).map(|_| {
                    ctx.world_renderer
                        .add_instance(light_mesh, Vec3::ZERO, Quat::IDENTITY)
                }),
            );
        }

        for (i, inst) in light_instances.iter().enumerate() {
            let ring_rot = Quat::from_rotation_y(
                (i as f32) / light_instances.len() as f32 * std::f32::consts::TAU,
            );

            let rot =
                Quat::from_euler(EulerRot::YXZ, -state.lights.theta, -state.lights.phi, 0.0)
                    * ring_rot;
            ctx.world_renderer.set_instance_transform(
                *inst,
                rot * (Vec3::Z * state.lights.distance) + Vec3::new(0.1, 1.2, 0.0),
                rot,
            );

            ctx.world_renderer
                .get_instance_dynamic_parameters_mut(*inst)
                .emissive_multiplier = state.lights.multiplier;
        }*/
    }

    fn update_objects(&mut self, persisted: &mut PersistedState, ctx: &mut FrameContext) {
        let emissive_toggle_mult = if persisted.light.enable_emissive {
            1.0
        } else {
            0.0
        };

        let mut visible_objects = 0;
        let mut total_sub_objects = 0;
        let mut frustum_culled = 0;
        let mut occlusion_culled = 0;
        let total_elements = persisted.scene.elements.len();
        let frustum_culling_enabled = persisted.frustum_culling.enabled;
        let occlusion_culling_enabled = persisted.occlusion_culling.enabled;
        let triangle_culling_enabled = persisted.triangle_culling.enabled;

        // Update occlusion culler config if changed
        self.occlusion_culler.update_config(persisted.occlusion_culling.clone());
        
        // Update triangle culler config if changed
        self.triangle_culler.update_config(persisted.triangle_culling.clone());

        // Only create frustum if culling is enabled
        let (frustum, view_proj_matrix) = if frustum_culling_enabled || occlusion_culling_enabled {
            let lens = CameraLens {
                aspect_ratio: ctx.aspect_ratio(),
                vertical_fov: persisted.camera.vertical_fov,
                ..Default::default()
            };

            let camera_matrices = self
                .camera
                .final_transform
                .into_position_rotation()
                .through(&lens);

            let view_proj = camera_matrices.view_to_clip * camera_matrices.world_to_view;
            let frustum = Frustum::from_view_projection_matrix(view_proj);
            (Some(frustum), Some(view_proj))
        } else {
            (None, None)
        };

        // Prepare occlusion culler for new frame
        if occlusion_culling_enabled {
            self.occlusion_culler.prepare_frame();
        }

        // PASS 1: Add visible objects as potential occluders
        if occlusion_culling_enabled {
            for elem in persisted.scene.elements.iter() {
                if let Some(bounding_box) = &elem.bounding_box {
                    let world_aabb = bounding_box.transform(&Mat4::from(elem.transform.affine_transform()));
                    if let Some(ref view_proj) = view_proj_matrix {
                        self.occlusion_culler.add_occluder(world_aabb, view_proj);
                    }
                }
            }
        }

        // PASS 2: Test all objects for visibility
        for elem in persisted.scene.elements.iter_mut() {
            // Analyze GLTF files to extract nodes if not already done
            if elem.is_compound && elem.mesh_nodes.is_empty() {
                if let Err(e) = self.analyze_gltf_nodes(elem, ctx.world_renderer) {
                    println!("Warning: Failed to analyze GLTF nodes: {}", e);
                }
            }

            let mut element_is_visible = true;
            
            if frustum_culling_enabled || occlusion_culling_enabled {
                if elem.is_compound && !elem.mesh_nodes.is_empty() {
                    // For compound objects (GLTF with multiple nodes), test each node
                    let mut any_node_visible = false;
                    
                    for node in &elem.mesh_nodes {
                        total_sub_objects += 1;
                        let mut node_visible = true;
                        
                        if let Some(node_aabb) = &node.bounding_box {
                            // Transform node AABB to world space using both element and node transforms
                            let combined_transform = elem.transform.affine_transform() * node.local_transform.affine_transform();
                            let world_aabb = node_aabb.transform(&Mat4::from(combined_transform));
                            
                            // Test frustum culling first
                            if frustum_culling_enabled {
                                if let Some(ref frustum) = frustum {
                                    node_visible = if persisted.frustum_culling.use_sphere_culling {
                                        let sphere_center = world_aabb.center();
                                        let sphere_radius = world_aabb.half_size().length();
                                        frustum.is_visible_sphere(sphere_center, sphere_radius)
                                    } else {
                                        frustum.is_visible_aabb(&world_aabb)
                                    };
                                    
                                    if !node_visible {
                                        frustum_culled += 1;
                                    }
                                }
                            }
                            
                            // Test occlusion culling if still visible after frustum test
                            if node_visible && occlusion_culling_enabled {
                                if let Some(ref view_proj) = view_proj_matrix {
                                    if self.occlusion_culler.is_occluded(&world_aabb, view_proj) {
                                        node_visible = false;
                                        occlusion_culled += 1;
                                    }
                                }
                            }
                            
                            if node_visible {
                                any_node_visible = true;
                                visible_objects += 1;
                            }
                        } else {
                            // If no bounding box, assume visible
                            any_node_visible = true;
                            visible_objects += 1;
                        }
                    }
                    
                    element_is_visible = any_node_visible;
                } else {
                    // For simple objects, use the element's bounding box
                    total_sub_objects += 1;
                    
                    // Calculate world-space bounding box if not cached
                    if elem.bounding_box.is_none() {
                        let default_size = Vec3::splat(persisted.frustum_culling.default_object_size);
                        elem.bounding_box = Some(Aabb::from_center_size(Vec3::ZERO, default_size));
                    }

                    if let Some(local_aabb) = &elem.bounding_box {
                        let world_aabb = local_aabb.transform(&Mat4::from(elem.transform.affine_transform()));
                        
                        // Test frustum culling first
                        if frustum_culling_enabled {
                            if let Some(ref frustum) = frustum {
                                element_is_visible = if persisted.frustum_culling.use_sphere_culling {
                                    let world_center = elem.transform.position;
                                    let world_scale = elem.transform.scale.max_element();
                                    let sphere_radius = local_aabb.half_size().length() * world_scale;
                                    frustum.is_visible_sphere(world_center, sphere_radius)
                                } else {
                                    frustum.is_visible_aabb(&world_aabb)
                                };
                                
                                if !element_is_visible {
                                    frustum_culled += 1;
                                }
                            }
                        }
                        
                        // Test occlusion culling if still visible after frustum test
                        if element_is_visible && occlusion_culling_enabled {
                            if let Some(ref view_proj) = view_proj_matrix {
                                if self.occlusion_culler.is_occluded(&world_aabb, view_proj) {
                                    element_is_visible = false;
                                    occlusion_culled += 1;
                                }
                            }
                        }
                        
                        if element_is_visible {
                            visible_objects += 1;
                        }
                    }
                }
            } else {
                // Culling disabled - count all objects
                if elem.is_compound {
                    total_sub_objects += elem.mesh_nodes.len();
                    visible_objects += elem.mesh_nodes.len();
                } else {
                    total_sub_objects += 1;
                    visible_objects += 1;
                }
            }

            // Apply visibility results
            if element_is_visible {
                // Update instance parameters and transform only for visible objects
                ctx.world_renderer
                    .get_instance_dynamic_parameters_mut(elem.instance)
                    .emissive_multiplier = persisted.light.emissive_multiplier * emissive_toggle_mult;
                ctx.world_renderer
                    .set_instance_transform(elem.instance, elem.transform.affine_transform());
                
                // Perform triangle culling analysis for visible objects
                if triangle_culling_enabled {
                    self.analyze_triangle_culling(elem, &persisted.triangle_culling, view_proj_matrix.as_ref());
                }
            } else {
                // Apply culling based on the chosen method
                match persisted.frustum_culling.culling_method {
                    CullingMethod::EmissiveMultiplier => {
                        // Make objects invisible by setting emissive to 0
                        ctx.world_renderer
                            .get_instance_dynamic_parameters_mut(elem.instance)
                            .emissive_multiplier = 0.0;
                    }
                    CullingMethod::MoveAway => {
                        // Move objects far away (more effective for GPU culling)
                        ctx.world_renderer
                            .get_instance_dynamic_parameters_mut(elem.instance)
                            .emissive_multiplier = 0.0;
                        
                        let mut culled_transform = elem.transform.clone();
                        culled_transform.position = Vec3::new(1000000.0, 1000000.0, 1000000.0);
                        ctx.world_renderer
                            .set_instance_transform(elem.instance, culled_transform.affine_transform());
                    }
                    CullingMethod::ScaleToZero => {
                        // Scale objects to zero size (effective for GPU culling)
                        ctx.world_renderer
                            .get_instance_dynamic_parameters_mut(elem.instance)
                            .emissive_multiplier = 0.0;
                        
                        let mut culled_transform = elem.transform.clone();
                        culled_transform.scale = Vec3::ZERO;
                        ctx.world_renderer
                            .set_instance_transform(elem.instance, culled_transform.affine_transform());
                    }
                }
            }
        }

        // Optional: Log culling statistics
        if (frustum_culling_enabled || occlusion_culling_enabled) && persisted.frustum_culling.debug_logging {
            static mut FRAME_COUNTER: u32 = 0;
            unsafe {
                FRAME_COUNTER += 1;
                if FRAME_COUNTER % persisted.frustum_culling.log_interval_frames == 0 {
                    let mut log_msg = format!("Culling Stats: {}/{} sub-objects visible from {} elements", 
                        visible_objects, total_sub_objects, total_elements);
                    
                    if frustum_culling_enabled && occlusion_culling_enabled {
                        log_msg += &format!(" (Frustum: {} culled, Occlusion: {} culled)", frustum_culled, occlusion_culled);
                    } else if frustum_culling_enabled {
                        log_msg += &format!(" (Frustum culling only)");
                    } else if occlusion_culling_enabled {
                        log_msg += &format!(" (Occlusion culling only)");
                    }
                    
                    println!("{}", log_msg);
                    
                    // Show occlusion culling statistics
                    if occlusion_culling_enabled {
                        let stats = self.occlusion_culler.get_statistics();
                        println!("  Occlusion Stats: {} occluders, {:.1}% depth buffer usage", 
                            stats.total_occluders, stats.depth_buffer_usage);
                    }
                }
            }
        }
        
        // Update triangle culling frame counter and potentially log statistics
        if triangle_culling_enabled {
            self.triangle_culler.end_frame();
        }
    }

    pub fn frame(
        &mut self,
        mut ctx: FrameContext,
        persisted: &mut PersistedState,
    ) -> WorldFrameDesc {
        // Limit framerate. Not particularly precise.
        if self.max_fps != MAX_FPS_LIMIT {
            std::thread::sleep(std::time::Duration::from_micros(
                1_000_000 / self.max_fps as u64,
            ));
        }

        self.keyboard.update(ctx.events);
        self.mouse.update(ctx.events);
        self.handle_file_drop_events(persisted, ctx.world_renderer, ctx.events);

        let orig_persisted_state = persisted.clone();
        let orig_render_overrides = ctx.world_renderer.render_overrides;

        self.do_gui(persisted, &mut ctx);
        
        // Procesar inicialización pendiente del streaming
        if let Err(e) = futures::executor::block_on(
            self.streaming_integration.process_pending_initialization()
        ) {
            log::error!("Error procesando inicialización de streaming: {}", e);
        }
        
        self.update_lights(persisted, &mut ctx);
        self.update_objects(persisted, &mut ctx);
        self.update_sun(persisted, &mut ctx);

        // Update bounding boxes for new objects
        self.update_bounding_boxes(persisted, ctx.world_renderer);
        
        // Analyze GLTF files for compound objects
        let mut elements_to_analyze = Vec::new();
        
        for (index, elem) in persisted.scene.elements.iter().enumerate() {
            if !elem.is_compound {
                if let MeshSource::File(path) = &elem.source {
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");
                    
                    if extension == "gltf" || extension == "glb" {
                        elements_to_analyze.push(index);
                    }
                }
            }
        }
        
        for index in elements_to_analyze {
            if let Some(elem) = persisted.scene.elements.get_mut(index) {
                if let Err(e) = self.analyze_gltf_nodes(elem, ctx.world_renderer) {
                    if let MeshSource::File(path) = &elem.source {
                        println!("Warning: Failed to analyze GLTF nodes for {}: {}", path.display(), e);
                    }
                }
            }
        }

        self.update_camera(persisted, &ctx);

        if self
            .keyboard
            .was_just_pressed(self.keymap_config.sequencer.add_keyframe)
            || (self.mouse.buttons_pressed & (1 << 1)) != 0
        {
            self.add_sequence_keyframe(persisted);
        }

        if self
            .keyboard
            .was_just_pressed(self.keymap_config.sequencer.play)
        {
            match self.sequence_playback_state {
                SequencePlaybackState::NotPlaying => {
                    self.play_sequence(persisted);
                }
                SequencePlaybackState::Playing { .. } => {
                    self.stop_sequence();
                }
            };
        }

        ctx.world_renderer.ev_shift = persisted.exposure.ev_shift;
        ctx.world_renderer.contrast = persisted.exposure.contrast;
        ctx.world_renderer.dynamic_exposure.enabled = persisted.exposure.use_dynamic_adaptation;
        ctx.world_renderer.dynamic_exposure.speed_log2 =
            persisted.exposure.dynamic_adaptation_speed;
        ctx.world_renderer.dynamic_exposure.histogram_clipping.low =
            persisted.exposure.dynamic_adaptation_low_clip;
        ctx.world_renderer.dynamic_exposure.histogram_clipping.high =
            persisted.exposure.dynamic_adaptation_high_clip;

        if persisted.should_reset_path_tracer(&orig_persisted_state)
            || ctx.world_renderer.render_overrides != orig_render_overrides
        {
            self.reset_path_tracer = true;
        }

        // Reset accumulation of the path tracer whenever the camera moves
        if (self.reset_path_tracer
            || self
                .keyboard
                .was_just_pressed(self.keymap_config.rendering.reset_path_tracer))
            && ctx.world_renderer.render_mode == RenderMode::Reference
        {
            ctx.world_renderer.reset_reference_accumulation = true;
            self.reset_path_tracer = false;
        }

        let lens = CameraLens {
            aspect_ratio: ctx.aspect_ratio(),
            vertical_fov: persisted.camera.vertical_fov,
            ..Default::default()
        };

        WorldFrameDesc {
            camera_matrices: self
                .camera
                .final_transform
                .into_position_rotation()
                .through(&lens),
            render_extent: ctx.render_extent,
            sun_direction: self.sun_direction_interp,
        }
    }

    pub fn is_sequence_playing(&self) -> bool {
        matches!(
            &self.sequence_playback_state,
            SequencePlaybackState::Playing { .. }
        )
    }

    pub fn stop_sequence(&mut self) {
        self.sequence_playback_state = SequencePlaybackState::NotPlaying;
    }

    pub fn play_sequence(&mut self, persisted: &mut PersistedState) {
        // Allow some time at the start of the playback before the camera starts moving
        const PLAYBACK_WARMUP_DURATION: f32 = 0.5;

        let t = self
            .active_camera_key
            .and_then(|i| Some(persisted.sequence.get_item(i)?.t))
            .unwrap_or(-PLAYBACK_WARMUP_DURATION);

        self.sequence_playback_state = SequencePlaybackState::Playing {
            t,
            sequence: persisted.sequence.to_playback(),
        };
    }

    pub fn add_sequence_keyframe(&mut self, persisted: &mut PersistedState) {
        persisted.sequence.add_keyframe(
            self.active_camera_key,
            SequenceValue {
                camera_position: MemOption::new(persisted.camera.position),
                camera_direction: MemOption::new(persisted.camera.rotation * -Vec3::Z),
                towards_sun: MemOption::new(persisted.light.sun.controller.towards_sun()),
            },
        );

        if let Some(idx) = &mut self.active_camera_key {
            *idx += 1;
        }
    }

    pub fn jump_to_sequence_key(&mut self, persisted: &mut PersistedState, idx: usize) {
        let exact_item = if let Some(item) = persisted.sequence.get_item(idx) {
            item.clone()
        } else {
            return;
        };

        if let Some(value) = persisted.sequence.to_playback().sample(exact_item.t) {
            self.camera.driver_mut::<Position>().position = exact_item
                .value
                .camera_position
                .unwrap_or(value.camera_position);
            self.camera
                .driver_mut::<YawPitch>()
                .set_rotation_quat(dolly::util::look_at::<dolly::handedness::RightHanded>(
                    exact_item
                        .value
                        .camera_direction
                        .unwrap_or(value.camera_direction),
                ));

            self.camera.update(1e10);

            persisted
                .light
                .sun
                .controller
                .set_towards_sun(exact_item.value.towards_sun.unwrap_or(value.towards_sun));
        }

        self.active_camera_key = Some(idx);
        self.sequence_playback_state = SequencePlaybackState::NotPlaying;
    }

    pub fn replace_camera_sequence_key(&mut self, persisted: &mut PersistedState, idx: usize) {
        persisted.sequence.each_key(|i, item| {
            if idx != i {
                return;
            }

            item.value.camera_position = MemOption::new(persisted.camera.position);
            item.value.camera_direction = MemOption::new(persisted.camera.rotation * -Vec3::Z);
            item.value.towards_sun = MemOption::new(persisted.light.sun.controller.towards_sun());
        })
    }

    pub fn delete_camera_sequence_key(&mut self, persisted: &mut PersistedState, idx: usize) {
        persisted.sequence.delete_key(idx);

        self.active_camera_key = None;
    }

    pub(crate) fn load_mesh(
        &mut self,
        world_renderer: &mut WorldRenderer,
        source: &MeshSource,
    ) -> anyhow::Result<MeshHandle> {
        log::info!("Loading a mesh from {:?}", source);

        let path = match source {
            MeshSource::File(path) => {
                fn calculate_hash(t: &PathBuf) -> u64 {
                    let mut s = DefaultHasher::new();
                    t.hash(&mut s);
                    s.finish()
                }

                let path_hash = match path.canonicalize() {
                    Ok(canonical) => calculate_hash(&canonical),
                    Err(_) => calculate_hash(path),
                };

                let cached_mesh_name = format!("{:8.8x}", path_hash);
                let cached_mesh_path = PathBuf::from(format!("/cache/{}.mesh", cached_mesh_name));

                if !canonical_path_from_vfs(&cached_mesh_path).map_or(false, |path| path.exists()) {
                    kajiya_asset_pipe::process_mesh_asset(
                        kajiya_asset_pipe::MeshAssetProcessParams {
                            path: path.clone(),
                            output_name: cached_mesh_name,
                            scale: 1.0,
                        },
                    )?;
                }

                cached_mesh_path
            }
            MeshSource::Cache(path) => path.clone(),
        };

        Ok(*self.known_meshes.entry(path.clone()).or_insert_with(|| {
            world_renderer
                .add_baked_mesh(path, AddMeshOptions::new())
                .unwrap()
        }))
    }

    pub(crate) fn add_mesh_instance(
        &mut self,
        persisted: &mut PersistedState,
        world_renderer: &mut WorldRenderer,
        source: MeshSource,
        transform: SceneElementTransform,
    ) -> anyhow::Result<()> {
        let mesh = self.load_mesh(world_renderer, &source)?;
        let inst = world_renderer.add_instance(mesh, transform.affine_transform());

        persisted.scene.elements.push(SceneElement {
            source,
            instance: inst,
            transform,
            bounding_box: None, // Will be calculated later when mesh data is available
            mesh_nodes: Vec::new(),
            is_compound: false,
        });

        Ok(())
    }

    fn handle_file_drop_events(
        &mut self,
        persisted: &mut PersistedState,
        world_renderer: &mut WorldRenderer,
        events: &[winit::event::Event<()>],
    ) {
        for event in events {
            match event {
                winit::event::Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::DroppedFile(path),
                } => {
                    let extension = path
                        .extension()
                        .map_or("".to_string(), |ext| ext.to_string_lossy().into_owned());

                    match extension.as_str() {
                        "hdr" | "exr" => {
                            // IBL
                            match world_renderer.ibl.load_image(path) {
                                Ok(_) => {
                                    persisted.scene.ibl = Some(path.clone());
                                }
                                Err(err) => {
                                    log::error!("{:#}", err);
                                }
                            }
                        }
                        "ron" | "dmoon" => {
                            // Scene
                            if let Err(err) = self.load_scene(persisted, world_renderer, path) {
                                log::error!("Failed to load scene: {:#}", err);
                            }
                        }
                        "gltf" | "glb" => {
                            // Mesh
                            if let Err(err) = self.add_mesh_instance(
                                persisted,
                                world_renderer,
                                MeshSource::File(path.clone()),
                                SceneElementTransform::IDENTITY,
                            ) {
                                log::error!("{:#}", err);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// Calculate a more accurate bounding box for a mesh instance
    pub fn calculate_mesh_bounding_box(
        &self,
        _world_renderer: &WorldRenderer, // Prefixed with _ to suppress unused warning
        mesh_handle: MeshHandle,
    ) -> Option<Aabb> {
        // In a real implementation, you would:
        // 1. Get the mesh data from world_renderer
        // 2. Calculate the actual AABB from vertex positions
        // 3. Cache the result
        
        // For now, return a default based on mesh handle ID
        let handle_id = mesh_handle.0; // Assuming MeshHandle has a numeric ID
        let base_size = 1.0 + (handle_id % 5) as f32; // Vary size based on mesh ID
        
        Some(Aabb::from_center_size(
            Vec3::ZERO,
            Vec3::splat(base_size)
        ))
    }

    /// Update bounding boxes for all scene elements that don't have them
    pub fn update_bounding_boxes(
        &self,
        persisted: &mut PersistedState,
        _world_renderer: &WorldRenderer, // Prefixed with _ to suppress unused warning
    ) {
        for elem in persisted.scene.elements.iter_mut() {
            if elem.bounding_box.is_none() {
                // Try to get the mesh handle from the instance
                // This is a simplified version - in practice you'd need to access the mesh data
                if let Some(aabb) = self.calculate_mesh_bounding_box(_world_renderer, MeshHandle(0)) {
                    elem.bounding_box = Some(aabb);
                }
            }
        }
    }

    /// Analyze a GLTF file and extract individual mesh nodes for better culling
    pub fn analyze_gltf_nodes(
        &self,
        elem: &mut SceneElement,
        _world_renderer: &WorldRenderer, // Prefixed with _ to suppress unused warning
    ) -> anyhow::Result<()> {
        if let MeshSource::File(path) = &elem.source {
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            // Handle direct GLTF files
            if extension == "gltf" || extension == "glb" {
                let gltf_result = self.load_and_analyze_gltf(path);
                
                match gltf_result {
                    Ok(nodes) => {
                        elem.mesh_nodes = nodes;
                        elem.is_compound = elem.mesh_nodes.len() > 1;
                        
                        println!("Analyzed GLTF '{}': Found {} mesh nodes", 
                            path.display(), 
                            elem.mesh_nodes.len()
                        );
                    }
                    Err(e) => {
                        println!("Warning: Failed to parse GLTF '{}': {}. Using fallback.", path.display(), e);
                        
                        // Fallback to mock data if parsing fails
                        elem.mesh_nodes = vec![
                            MeshNode {
                                name: Some("Fallback_Node".to_string()),
                                local_transform: SceneElementTransform::IDENTITY,
                                bounding_box: Some(Aabb::from_center_size(Vec3::ZERO, Vec3::splat(1.0))),
                            },
                        ];
                        elem.is_compound = false;
                    }
                }
            }
            // Handle .dmoon files that might reference GLTF files
            else if extension == "dmoon" {
                // For .dmoon files, we need to look at the mesh reference within the file
                // This is a simplified approach - in a real implementation you'd parse the .dmoon file
                // For now, we'll check if this element has a mesh reference that points to a GLTF file
                
                // Try to extract the GLTF path from the dmoon context
                if let Some(gltf_path) = self.extract_gltf_path_from_dmoon(path) {
                    println!("Found GLTF reference in dmoon file: {}", gltf_path.display());
                    
                    let gltf_result = self.load_and_analyze_gltf(&gltf_path);
                    
                    match gltf_result {
                        Ok(nodes) => {
                            elem.mesh_nodes = nodes;
                            elem.is_compound = elem.mesh_nodes.len() > 1;
                            
                            println!("Analyzed referenced GLTF from dmoon '{}': Found {} mesh nodes", 
                                gltf_path.display(), 
                                elem.mesh_nodes.len()
                            );
                        }
                        Err(e) => {
                            println!("Warning: Failed to parse referenced GLTF '{}': {}. Using fallback.", gltf_path.display(), e);
                            elem.mesh_nodes = vec![
                                MeshNode {
                                    name: Some("Fallback_Dmoon_Node".to_string()),
                                    local_transform: SceneElementTransform::IDENTITY,
                                    bounding_box: Some(Aabb::from_center_size(Vec3::ZERO, Vec3::splat(2.0))),
                                },
                            ];
                            elem.is_compound = false;
                        }
                    }
                } else {
                    println!("No GLTF reference found in dmoon file: {}", path.display());
                }
            }
        }
        
        Ok(())
    }

    /// Extract the GLTF path referenced by a dmoon file
    fn extract_gltf_path_from_dmoon(&self, dmoon_path: &std::path::Path) -> Option<std::path::PathBuf> {
        use std::fs;
        
        // Try to read and parse the dmoon file
        if let Ok(content) = fs::read_to_string(dmoon_path) {
            // Look for mesh references in the dmoon content
            // This is a simple approach - looking for .gltf or .glb file references
            for line in content.lines() {
                if line.contains("mesh:") && (line.contains(".gltf") || line.contains(".glb")) {
                    // Extract the path between quotes
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line.rfind('"') {
                            if start < end {
                                let mesh_path = &line[start+1..end];
                                
                                // Remove leading slash if present and construct full path
                                let mesh_path = mesh_path.trim_start_matches('/');
                                let full_path = std::path::Path::new("assets").join(mesh_path);
                                
                                println!("Extracted GLTF path from dmoon: {}", full_path.display());
                                return Some(full_path);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Load and analyze a GLTF file to extract mesh nodes
    fn load_and_analyze_gltf(&self, path: &std::path::Path) -> anyhow::Result<Vec<MeshNode>> {
        use std::fs::File;
        use std::io::BufReader;
        
        // Resolve the full path (GLTF files are typically in assets/)
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::path::Path::new("assets").join(path)
        };

        println!("Attempting to load GLTF from: {}", full_path.display());

        // Try to load the GLTF file
        let file = File::open(&full_path)
            .with_context(|| format!("Failed to open GLTF file: {}", full_path.display()))?;
        
        let reader = BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader)
            .with_context(|| format!("Failed to parse GLTF file: {}", full_path.display()))?;

        let mut mesh_nodes = Vec::new();

        // Print basic GLTF info
        println!("GLTF file loaded successfully:");
        println!("  - Scenes: {}", gltf.scenes().count());
        println!("  - Nodes: {}", gltf.nodes().count());
        println!("  - Meshes: {}", gltf.meshes().count());
        
        // Iterate through all scenes in the GLTF
        for (scene_idx, scene) in gltf.scenes().enumerate() {
            println!("Processing scene {}: {:?}", scene_idx, scene.name().unwrap_or("unnamed"));
            
            // Process each root node in the scene
            for node in scene.nodes() {
                self.process_gltf_node(&node, Mat4::IDENTITY, &mut mesh_nodes)?;
            }
        }

        if mesh_nodes.is_empty() {
            return Err(anyhow::anyhow!("No mesh nodes found in GLTF file"));
        }

        println!("Successfully extracted {} mesh nodes from GLTF", mesh_nodes.len());
        for (idx, node) in mesh_nodes.iter().enumerate() {
            println!("  Node {}: '{}' at {:?}", 
                idx, 
                node.name.as_deref().unwrap_or("unnamed"), 
                node.local_transform.position
            );
        }
        
        Ok(mesh_nodes)
    }

    /// Recursively process GLTF nodes and extract mesh information
    fn process_gltf_node(
        &self, 
        node: &gltf::Node, 
        parent_transform: Mat4,
        mesh_nodes: &mut Vec<MeshNode>
    ) -> anyhow::Result<()> {
        let node_name = node.name().unwrap_or("unnamed");
        println!("Processing node: '{}'", node_name);
        
        // Get node transform
        let node_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
        let combined_transform = parent_transform * node_transform;

        // If this node has a mesh, create a MeshNode
        if let Some(mesh) = node.mesh() {
            // Extract position, rotation, and scale from the transform matrix
            let (scale, rotation, translation) = combined_transform.to_scale_rotation_translation();
            
            // Convert rotation quaternion to Euler angles
            let (x, y, z) = rotation.to_euler(dolly::glam::EulerRot::YXZ);
            let rotation_degrees = Vec3::new(
                x.to_degrees(),
                y.to_degrees(), 
                z.to_degrees()
            );

            // Create bounding box based on mesh (for now, use a reasonable default)
            let max_scale = scale.max_element();
            let bounding_size = Vec3::splat(max_scale * 2.0); // Reasonable default based on scale
            
            let mesh_node = MeshNode {
                name: Some(node_name.to_string()),
                local_transform: SceneElementTransform {
                    position: translation,
                    rotation_euler_degrees: rotation_degrees,
                    scale,
                },
                bounding_box: Some(Aabb::from_center_size(translation, bounding_size)),
            };

            mesh_nodes.push(mesh_node);
            
            println!("  -> Found mesh node: '{}' at position {:?} (primitives: {})", 
                node_name, 
                translation,
                mesh.primitives().count()
            );
        } else {
            println!("  -> Node '{}' has no mesh, checking children", node_name);
        }

        // Recursively process child nodes
        let child_count = node.children().count();
        if child_count > 0 {
            println!("  -> Processing {} children of '{}'", child_count, node_name);
            for child in node.children() {
                self.process_gltf_node(&child, combined_transform, mesh_nodes)?;
            }
        }

        Ok(())
    }

    /// Analyze triangle culling for a given scene element
    fn analyze_triangle_culling(
        &mut self,
        elem: &SceneElement,
        _config: &crate::math::triangle_culling::TriangleCullingConfig,
        view_proj_matrix: Option<&Mat4>,
    ) {
        // For now, we'll generate some example triangles for demonstration
        // In a real implementation, you would extract actual triangles from the mesh data
        let example_triangles = self.generate_example_triangles_for_element(elem);
        
        for triangle in example_triangles {
            self.triangle_culler.test_triangle(&triangle, view_proj_matrix);
        }
    }
    
    /// Generate example triangles for demonstration purposes
    /// In a real implementation, this would extract actual triangles from mesh data
    fn generate_example_triangles_for_element(&self, elem: &SceneElement) -> Vec<crate::math::Triangle> {
        let mut triangles = Vec::new();
        
        // Transform to world space using element transform
        let transform = Mat4::from(elem.transform.affine_transform());
        
        if elem.is_compound {
            // For compound objects, generate triangles for each mesh node
            for node in &elem.mesh_nodes {
                if let Some(aabb) = &node.bounding_box {
                    let combined_transform = transform * Mat4::from(node.local_transform.affine_transform());
                    triangles.extend(self.triangles_from_aabb(aabb, &combined_transform));
                }
            }
        } else {
            // For simple objects, generate triangles from the element's bounding box
            if let Some(aabb) = &elem.bounding_box {
                triangles.extend(self.triangles_from_aabb(aabb, &transform));
            }
        }
        
        triangles
    }
    
    /// Generate example triangles from an AABB (for demonstration)
    /// In a real implementation, this would use actual mesh geometry
    fn triangles_from_aabb(&self, aabb: &crate::math::Aabb, transform: &Mat4) -> Vec<crate::math::Triangle> {
        let min_point = aabb.min;
        let max_point = aabb.max;
        
        // Create two triangles for one face of the AABB as an example
        let v0 = transform.transform_point3(Vec3::new(min_point.x, min_point.y, min_point.z));
        let v1 = transform.transform_point3(Vec3::new(max_point.x, min_point.y, min_point.z));
        let v2 = transform.transform_point3(Vec3::new(max_point.x, max_point.y, min_point.z));
        let v3 = transform.transform_point3(Vec3::new(min_point.x, max_point.y, min_point.z));
        
        vec![
            crate::math::Triangle::new([v0, v1, v2]),
            crate::math::Triangle::new([v0, v2, v3]),
        ]
    }

    /// Get triangle culling statistics
    pub fn get_triangle_culling_statistics(&self) -> &crate::math::triangle_culling::TriangleCullingStats {
        self.triangle_culler.get_statistics()
    }

    //...existing code...
}

#[derive(PartialEq, Eq)]
pub enum LeftClickEditMode {
    MoveSun,
    //MoveLocalLights,
}
