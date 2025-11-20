

pub mod font_awesome;
pub mod font_awesome_brands;
pub mod font_setup;

pub use font_awesome::*;
pub use font_awesome_brands::*;

pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        
        "dmoon" => &ICON_FILM.to_string(),

        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => &ICON_CUBE.to_string(),

        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => &ICON_IMAGE.to_string(),

        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => &ICON_BOLT.to_string(),

        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => &ICON_VOLUME_HIGH.to_string(),

        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => &ICON_CODE.to_string(),

        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => &ICON_GEAR.to_string(),

        _ => &ICON_FILE.to_string(),
    }
}

pub fn create_icon_label(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { &ICON_FOLDER_OPEN.to_string() } else { &ICON_FOLDER.to_string() };
    create_icon_label(icon, foldername)
}
pub const ICON_FA_MOON: &str = "\u{f186}";       
pub const ICON_FA_FIRE: &str = "\u{f06d}";       
pub const ICON_FA_WATER: &str = "\u{f773}";      
pub const ICON_FA_CLOUD: &str = "\u{f0c2}";      

pub const ICON_FA_HOME: &str = "\u{f015}";       
pub const ICON_FA_SEARCH: &str = "\u{f002}";     
pub const ICON_FA_PLUS: &str = "\u{f067}";       
pub const ICON_FA_MINUS: &str = "\u{f068}";      
pub const ICON_FA_TIMES: &str = "\u{f00d}";      
pub const ICON_FA_CHECK: &str = "\u{f00c}";      

pub const ICON_FA_SAVE: &str = "\u{f0c7}";       
pub const ICON_FA_FOLDER_PLUS: &str = "\u{f65e}"; 
pub const ICON_FA_DOWNLOAD: &str = "\u{f019}";   
pub const ICON_FA_UPLOAD: &str = "\u{f093}";     
pub const ICON_FA_DATABASE: &str = "\u{f1c0}";   

pub const ICON_FA_TACHOMETER: &str = "\u{f3f4}"; 
pub const ICON_FA_MEMORY: &str = "\u{f538}";     
pub const ICON_FA_MICROCHIP: &str = "\u{f2db}";  
pub const ICON_FA_CHART_BAR: &str = "\u{f080}";  

pub const FONT_ICON_FILE_NAME_FAS: &str = "fa-solid-900.otf";      
pub const FONT_ICON_FILE_NAME_FAB: &str = "fa-brands-400.otf";     
pub const FONT_ICON_FILE_NAME_FAR: &str = "fa-regular-400.otf";    

pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        
        "dmoon" => ICON_FA_SCENE,

        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => ICON_FA_MODEL,

        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => ICON_FA_TEXTURE,

        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => ICON_FA_SHADER,

        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => ICON_FA_AUDIO,

        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => ICON_FA_CODE,

        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => ICON_FA_COGS,

        _ => ICON_FA_FILE,
    }
}

pub fn create_icon_label(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { ICON_FA_FOLDER_OPEN } else { ICON_FA_FOLDER };
    create_icon_label(icon, foldername)
}
