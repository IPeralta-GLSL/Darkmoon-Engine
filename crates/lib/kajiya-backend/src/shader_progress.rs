use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ShaderCompilationProgress {
    pub total_shaders: usize,
    pub completed_shaders: usize,
    pub current_shader: Option<String>,
    pub is_complete: bool,
    pub failed_shaders: Vec<String>,
    pub is_simulation_mode: bool,
}

impl ShaderCompilationProgress {
    pub fn new() -> Self {
        Self {
            total_shaders: 0,
            completed_shaders: 0,
            current_shader: None,
            is_complete: false,
            failed_shaders: Vec::new(),
            is_simulation_mode: false,
        }
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.total_shaders == 0 {
            100.0
        } else {
            (self.completed_shaders as f32 / self.total_shaders as f32) * 100.0
        }
    }

    pub fn status_text(&self) -> String {
        if self.is_complete {
            let status = if self.is_simulation_mode { 
                "Simulation complete! Real shader compilation may continue..."
            } else {
                "Shader compilation complete!"
            };
            format!("{} ({}/{})", status, self.completed_shaders, self.total_shaders)
        } else if let Some(current) = &self.current_shader {
            format!("Compiling: {} ({}/{})", current, self.completed_shaders, self.total_shaders)
        } else {
            format!("Preparing shader compilation... ({}/{})", self.completed_shaders, self.total_shaders)
        }
    }
}

pub struct ShaderProgressTracker {
    progress: Arc<Mutex<ShaderCompilationProgress>>,
    shader_states: HashMap<String, bool>, 
    pipeline_compilation_active: bool,
    
    frames_since_last_compilation: u32,
    pipeline_compilation_cooldown_frames: u32,
    total_pipelines_compiled_this_session: u32,
}

impl ShaderProgressTracker {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(ShaderCompilationProgress::new())),
            shader_states: HashMap::new(),
            pipeline_compilation_active: false,
            frames_since_last_compilation: 0,
            pipeline_compilation_cooldown_frames: 60, 
            total_pipelines_compiled_this_session: 0,
        }
    }

    pub fn get_progress(&self) -> Arc<Mutex<ShaderCompilationProgress>> {
        self.progress.clone()
    }

    pub fn register_shader(&mut self, shader_path: &str) {
        self.shader_states.insert(shader_path.to_string(), false);
        if let Ok(mut progress) = self.progress.lock() {
            progress.total_shaders = self.shader_states.len();
        }
    }

    pub fn start_compiling_shader(&mut self, shader_path: &str) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.current_shader = Some(shader_path.to_string());
        }
    }

    pub fn finish_compiling_shader(&mut self, shader_path: &str, success: bool) {
        if let Some(compiled) = self.shader_states.get_mut(shader_path) {
            *compiled = success;
        }

        if let Ok(mut progress) = self.progress.lock() {
            if success {
                progress.completed_shaders += 1;
            } else {
                progress.failed_shaders.push(shader_path.to_string());
            }

            progress.current_shader = None;
            let all_processed = progress.completed_shaders + progress.failed_shaders.len() >= progress.total_shaders;

            progress.is_complete = all_processed && !self.pipeline_compilation_active;
        }
    }

    pub fn set_pipeline_compilation_active(&mut self, active: bool) {
        log::debug!("Setting pipeline compilation active: {}", active);
        self.pipeline_compilation_active = active;

        if active {
            self.frames_since_last_compilation = 0;
        }
        
        if let Ok(mut progress) = self.progress.lock() {
            let all_processed = progress.completed_shaders + progress.failed_shaders.len() >= progress.total_shaders;
            progress.is_complete = all_processed && !active;

            if active && progress.is_simulation_mode {
                log::info!("Pipeline compilation starting, disabling simulation mode");
                progress.is_simulation_mode = false;
            }
        }
    }

    pub fn update_frame(&mut self, pipelines_compiled_this_frame: u32) {
        if pipelines_compiled_this_frame > 0 {
            
            self.frames_since_last_compilation = 0;
            self.total_pipelines_compiled_this_session += pipelines_compiled_this_frame;
            log::debug!("Compiled {} pipelines this frame (total: {})", 
                pipelines_compiled_this_frame, self.total_pipelines_compiled_this_session);
        } else if self.pipeline_compilation_active {
            
            self.frames_since_last_compilation += 1;
        }

        if self.pipeline_compilation_active && 
           self.frames_since_last_compilation >= self.pipeline_compilation_cooldown_frames {
            log::info!("Pipeline compilation cooldown complete after {} frames with no activity. Total pipelines compiled: {}",
                self.frames_since_last_compilation, self.total_pipelines_compiled_this_session);
            self.pipeline_compilation_active = false;

            if let Ok(mut progress) = self.progress.lock() {
                let all_processed = progress.completed_shaders + progress.failed_shaders.len() >= progress.total_shaders;
                progress.is_complete = all_processed;
            }
        }
    }

    pub fn should_force_finish_compilation(&self) -> bool {
        
        self.total_pipelines_compiled_this_session > 50 && 
        self.frames_since_last_compilation > 30
    }

    pub fn set_simulation_mode(&mut self, is_simulation: bool) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.is_simulation_mode = is_simulation;
        }
    }

    pub fn reset_for_real_compilation(&mut self) {
        log::info!("Resetting shader progress tracker for real compilation");
        self.shader_states.clear();
        self.pipeline_compilation_active = true;
        self.frames_since_last_compilation = 0;
        self.total_pipelines_compiled_this_session = 0;
        if let Ok(mut progress) = self.progress.lock() {
            progress.total_shaders = 0;
            progress.completed_shaders = 0;
            progress.current_shader = None;
            progress.is_complete = false;
            progress.failed_shaders.clear();
            progress.is_simulation_mode = false;
        }
    }

    pub fn is_compilation_complete(&self) -> bool {
        if let Ok(progress) = self.progress.lock() {
            progress.is_complete
        } else {
            false
        }
    }

    pub fn is_pipeline_compilation_active(&self) -> bool {
        self.pipeline_compilation_active
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_SHADER_PROGRESS: Arc<Mutex<ShaderProgressTracker>> = 
        Arc::new(Mutex::new(ShaderProgressTracker::new()));
}

pub fn start_real_compilation() {
    if let Ok(mut tracker) = GLOBAL_SHADER_PROGRESS.lock() {
        tracker.reset_for_real_compilation();
    }
}

pub fn update_pipeline_compilation_frame(pipelines_compiled_this_frame: u32) {
    if let Ok(mut tracker) = GLOBAL_SHADER_PROGRESS.lock() {
        tracker.update_frame(pipelines_compiled_this_frame);
    }
}

pub fn is_compilation_active() -> bool {
    if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
        if let Ok(progress) = tracker.get_progress().lock() {
            return progress.total_shaders > 0 && !progress.is_complete;
        }
    }
    false
}

pub fn is_compilation_or_heavy_work_active() -> bool {
    
    if is_compilation_active() {
        return true;
    }

    if let Ok(tracker) = GLOBAL_SHADER_PROGRESS.lock() {
        if let Ok(progress) = tracker.get_progress().lock() {
            
            if progress.total_shaders > 0 && tracker.is_pipeline_compilation_active() {
                return true;
            }
        }
    }
    
    false
}
