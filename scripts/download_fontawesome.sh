#!/bin/bash

# Script para descargar Font Awesome 6 Free para Darkmoon Engine
# Basado en IconFontCppHeaders

echo "Descargando Font Awesome 6 Free..."

ASSETS_FONTS_DIR="assets/fonts"
FA_VERSION="6.5.1"
FA_URL="https://github.com/FortAwesome/Font-Awesome/releases/download/${FA_VERSION}/fontawesome-free-${FA_VERSION}-desktop.zip"

# Crear directorio si no existe
mkdir -p "${ASSETS_FONTS_DIR}"

# Descargar Font Awesome
echo "Descargando desde: ${FA_URL}"
curl -L "${FA_URL}" -o "/tmp/fontawesome.zip"

# Extraer solo los archivos de fuentes que necesitamos
echo "Extrayendo archivos de fuentes..."
unzip -j "/tmp/fontawesome.zip" "fontawesome-free-${FA_VERSION}-desktop/otfs/Font Awesome 6 Free-Solid-900.otf" -d "${ASSETS_FONTS_DIR}/"
unzip -j "/tmp/fontawesome.zip" "fontawesome-free-${FA_VERSION}-desktop/otfs/Font Awesome 6 Brands-Regular-400.otf" -d "${ASSETS_FONTS_DIR}/"

# Renombrar archivos para que sean más fáciles de usar
mv "${ASSETS_FONTS_DIR}/Font Awesome 6 Free-Solid-900.otf" "${ASSETS_FONTS_DIR}/fa-solid-900.otf"
mv "${ASSETS_FONTS_DIR}/Font Awesome 6 Brands-Regular-400.otf" "${ASSETS_FONTS_DIR}/fa-brands-400.otf"

# Limpiar archivo temporal
rm "/tmp/fontawesome.zip"

echo "✅ Font Awesome 6 descargado exitosamente en ${ASSETS_FONTS_DIR}/"
echo ""
echo "Archivos descargados:"
echo "- fa-solid-900.otf (iconos sólidos)"
echo "- fa-brands-400.otf (iconos de marcas)"
echo ""
echo "Para usar en tu código, incluye los archivos en tu aplicación y configura imgui-rs"
echo "para cargar estas fuentes junto con los iconos definidos en icons/mod.rs"
