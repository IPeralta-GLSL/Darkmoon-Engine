use crate::{file::LoadFile, normalized_path_from_vfs, shader_compiler::CompiledShader};
use anyhow::{Context, Result};
use nanoserde::DeJson;
use parking_lot::Mutex;
use std::process::Command;
use turbosloth::*;

#[derive(Clone, Hash)]
pub struct CompileRustShader {
    pub entry: String,
}

#[async_trait]
impl LazyWorker for CompileRustShader {
    type Output = Result<CompiledShader>;

    async fn run(self, ctx: RunContext) -> Self::Output {
        CompileRustShaderCrate.into_lazy().eval(&ctx).await?;

        let compile_result = LoadFile::new("/rust-shaders-compiled/shaders.json")?
            .into_lazy()
            .eval(&ctx)
            .await?;

        let compile_result =
            RustShaderCompileResult::deserialize_json(std::str::from_utf8(&compile_result)?)?;

        let shader_file = compile_result
            .entry_to_shader_module
            .into_iter()
            .find_map(|(entry, module)| {
                if entry == self.entry {
                    Some(module)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                anyhow::anyhow!("No Rust-GPU module found for entry point {}", self.entry)
            })?;

        let spirv_blob = LoadFile::new(format!("/rust-shaders-compiled/{}", shader_file))?
            .into_lazy()
            .eval(&ctx)
            .await?;

        Ok(CompiledShader {
            name: "rust-gpu".to_owned(),
            spirv: (*spirv_blob).clone(),
        })
    }
}

#[derive(DeJson)]
struct RustShaderCompileResult {
    
    entry_to_shader_module: Vec<(String, String)>,
}

#[derive(Clone, Hash)]
pub struct CompileRustShaderCrate;

#[async_trait]
impl LazyWorker for CompileRustShaderCrate {
    type Output = Result<()>;

    async fn run(self, ctx: RunContext) -> Self::Output {
        let src_dirs = || -> Result<_> {
            Ok([
                normalized_path_from_vfs("/kajiya/crates/lib/rust-shaders/src")?,
                normalized_path_from_vfs("/kajiya/crates/lib/rust-shaders-shared/src")?,
            ])
        };

        let src_dirs = match src_dirs() {
            Ok(src_dirs) => src_dirs,
            Err(_) => {
                log::info!("Rust shader sources not found. Using the precompiled versions.");
                return Ok(());
            }
        };

        lazy_static::lazy_static! {
            static ref BUILD_TASK_CANCEL: Mutex<Option<std::sync::mpsc::Sender<()>>> = Mutex::new(None);
        }
        let mut prev_build_task_cancel = BUILD_TASK_CANCEL.lock();
        let (cancel_tx, cancel_rx) = std::sync::mpsc::channel();

        if let Some(cancel) = prev_build_task_cancel.replace(cancel_tx) {
            let _ = cancel.send(());
        }

        std::thread::spawn(move || -> anyhow::Result<()> {
            log::info!("Building Rust-GPU shaders in the background...");

            if let Err(err) = compile_rust_shader_crate_thread(cancel_rx) {
                log::error!("Failed to build Rust-GPU shaders. Falling back to the previously compiled ones. Error: {:?}", err);
            }

            Ok(())
        });

        for src_dir in src_dirs {
            let invalidation_trigger = ctx.get_invalidation_trigger();
            crate::file::FILE_WATCHER
                .lock()
                .watch(src_dir.clone(), move |event| {
                    if matches!(event, hotwatch::Event::Write(_)) {
                        invalidation_trigger();
                    }
                })
                .with_context(|| {
                    format!("CompileRustShaderCrate: trying to watch {:?}", src_dir)
                })?;
        }

        Ok(())
    }
}

fn compile_rust_shader_crate_thread(
    cancel_rx: std::sync::mpsc::Receiver<()>,
) -> anyhow::Result<()> {
    let builder_dir = normalized_path_from_vfs("/kajiya/crates/bin/rust-shader-builder")?;

    let mut child = Command::new("cargo")
        .env_remove("RUSTUP_TOOLCHAIN")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("CARGO_PROFILE_RELEASE_DEBUG")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")

        .env_remove("RUSTFLAGS")
        .env_remove("OUT_DIR")
        .arg("run")
        .arg("--release")
        .arg("--")
        .current_dir(builder_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to execute Rust-GPU builder")?;

    let output = loop {
        let should_bail = !matches!(
            cancel_rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        );

        if should_bail {
            log::info!("Rust-GPU shader builder thread received a stop command.");
            return child.kill().context("killing the Rust-GPU shader builder");
        }

        match child.try_wait() {
            
            Ok(Some(_)) => break child.wait_with_output()?,
            
            Ok(None) => (),
            
            Err(err) => return Err(err).context("error while executing Rust-GPU builder"),
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    };

    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        let out = String::from_utf8(output.stdout)?;
        anyhow::bail!("Shader builder failed:\n {}\n{}", out, err)
    } else {
        log::info!("Rust-GPU cargo process finished.");
    }

    Ok(())
}
