use anyhow::anyhow;
use std::path::PathBuf;

pub fn find_git_root(dir: &PathBuf) -> anyhow::Result<PathBuf> {
    let mut git_root = dir.as_path();
    loop {
        match std::fs::metadata(git_root.join(".git")) {
            Ok(m) if m.is_dir() => {
                break;
            }
            Ok(_) | Err(_) => {
                // try one level up
                git_root = git_root
                    .parent()
                    .ok_or_else(|| anyhow!("Failed to detect git directory"))?;
            }
        }
    }
    Ok(git_root.join(".git").to_path_buf())
}

pub fn find_git_root_from_cwd() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir().expect("Expect retrieving current working directory");
    return find_git_root(&cwd);
}

pub fn read<P: std::convert::AsRef<std::path::Path>>(path: P) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

pub fn file_exists<P: std::convert::AsRef<std::path::Path>>(path: P) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            return metadata.is_file();
        }
        Err(_) => {
            return false;
        }
    }
}
