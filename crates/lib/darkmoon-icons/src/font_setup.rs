// Configuración de fuentes de iconos con imgui-rs 0.7
use imgui::{FontConfig, FontGlyphRanges, FontSource, Context};
use crate::*;

pub fn setup_icon_fonts(imgui: &mut Context) -> Result<(), String> {
    // Configuración de la fuente base
    let font_size = 16.0;
    let icon_font_size = font_size * 2.0 / 3.0; // Font Awesome necesita ser reducido
    
    // Cargar fuente de iconos desde assets/fonts/
    let font_path = format!("assets/fonts/{}", FONT_ICON_FILE_NAME_FAS);
    let font_data = std::fs::read(&font_path)
        .map_err(|e| format!("Error leyendo fuente {}: {}", font_path, e))?;
    
    // Configurar rango de iconos Font Awesome
    let icon_ranges = FontGlyphRanges::from_slice(&[font_awesome::ICON_MIN as u16, font_awesome::ICON_MAX_16 as u16, 0]);
    
    // Añadir fuente de iconos Font Awesome usando la API correcta
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: &font_data,
            size_pixels: icon_font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.0,
                glyph_ranges: icon_ranges,
                ..FontConfig::default()
            }),
        },
    ]);
    
    Ok(())
}

// Función helper para crear labels con iconos
pub fn create_icon_label_helper(icon: &str, text: &str) -> String {
    format!("{} {}", icon, text)
}

// Ejemplos de uso específicos para el asset browser
pub fn get_file_icon_label_helper(extension: &str, filename: &str) -> String {
    let icon = get_file_icon(extension);
    create_icon_label_helper(&icon.to_string(), filename)
}

pub fn get_folder_icon_label_helper(foldername: &str) -> String {
    create_icon_label_helper(&ICON_FOLDER.to_string(), foldername)
}
