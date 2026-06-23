//! Action helpers for welcome screen behaviors.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Derive a destination folder name from a git remote URL.
pub fn repo_name_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let segment = trimmed.rsplit('/').next()?.trim_end_matches(".git");
    if segment.is_empty() {
        None
    } else {
        Some(segment.to_owned())
    }
}

/// Default clone parent directory under the user's home folder.
pub fn default_clone_parent() -> PathBuf {
    directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("OpenCore")
        .join("repositories")
}

/// Build the destination path for cloning `url` under `parent`.
pub fn clone_destination(parent: &Path, url: &str) -> Option<PathBuf> {
    Some(parent.join(repo_name_from_url(url)?))
}

/// Clone a git repository into `destination`.
pub fn git_clone(url: &str, destination: &Path) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err(String::from("repository URL is required"));
    }
    if destination.exists() {
        return Err(format!(
            "destination already exists: {}",
            destination.display()
        ));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let output = Command::new("git")
        .args(["clone", url, &destination.to_string_lossy()])
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_owned())
    }
}

/// Create an empty file at `path`, creating parent directories when needed.
pub fn create_empty_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("file already exists: {}", path.display()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::File::create(path).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn repo_name_from_https_url() {
        assert_eq!(
            repo_name_from_url("https://example.com/org/demo.git"),
            Some(String::from("demo"))
        );
    }

    #[test]
    fn repo_name_from_ssh_url() {
        assert_eq!(
            repo_name_from_url("git@github.com:org/playground.git"),
            Some(String::from("playground"))
        );
    }

    #[test]
    fn clone_destination_joins_parent_and_repo_name() {
        let parent = Path::new("/tmp/clones");
        assert_eq!(
            clone_destination(parent, "https://example.com/demo.git"),
            Some(PathBuf::from("/tmp/clones/demo"))
        );
    }

    #[test]
    fn create_empty_file_writes_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested/untitled.rs");
        create_empty_file(&path).unwrap();
        assert!(path.is_file());
    }
}
