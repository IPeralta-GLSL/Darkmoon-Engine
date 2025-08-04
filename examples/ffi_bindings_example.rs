// Ejemplo de cómo sería con bindings C reales (NO recomendado para este caso)

// En build.rs:
use std::env;
use std::path::PathBuf;

fn main() {
    // Configurar bindgen para generar bindings de IconFontCppHeaders
    let bindings = bindgen::Builder::default()
        .header("vendor/IconFontCppHeaders/IconsFontAwesome6.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// En Cargo.toml necesitarías:
// [build-dependencies]
// bindgen = "0.69"

// En lib.rs:
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
