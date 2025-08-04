// Ejemplo modernizado del asset_browser.rs usando IconFontCppHeaders
use std::fs;
use std::path::{Path, PathBuf};
use imgui::{Ui, im_str, ImString};

// Importar el módulo de iconos (cuando esté disponible)
// use darkmoon_engine::icons::{get_file_icon_label, get_folder_icon_label};

pub struct AssetBrowser {
    pub open: bool,
    pub current_dir: PathBuf,
}

#[derive(Clone)]
pub enum AssetAction {
    None,
    LoadScene(PathBuf),
    LoadModel(PathBuf),
    LoadTexture(PathBuf),
    LoadShader(PathBuf),
    LoadAudio(PathBuf),
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            open: true,
            current_dir: PathBuf::from("assets"),
        }
    }

    pub fn show(&mut self, ui: &Ui) -> AssetAction {
        if !self.open {
            return AssetAction::None;
        }
        let current_dir = self.current_dir.clone();
        let mut action = AssetAction::None;
        
        imgui::Window::new(im_str!("Assets Browser"))
            .opened(&mut self.open)
            .resizable(true)
            .size([400.0, 500.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                Self::show_dir_recursive(ui, &current_dir, &mut action);
            });
        
        action
    }

    fn show_dir_recursive(ui: &Ui, dir: &Path, action: &mut AssetAction) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name = ImString::from(file_name.to_string_lossy().to_string());
                
                if path.is_dir() {
                    // Usar icono de carpeta profesional
                    let folder_label = ImString::from(format!("📁 {}", file_name.to_str()));
                    // Con iconos sería: 
                    // let folder_label = ImString::from(get_folder_icon_label(file_name.to_str(), false));
                    
                    imgui::TreeNode::new(&folder_label)
                        .default_open(false)
                        .build(ui, || {
                            Self::show_dir_recursive(ui, &path, action);
                        });
                } else {
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");
                    
                    Self::show_file_item(ui, &path, &file_name, extension, action);
                }
            }
        } else {
            ui.text(im_str!("No se pudo leer la carpeta de assets."));
        }
    }

    fn show_file_item(ui: &Ui, path: &Path, file_name: &ImString, extension: &str, action: &mut AssetAction) {
        match extension {
            "dmoon" => {
                // Escenas - Seleccionables para cargar
                let scene_label = ImString::from(format!("🎬 {}", file_name.to_str()));
                // Con iconos sería: let scene_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                
                if imgui::Selectable::new(&scene_label).build(ui) {
                    *action = AssetAction::LoadScene(path.to_path_buf());
                }
            }
            "gltf" | "glb" | "obj" | "fbx" => {
                // Modelos 3D - Seleccionables para importar
                let model_label = ImString::from(format!("🗿 {}", file_name.to_str()));
                // Con iconos sería: let model_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                
                if imgui::Selectable::new(&model_label).build(ui) {
                    *action = AssetAction::LoadModel(path.to_path_buf());
                }
            }
            "png" | "jpg" | "jpeg" | "tga" | "dds" | "hdr" | "exr" | "bmp" => {
                // Texturas - Con preview al hacer hover (futuro)
                let image_label = ImString::from(format!("🖼️ {}", file_name.to_str()));
                // Con iconos sería: let image_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                
                if imgui::Selectable::new(&image_label).build(ui) {
                    *action = AssetAction::LoadTexture(path.to_path_buf());
                }
                
                // Mostrar información adicional (tamaño, formato, etc.)
                if ui.is_item_hovered() {
                    ui.tooltip_text(format!("Textura: {}", path.display()));
                }
            }
            "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "comp" => {
                // Shaders - Con indicación del tipo
                let shader_type = Self::get_shader_type_suffix(extension);
                let shader_label = ImString::from(format!("⚡ {} {}", file_name.to_str(), shader_type));
                // Con iconos sería: let shader_label = ImString::from(get_file_icon_label(extension, &format!("{} {}", file_name.to_str(), shader_type)));
                
                if imgui::Selectable::new(&shader_label).build(ui) {
                    *action = AssetAction::LoadShader(path.to_path_buf());
                }
            }
            "wav" | "mp3" | "ogg" | "flac" => {
                // Audio - Con controles de reproducción (futuro)
                let audio_label = ImString::from(format!("🔊 {}", file_name.to_str()));
                // Con iconos sería: let audio_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                
                if imgui::Selectable::new(&audio_label).build(ui) {
                    *action = AssetAction::LoadAudio(path.to_path_buf());
                }
            }
            "rs" | "cpp" | "c" | "h" | "hpp" => {
                // Código fuente - Solo mostrar, no cargar
                let code_label = ImString::from(format!("📄 {}", file_name.to_str()));
                ui.bullet_text(&code_label);
            }
            "toml" | "yaml" | "json" | "xml" => {
                // Archivos de configuración
                let config_label = ImString::from(format!("⚙️ {}", file_name.to_str()));
                ui.bullet_text(&config_label);
            }
            _ => {
                // Archivos genéricos
                ui.bullet_text(&file_name);
            }
        }
    }

    fn get_shader_type_suffix(extension: &str) -> &'static str {
        match extension {
            "vert" => "(Vertex)",
            "frag" => "(Fragment)",
            "geom" => "(Geometry)",
            "comp" => "(Compute)",
            "tesc" => "(Tess Control)",
            "tese" => "(Tess Eval)",
            _ => "",
        }
    }
}
