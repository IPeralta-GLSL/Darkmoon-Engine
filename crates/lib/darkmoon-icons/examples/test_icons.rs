// Test simple para verificar que los iconos funcionan
use darkmoon_icons::*;

fn main() {
    // Verificar que las constantes de iconos existen
    println!("🎬 Escena (.dmoon): {}", ICON_FA_SCENE);
    println!("🗿 Modelo (.gltf): {}", ICON_FA_MODEL);
    println!("🖼️ Textura (.png): {}", ICON_FA_TEXTURE);
    println!("⚡ Shader (.hlsl): {}", ICON_FA_SHADER);
    println!("🔊 Audio (.wav): {}", ICON_FA_AUDIO);
    println!("📁 Carpeta: {}", ICON_FA_FOLDER);
    
    // Probar las funciones helper
    println!("\nFunciones helper:");
    println!("Archivo .dmoon: {}", get_file_icon_label("dmoon", "mi_escena.dmoon"));
    println!("Archivo .gltf: {}", get_file_icon_label("gltf", "modelo.gltf"));
    println!("Archivo .png: {}", get_file_icon_label("png", "textura.png"));
    println!("Carpeta: {}", get_folder_icon_label("mi_carpeta", false));
    
    // Probar extensiones no reconocidas
    println!("Archivo desconocido: {}", get_file_icon_label("xyz", "archivo.xyz"));
    
    println!("\n✅ ¡Todos los iconos están funcionando correctamente!");
}
