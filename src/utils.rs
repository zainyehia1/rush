use std::fs;
use std::env;
use std::path;
use std::os::unix::fs::PermissionsExt;

pub fn get_path_executables() -> Vec<String> {
    let path = env::var("PATH").unwrap_or_default();
    env::split_paths(&path).flat_map(|dir| {
        fs::read_dir(dir).into_iter().flatten().filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
    }).collect()
}

pub fn locate_executables(command: &str, path: &str) -> Option<path::PathBuf> {
    env::split_paths(&path).map(|dir| dir.join(command)).find(|path| path.is_file() && {
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    })
}
