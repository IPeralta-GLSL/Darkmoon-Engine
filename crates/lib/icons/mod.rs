

pub mod font_setup;

pub const ICON_MIN_FA: u16 = 0xe000;
pub const ICON_MAX_16_FA: u16 = 0xf8ff;
pub const ICON_MAX_FA: u32 = 0x10ffff;

pub const ICON_FA_FILE: &str = "\u{f15b}";
pub const ICON_FA_FOLDER: &str = "\u{f07b}";
pub const ICON_FA_FOLDER_OPEN: &str = "\u{f07c}";

pub const ICON_FA_SCENE: &str = "\u{f008}";      
pub const ICON_FA_MODEL: &str = "\u{f1b2}";      
pub const ICON_FA_TEXTURE: &str = "\u{f03e}";    
pub const ICON_FA_SHADER: &str = "\u{f0e7}";     
pub const ICON_FA_AUDIO: &str = "\u{f028}";      
pub const ICON_FA_MESH: &str = "\u{f5fd}";       

pub const ICON_FA_PLAY: &str = "\u{f04b}";       
pub const ICON_FA_PAUSE: &str = "\u{f04c}";      
pub const ICON_FA_STOP: &str = "\u{f04d}";       
pub const ICON_FA_COGS: &str = "\u{f085}";       
pub const ICON_FA_EYE: &str = "\u{f06e}";        
pub const ICON_FA_EYE_SLASH: &str = "\u{f070}";  

pub const ICON_FA_CODE: &str = "\u{f121}";       
pub const ICON_FA_BUG: &str = "\u{f188}";        
pub const ICON_FA_WRENCH: &str = "\u{f0ad}";     
pub const ICON_FA_PALETTE: &str = "\u{f53f}";    
pub const ICON_FA_LIGHTBULB: &str = "\u{f0eb}";  
pub const ICON_FA_CAMERA: &str = "\u{f030}";     

pub const ICON_FA_ARROWS_ALT: &str = "\u{f0b2}";     
pub const ICON_FA_EXPAND_ARROWS: &str = "\u{f31e}";  
pub const ICON_FA_SYNC_ALT: &str = "\u{f2f1}";       
pub const ICON_FA_VECTOR_SQUARE: &str = "\u{f5cb}";  
pub const ICON_FA_CROSSHAIRS: &str = "\u{f05b}";     

pub const ICON_FA_SUN: &str = "\u{f185}";        
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
