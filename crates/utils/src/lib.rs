use std::path::{Path, PathBuf};

use directories::ProjectDirs;

pub const EDMAN_UNIQUE_NAME: &str = "io.github.femshima.edman";

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("io.github", "femshima", "edman").expect("Project directory not found")
}

pub fn create_parent_dirs<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(unix)]
pub fn sock_path() -> PathBuf {
    let project_dirs = project_dirs();
    cfg_if::cfg_if! {
        if #[cfg(target_os="linux")] {
            let runtime_dir = project_dirs
                .runtime_dir()
                .expect("Runtime directory not found");
        } else {
            let runtime_dir = project_dirs.cache_dir();
        }
    };

    runtime_dir.join("edman.sock")
}

#[cfg(windows)]
pub fn sock_path() -> &'static str {
    "[::1]:50044"
}

pub fn manifest_path_firefox() -> PathBuf {
    project_dirs()
        .config_local_dir()
        .join("manifest_firefox.json")
}
pub fn manifest_path_chromium() -> PathBuf {
    project_dirs()
        .config_local_dir()
        .join("manifest_chromium.json")
}
