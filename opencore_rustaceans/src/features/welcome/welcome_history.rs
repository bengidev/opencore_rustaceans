//! Recent-project history persistence.
//!
//! Strategy trait with file and in-memory backends, mirroring onboarding.

use std::io;
use std::path::{Path, PathBuf};

/// Maximum recent projects shown on the welcome screen.
pub const MAX_RECENT_PROJECTS: usize = 5;

/// Persists recently opened project paths.
pub trait WelcomeHistory: Send + Sync {
    fn load(&self) -> Result<Vec<PathBuf>, WelcomeHistoryError>;
    fn save(&self, projects: &[PathBuf]) -> Result<(), WelcomeHistoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum WelcomeHistoryError {
    #[error("could not resolve a project data directory")]
    NoProjectDirs,

    #[error("could not serialize welcome history: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("io error while persisting welcome history: {0}")]
    Io(#[from] io::Error),
}

/// Insert or bump `path` to the front of `history`, capped at [`MAX_RECENT_PROJECTS`].
pub fn touch_project(history: &mut Vec<PathBuf>, path: PathBuf) {
    history.retain(|existing| existing != &path);
    history.insert(0, path);
    history.truncate(MAX_RECENT_PROJECTS);
}

/// Display label for a project path (folder basename).
pub fn project_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_project_moves_existing_entry_to_front() {
        let mut history = vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ];
        touch_project(&mut history, PathBuf::from("/b"));
        assert_eq!(
            history,
            vec![
                PathBuf::from("/b"),
                PathBuf::from("/a"),
                PathBuf::from("/c"),
            ]
        );
    }

    #[test]
    fn touch_project_caps_at_max() {
        let mut history: Vec<PathBuf> = (0..MAX_RECENT_PROJECTS)
            .map(|i| PathBuf::from(format!("/p{i}")))
            .collect();
        touch_project(&mut history, PathBuf::from("/new"));
        assert_eq!(history.len(), MAX_RECENT_PROJECTS);
        assert_eq!(history[0], PathBuf::from("/new"));
    }

    #[test]
    fn project_label_uses_basename() {
        assert_eq!(
            project_label(Path::new("/Users/demo/opencore_rustaceans")),
            "opencore_rustaceans"
        );
    }
}
