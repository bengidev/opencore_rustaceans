//! In-memory recent-project history for tests and fallback.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::welcome_history::{WelcomeHistory, WelcomeHistoryError};

#[derive(Debug, Default, Clone)]
pub struct InMemoryWelcomeHistory {
    projects: Arc<Mutex<Vec<PathBuf>>>,
}

impl InMemoryWelcomeHistory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl WelcomeHistory for InMemoryWelcomeHistory {
    fn load(&self) -> Result<Vec<PathBuf>, WelcomeHistoryError> {
        Ok(self.projects.lock().unwrap().clone())
    }

    fn save(&self, projects: &[PathBuf]) -> Result<(), WelcomeHistoryError> {
        *self.projects.lock().unwrap() = projects.to_vec();
        Ok(())
    }
}
