
pub mod font_awesome;
pub mod font_awesome_brands;
pub mod font_setup;
pub use font_awesome::{
    ICON_FILE, ICON_FOLDER, ICON_FOLDER_OPEN, ICON_FILM, ICON_CUBE, 
    ICON_IMAGE, ICON_BOLT, ICON_VOLUME_HIGH, ICON_CODE, ICON_GEAR,
    FONT_ICON_FILE_NAME_FAS, FONT_ICON_FILE_NAME_FAR
};
pub use font_awesome_brands::*;
pub fn get_file_icon(extension: &str) -> char {
    match extension.to_lowercase().as_str() {
        "dmoon" => ICON_FILM,
        
        "gltf" | "glb" | "obj" | "fbx" | "dae" | "3ds" | "blend" => ICON_CUBE,
        
        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "hdr" | "exr" | "tiff" => ICON_IMAGE,
        
        "hlsl" | "glsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => ICON_BOLT,
        
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => ICON_VOLUME_HIGH,
        
        "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" => ICON_CODE,
        
        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" => ICON_GEAR,
        
        _ => ICON_FILE,
    }
}

pub fn create_icon_label(icon: char, text: &str) -> String {
    format!("{} {}", icon, text)
}

pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label(icon, filename)
}

pub fn get_folder_icon_label(foldername: &str, is_open: bool) -> String {
    let icon = if is_open { ICON_FOLDER_OPEN } else { ICON_FOLDER };
    create_icon_label(icon, foldername)
}
