pub mod font_setup;

// Font Awesome icons constants for Rust
// Basado en IconFontCppHeaders - Font Awesome 6
// https://github.com/juliettef/IconFontCppHeaders

pub const ICON_MIN_FA: u16 = 0xe000;
pub const ICON_MAX_16_FA: u16 = 0xf8ff;
pub const ICON_MAX_FA: u32 = 0x10ffff;

// ========== ICONOS PRINCIPALES PARA DARKMOON ENGINE ==========

// Archivos y carpetas
pub const ICON_FA_FILE: &str = "\u{f15b}";
pub const ICON_FA_FOLDER: &str = "\u{f07b}";
pub const ICON_FA_FOLDER_OPEN: &str = "\u{f07c}";

// Assets específicos del motor
pub const ICON_FA_SCENE: &str = "\u{f008}";      // Film - Escenas .dmoon
pub const ICON_FA_MODEL: &str = "\u{f1b2}";      // Cube - Modelos 3D (.gltf, .glb)
pub const ICON_FA_TEXTURE: &str = "\u{f03e}";    // Image - Texturas e imágenes
pub const ICON_FA_SHADER: &str = "\u{f0e7}";     // Bolt - Shaders (.hlsl, .glsl, .wgsl)
pub const ICON_FA_AUDIO: &str = "\u{f028}";      // Volume - Audio (.wav, .mp3, .ogg)
pub const ICON_FA_MESH: &str = "\u{f5fd}";       // Shapes - Meshes

// Controles de interfaz
pub const ICON_FA_PLAY: &str = "\u{f04b}";       // Play
pub const ICON_FA_PAUSE: &str = "\u{f04c}";      // Pause
pub const ICON_FA_STOP: &str = "\u{f04d}";       // Stop
pub const ICON_FA_COGS: &str = "\u{f085}";       // Settings/Config
pub const ICON_FA_EYE: &str = "\u{f06e}";        // Visibility on
pub const ICON_FA_EYE_SLASH: &str = "\u{f070}";  // Visibility off

// Herramientas de desarrollo
pub const ICON_FA_CODE: &str = "\u{f121}";       // Code
pub const ICON_FA_BUG: &str = "\u{f188}";        // Debug
pub const ICON_FA_WRENCH: &str = "\u{f0ad}";     // Tools
pub const ICON_FA_PALETTE: &str = "\u{f53f}";    // Color/Materials
pub const ICON_FA_LIGHTBULB: &str = "\u{f0eb}";  // Lighting
pub const ICON_FA_CAMERA: &str = "\u{f030}";     // Camera

// Transformaciones y geometría
pub const ICON_FA_ARROWS_ALT: &str = "\u{f0b2}";     // Move/Transform
pub const ICON_FA_EXPAND_ARROWS: &str = "\u{f31e}";  // Scale
pub const ICON_FA_SYNC_ALT: &str = "\u{f2f1}";       // Rotate
pub const ICON_FA_VECTOR_SQUARE: &str = "\u{f5cb}";  // Bounding box
pub const ICON_FA_CROSSHAIRS: &str = "\u{f05b}";     // Target/Pivot

// Renderizado y efectos
pub const ICON_FA_SUN: &str = "\u{f185}";        // Directional light
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
