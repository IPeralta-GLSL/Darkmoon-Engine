# Integración de IconFontCppHeaders en Darkmoon Engine

Esta implementación integra [IconFontCppHeaders](https://github.com/juliettef/IconFontCppHeaders) para usar iconos profesionales Font Awesome en la interfaz de usuario de Darkmoon Engine.

## ¿Qué incluye?

- ✅ **Constantes de iconos Font Awesome 6** en Rust (`crates/lib/icons/mod.rs`)
- ✅ **Utilidades para configurar fuentes** con imgui-rs (`crates/lib/icons/font_setup.rs`)
- ✅ **Script de descarga automática** de Font Awesome (`scripts/download_fontawesome.sh`)
- ✅ **Ejemplo actualizado** del asset browser (`examples/asset_browser_with_icons.rs`)
- ✅ **Funciones helper** para obtener iconos por tipo de archivo

## Instalación rápida

1. **Descargar Font Awesome:**
   ```bash
   ./scripts/download_fontawesome.sh
   ```

2. **Añadir el módulo icons al Cargo.toml:**
   ```toml
   [dependencies]
   # ... tus dependencias existentes
   ```

3. **Integrar en tu aplicación imgui-rs:**
   ```rust
   use darkmoon_engine::icons::font_setup::setup_icon_fonts;
   
   // Durante la inicialización de imgui
   setup_icon_fonts(&mut imgui_context)?;
   ```

## Ventajas sobre emojis

| Aspecto | Emojis actuales | Font Awesome Icons |
|---------|----------------|-------------------|
| **Consistencia** | ❌ Varían por sistema | ✅ Siempre iguales |
| **Profesional** | ❌ Casuales | ✅ Diseño profesional |
| **Personalizable** | ❌ Color/tamaño fijo | ✅ Color y tamaño configurable |
| **Variedad** | ❌ Limitados | ✅ +2000 iconos disponibles |
| **Tema oscuro** | ❌ Pueden no verse bien | ✅ Se adaptan al tema |

## Iconos disponibles para Darkmoon Engine

### Assets del motor
- `ICON_FA_SCENE` 🎬 → **⚡** Escenas (.dmoon)
- `ICON_FA_MODEL` 🗿 → **📦** Modelos 3D (.gltf, .glb)
- `ICON_FA_TEXTURE` 🖼️ → **🖼️** Texturas e imágenes
- `ICON_FA_SHADER` ⚡ → **🔧** Shaders (.hlsl, .glsl)
- `ICON_FA_AUDIO` 🔊 → **🔊** Audio (.wav, .mp3)

### Herramientas de desarrollo
- `ICON_FA_PLAY` ▶️ Play/Run
- `ICON_FA_PAUSE` ⏸️ Pause
- `ICON_FA_COGS` ⚙️ Settings
- `ICON_FA_BUG` 🐛 Debug
- `ICON_FA_CAMERA` 📷 Camera

### UI y navegación
- `ICON_FA_FOLDER` 📁 Carpetas
- `ICON_FA_SEARCH` 🔍 Búsqueda
- `ICON_FA_PLUS` ➕ Añadir
- `ICON_FA_SAVE` 💾 Guardar

## Ejemplo de uso

```rust
use darkmoon_engine::icons::*;

// Crear label con icono
let scene_label = create_icon_label(ICON_FA_SCENE, "mi_escena.dmoon");

// Obtener icono por extensión de archivo
let icon = get_file_icon("gltf"); // Retorna ICON_FA_MODEL

// Label automático por tipo de archivo
let label = get_file_icon_label("png", "textura.png");
// Resultado: "🖼️ textura.png"
```

## Configuración avanzada

### Personalizar iconos por proyecto

Puedes extender los iconos añadiendo tus propias constantes:

```rust
// En tu mod.rs local
pub const ICON_DARKMOON_LOGO: &str = "\u{f123}"; // Tu icono personalizado
pub const ICON_CUSTOM_ASSET: &str = "\u{f456}";  // Tipo de asset específico
```

### Múltiples fuentes de iconos

```rust
// Combinar Font Awesome con otros sets de iconos
use darkmoon_engine::icons::*;

// Font Awesome para UI general
let ui_icon = ICON_FA_COGS; 

// Material Design para elementos específicos (si se añade)
// let material_icon = ICON_MD_SETTINGS;
```

## Archivos modificados en tu proyecto

Para integrar completamente, necesitarás actualizar:

1. **`asset_browser.rs`** - Reemplazar emojis con iconos
2. **Inicialización de imgui** - Cargar fuentes de iconos  
3. **`Cargo.toml`** - Añadir el módulo icons como dependencia
4. **Otros paneles UI** - Aplicar iconos consistentes

## Troubleshooting

### Los iconos no se muestran
- ✅ Verifica que las fuentes estén en `assets/fonts/`
- ✅ Asegúrate de que `setup_icon_fonts()` se llame antes de usar iconos
- ✅ Confirma que el rango de caracteres Unicode esté configurado correctamente

### Iconos cortados o mal alineados
- ✅ Ajusta el `icon_font_size` (recomendado: `base_font_size * 2.0 / 3.0`)
- ✅ Configura `glyph_min_advance_x` en la configuración de fuente
- ✅ Habilita `pixel_snap_h` para mejor alineación

## Próximos pasos

1. **Implementar la integración completa** en el asset browser actual
2. **Añadir más sets de iconos** (Material Design, Codicons)
3. **Crear temas de iconos** personalizados para Darkmoon Engine
4. **Implementar preview de texturas** con iconos contextuales
5. **Añadir tooltips informativos** con iconos descriptivos

## Recursos adicionales

- [IconFontCppHeaders GitHub](https://github.com/juliettef/IconFontCppHeaders)
- [Font Awesome Icons Gallery](https://fontawesome.com/icons)
- [imgui-rs Documentation](https://docs.rs/imgui/)
- [Dear ImGui Font Guide](https://github.com/ocornut/imgui/blob/master/docs/FONTS.md)
