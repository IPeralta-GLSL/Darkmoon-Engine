use std::path::{Path, PathBuf};

/// Given an absolute path, returns the VFS path (with mount point) if it matches a mount point.
/// Example: /home/user/project/assets/meshes/floor/textures/foo.png -> /meshes/floor/textures/foo.png
pub fn to_vfs_path(abs_path: &Path) -> Option<PathBuf> {
    let mounts = [
        ("/meshes", "assets/meshes"),
        ("/images", "assets/images"),
        ("/shaders", "assets/shaders"),
        ("/rust-shaders-compiled", "assets/rust-shaders-compiled"),
        ("/cache", "cache"),
    ];
    let abs_canon = abs_path.canonicalize().ok()?;
    for (mount, real) in mounts.iter() {
        let real_canon = Path::new(real).canonicalize().ok()?;
        if abs_canon.starts_with(&real_canon) {
            let rel = abs_canon.strip_prefix(&real_canon).ok()?;
            let mut vfs_path = PathBuf::from(mount);
            vfs_path.push(rel);
            return Some(vfs_path);
        }
    }
    None
}
