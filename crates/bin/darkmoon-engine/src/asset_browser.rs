use std::fs;
use std::path::{Path, PathBuf};
use imgui::{Ui, ImString};

use darkmoon_icons::*;

pub struct AssetBrowser {
    pub open: bool,
    pub current_dir: PathBuf,
}

#[derive(Clone)]
pub enum AssetAction {
    None,
    LoadScene(PathBuf),
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
        
        ui.window("Assets Browser")
            .opened(&mut self.open)
            .resizable(true)
            .size([400.0, 500.0], imgui::Condition::FirstUseEver)
            .build(|| {
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
                    let folder_label = ImString::from(get_folder_icon_label(file_name.to_str(), false));
                    ui.tree_node_config(&folder_label)
                        .default_open(false)
                        .build(|| {
                            Self::show_dir_recursive(ui, &path, action);
                        });
                } else {
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");
                    
                    match extension {
                        "dmoon" => {
                            let scene_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                            if ui.selectable(&scene_label) {
                                *action = AssetAction::LoadScene(path.clone());
                            }
                        }
                        "gltf" | "glb" => {
                            let model_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                            ui.bullet_text(&model_label);
                        }
                        "png" | "jpg" | "jpeg" | "tga" | "dds" | "hdr" | "exr" => {
                            let image_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                            ui.bullet_text(&image_label);
                        }
                        "hlsl" | "glsl" | "wgsl" => {
                            let shader_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                            ui.bullet_text(&shader_label);
                        }
                        "wav" | "mp3" | "ogg" => {
                            let audio_label = ImString::from(get_file_icon_label(extension, file_name.to_str()));
                            ui.bullet_text(&audio_label);
                        }
                        _ => {
                            ui.bullet_text(&file_name);
                        }
                    }
                }
            }
        } else {
            ui.text("No se pudo leer la carpeta de assets.");
        }
    }
}
