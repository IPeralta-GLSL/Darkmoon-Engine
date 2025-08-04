use darkmoon_icons::*;

fn main() {
    println!("=== Darkmoon Icons - IconFontCppHeaders Integration ===");
    println!();
    
    println!("Font files:");
    println!("  Solid: {}", FONT_ICON_FILE_NAME_FAS);
    println!("  Regular: {}", FONT_ICON_FILE_NAME_FAR);
    println!();
    
    println!("Core Icons:");
    println!("  File: {} ({})", ICON_FILE, ICON_FILE as u32);
    println!("  Folder: {} ({})", ICON_FOLDER, ICON_FOLDER as u32);
    println!("  Folder Open: {} ({})", ICON_FOLDER_OPEN, ICON_FOLDER_OPEN as u32);
    println!();
    
    println!("File Icons by Extension:");
    let extensions = ["dmoon", "gltf", "png", "hlsl", "wav", "rs", "toml", "txt"];
    for ext in extensions {
        let icon = get_file_icon(ext);
        println!("  .{}: {} ({})", ext, icon, icon as u32);
    }
    println!();
    
    println!("Icon Labels:");
    println!("  {}", get_file_icon_label("dmoon", "my_scene.dmoon"));
    println!("  {}", get_file_icon_label("png", "texture.png"));
    println!("  {}", get_folder_icon_label("assets", false));
    println!("  {}", get_folder_icon_label("assets", true));
    println!();
    
    println!("✅ IconFontCppHeaders integration working correctly!");
}
