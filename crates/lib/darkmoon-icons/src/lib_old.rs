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

// ========== DARKMOON ENGINE ICON MAPPINGS ==========
// Mapeo de iconos específicos del motor usando constantes de IconFontCppHeaders

/// Obtiene el icono apropiado para un tipo de archivo basado en su extensión
pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        // Escenas del motor
        "dmoon" => &ICON_FILM.to_string(),
        
        // Modelos 3D
        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => &ICON_CUBE.to_string(),
        
        // Texturas e imágenes
        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => &ICON_IMAGE.to_string(),
        
        // Shaders
        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => &ICON_BOLT.to_string(),
        
        // Audio
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => &ICON_VOLUME_HIGH.to_string(),
        
        // Código fuente
        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => &ICON_CODE.to_string(),
        
        // Configuración
        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => &ICON_GEAR.to_string(),
        
        // Archivos genéricos
        _ => &ICON_FILE.to_string(),
    }
}

/// Crea un label con icono para mostrar en la UI
pub fn create_icon_label(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

/// Obtiene un label con icono para un archivo
pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

/// Obtiene un label con icono para una carpeta
pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { &ICON_FOLDER_OPEN.to_string() } else { &ICON_FOLDER.to_string() };
    create_icon_label(icon, foldername)
}
pub const ICON_FA_MOON: &str = "\u{f186}";       // Night/Dark mode
pub const ICON_FA_FIRE: &str = "\u{f06d}";       // Particles/Fire
pub const ICON_FA_WATER: &str = "\u{f773}";      // Water effects
pub const ICON_FA_CLOUD: &str = "\u{f0c2}";      // Sky/Atmosphere

// Navegación y UI
pub const ICON_FA_HOME: &str = "\u{f015}";       // Home
pub const ICON_FA_SEARCH: &str = "\u{f002}";     // Search
pub const ICON_FA_PLUS: &str = "\u{f067}";       // Add
pub const ICON_FA_MINUS: &str = "\u{f068}";      // Remove
pub const ICON_FA_TIMES: &str = "\u{f00d}";      // Close/Delete
pub const ICON_FA_CHECK: &str = "\u{f00c}";      // Confirm/OK

// Almacenamiento y proyectos
pub const ICON_FA_SAVE: &str = "\u{f0c7}";       // Save
pub const ICON_FA_FOLDER_PLUS: &str = "\u{f65e}"; // New folder
pub const ICON_FA_DOWNLOAD: &str = "\u{f019}";   // Import
pub const ICON_FA_UPLOAD: &str = "\u{f093}";     // Export
pub const ICON_FA_DATABASE: &str = "\u{f1c0}";   // Data/Cache

// Sistemas y performance
pub const ICON_FA_TACHOMETER: &str = "\u{f3f4}"; // Performance/FPS
pub const ICON_FA_MEMORY: &str = "\u{f538}";     // Memory
pub const ICON_FA_MICROCHIP: &str = "\u{f2db}";  // CPU/Processing
pub const ICON_FA_CHART_BAR: &str = "\u{f080}";  // Stats/Profiling

// ========== CONFIGURACIÓN DE FUENTES ==========

// Nombres de archivos de fuentes Font Awesome
pub const FONT_ICON_FILE_NAME_FAS: &str = "fa-solid-900.otf";      // Solid icons
pub const FONT_ICON_FILE_NAME_FAB: &str = "fa-brands-400.otf";     // Brand icons
pub const FONT_ICON_FILE_NAME_FAR: &str = "fa-regular-400.otf";    // Regular icons (Pro only)

// ========== UTILIDADES ==========

/// Obtiene el icono apropiado para un tipo de archivo basado en su extensión
pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        // Escenas del motor
        "dmoon" => ICON_FA_SCENE,
        
        // Modelos 3D
        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => ICON_FA_MODEL,
        
        // Texturas e imágenes
        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => ICON_FA_TEXTURE,
        
        // Shaders
        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => ICON_FA_SHADER,
        
        // Audio
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => ICON_FA_AUDIO,
        
        // Código fuente
        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => ICON_FA_CODE,
        
        // Configuración
        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => ICON_FA_COGS,
        
        // Archivos genéricos
        _ => ICON_FA_FILE,
    }
}

/// Crea un label con icono para mostrar en la UI
pub fn create_icon_label(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

/// Obtiene un label con icono para un archivo
pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

/// Obtiene un label con icono para una carpeta
pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { ICON_FA_FOLDER_OPEN } else { ICON_FA_FOLDER };
    create_icon_label(icon, foldername)
}
