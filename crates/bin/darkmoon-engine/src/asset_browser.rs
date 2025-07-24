use std::fs;
use std::path::{Path, PathBuf};
use imgui::{Ui, im_str, ImString};

pub struct AssetBrowser {
    pub open: bool,
    pub current_dir: PathBuf,
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            open: true,
            current_dir: PathBuf::from("assets"),
        }
    }

    pub fn show(&mut self, ui: &Ui) {
        if !self.open {
            return;
        }
        let current_dir = self.current_dir.clone();
        imgui::Window::new(im_str!("Assets Browser"))
            .opened(&mut self.open)
            .resizable(true)
            .size([400.0, 500.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                Self::show_dir_static(ui, &current_dir);
            });
    }

    fn show_dir_static(ui: &Ui, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name = ImString::from(file_name.to_string_lossy().to_string());
                if path.is_dir() {
                    let tree = imgui::TreeNode::new(&file_name).build(ui, || {
                        Self::show_dir_static(ui, &path);
                    });
                    let _ = tree;
                } else {
                    ui.bullet_text(&file_name);
                }
            }
        } else {
            ui.text(im_str!("No se pudo leer la carpeta de assets."));
        }
    }
}
