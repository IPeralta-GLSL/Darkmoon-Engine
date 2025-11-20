use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("NGX/Lib/x64").display()
    );

    println!("cargo:rustc-link-lib=nvsdk_ngx_d");

    println!("cargo:rerun-if-changed=wrapper.h");

    let vulkan_sdk = env::var("VULKAN_SDK").unwrap_or_else(|_| {
        panic!("The environment variable `VULKAN_SDK` was not found. Is the Vulkan SDK installed?")
    });

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}/Include/Vulkan", vulkan_sdk))

        .header("wrapper.h")
        .allowlist_function("NVSDK.*")
        .allowlist_type("NVSDK.*")

        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        
        .generate()
        
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
