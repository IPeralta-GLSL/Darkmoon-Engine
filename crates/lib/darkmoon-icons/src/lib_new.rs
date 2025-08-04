//! Darkmoon Icons - Professional icon system for Darkmoon Engine
//! 
//! Based on official IconFontCppHeaders project by Juliette Foucaut
//! https://github.com/juliettef/IconFontCppHeaders

// Re-export Font Awesome 6 icons from official IconFontCppHeaders Rust files
pub mod font_awesome;
pub mod font_awesome_brands;
pub mod font_setup;

pub use font_awesome::*;
pub use font_awesome_brands::*;

// ========== DARKMOON ENGINE ICON UTILITIES ==========

/// Obtiene el icono apropiado para un tipo de archivo basado en su extensión
pub fn get_file_icon(extension: &str) -> char {
    match extension.to_lowercase().as_str() {
        // Escenas del motor
        "dmoon" => ICON_FILM,
        
        // Modelos 3D
        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => ICON_CUBE,
        
        // Texturas e imágenes
        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => ICON_IMAGE,
        
        // Shaders
        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => ICON_BOLT,
        
        // Audio
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => ICON_VOLUME_HIGH,
        
        // Código fuente
        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => ICON_CODE,
        
        // Configuración
        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => ICON_GEAR,
        
        // Archivos genéricos
        _ => ICON_FILE,
    }
}

/// Crea un label con icono para mostrar en la UI
pub fn create_icon_label(icon: char, text: &str) -> String {
    format!("{} {}", icon, text)
}

/// Obtiene un label con icono para un archivo
pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

/// Obtiene un label con icono para una carpeta
pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { ICON_FOLDER_OPEN } else { ICON_FOLDER };
    create_icon_label(icon, foldername)
}
