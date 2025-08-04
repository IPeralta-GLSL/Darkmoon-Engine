// Ejemplo de configuración de fuentes de iconos con imgui-rs
// Esto sería parte de la inicialización de tu aplicación

use imgui::{FontConfig, FontGlyphRanges, FontSource, Context};
use crate::icons::*;

pub fn setup_icon_fonts(imgui: &mut Context) -> Result<(), String> {
    let io = imgui.io_mut();
    
    // Configuración de la fuente base
    let font_size = 16.0;
    let icon_font_size = font_size * 2.0 / 3.0; // Font Awesome necesita ser reducido
    
    // Añadir fuente por defecto
    io.fonts.add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: font_size,
            ..FontConfig::default()
        }),
    }]);
    
    // Configurar rango de iconos Font Awesome
    let icon_ranges = FontGlyphRanges::from_slice(&[ICON_MIN_FA, ICON_MAX_16_FA, 0]);
    
    // Añadir fuente de iconos Font Awesome
    io.fonts.add_font(&[FontSource::TtfData {
        data: include_bytes!("../../../assets/fonts/fa-solid-900.ttf"), // Asegúrate de tener el archivo
        size_pixels: icon_font_size,
        config: Some(FontConfig {
            merge_mode: true,
            pixel_snap_h: true,
            glyph_min_advance_x: icon_font_size,
            glyph_ranges: icon_ranges,
            ..FontConfig::default()
        }),
    }]);
    
    Ok(())
}

// Función helper para crear labels con iconos
pub fn create_icon_label(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

// Ejemplos de uso específicos para el asset browser
pub fn get_file_icon_label(extension: &str, filename: &str) -> String {
    let icon = match extension {
        "dmoon" => ICON_FA_SCENE,
        "gltf" | "glb" => ICON_FA_MODEL,
        "png" | "jpg" | "jpeg" | "tga" | "dds" | "hdr" | "exr" => ICON_FA_TEXTURE,
        "hlsl" | "glsl" | "wgsl" => ICON_FA_SHADER,
        "wav" | "mp3" | "ogg" => ICON_FA_AUDIO,
        _ => ICON_FA_FILE,
    };
    
    create_icon_label(icon, filename)
}

pub fn get_folder_icon_label(foldername: &str) -> String {
    create_icon_label(ICON_FA_FOLDER, foldername)
}
