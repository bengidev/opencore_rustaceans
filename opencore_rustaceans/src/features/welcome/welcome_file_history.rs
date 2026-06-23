//! Filesystem-backed recent-project history.

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use super::welcome_history::{WelcomeHistory, WelcomeHistoryError};

const HISTORY_FILENAME: &str = "recent-projects.json";

#[derive(Debug, Clone)]
pub struct FileWelcomeHistory {
    history_path: PathBuf,
}

impl FileWelcomeHistory {
    pub fn from_project_dirs() -> Result<Self, WelcomeHistoryError> {
        let proj_dirs = ProjectDirs::from("com", "opencore", "opencore")
            .ok_or(WelcomeHistoryError::NoProjectDirs)?;
        Ok(Self::new_at(proj_dirs.data_dir()))
    }

    pub fn new_at<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            history_path: dir.as_ref().join(HISTORY_FILENAME),
        }
    }
}

impl WelcomeHistory for FileWelcomeHistory {
    fn load(&self) -> Result<Vec<PathBuf>, WelcomeHistoryError> {
        if !self.history_path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&self.history_path)?;
        let paths: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(paths.into_iter().map(PathBuf::from).collect())
    }

    fn save(&self, projects: &[PathBuf]) -> Result<(), WelcomeHistoryError> {
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let encoded: Vec<String> = projects
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();
        let raw = serde_json::to_string_pretty(&encoded)?;
        fs::write(&self.history_path, raw)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn starts_empty() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomeHistory::new_at(tmp.path());
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn round_trips_paths() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomeHistory::new_at(tmp.path());
        let paths = vec![
            PathBuf::from("/tmp/demo"),
            PathBuf::from("/tmp/playground"),
        ];
        store.save(&paths).unwrap();
        assert_eq!(store.load().unwrap(), paths);
    }

    #[test]
    fn corrupt_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let store = FileWelcomeHistory::new_at(tmp.path());
        fs::write(store.history_path.clone(), "not json").unwrap();
        assert!(store.load().unwrap().is_empty());
    }
}
