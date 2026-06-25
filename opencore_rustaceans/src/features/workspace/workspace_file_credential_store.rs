//! Filesystem-backed credential store (fallback when Keychain is unavailable).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use super::workspace_ai_provider::OPENROUTER_PROVIDER_ID;
use super::workspace_credential_store::{CredentialError, WorkspaceCredentialStore};

const CREDENTIALS_DIR: &str = "credentials";

#[derive(Debug, Clone)]
pub struct FileWorkspaceCredentialStore {
    credentials_dir: PathBuf,
}

impl FileWorkspaceCredentialStore {
    pub fn from_project_dirs() -> Result<Self, CredentialError> {
        let proj_dirs = ProjectDirs::from("com", "opencore", "opencore").ok_or(
            CredentialError::Store(String::from("no application data directory")),
        )?;
        Ok(Self::new_at(proj_dirs.data_dir()))
    }

    pub fn new_at<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            credentials_dir: dir.as_ref().join(CREDENTIALS_DIR),
        }
    }

    fn secret_path(&self, provider_id: &str) -> PathBuf {
        let filename = match provider_id {
            OPENROUTER_PROVIDER_ID => String::from("openrouter-api-key"),
            other => format!("{other}-api-key"),
        };
        self.credentials_dir.join(filename)
    }
}

impl WorkspaceCredentialStore for FileWorkspaceCredentialStore {
    fn secret(&self, provider_id: &str) -> Option<String> {
        let path = self.secret_path(provider_id);
        let raw = fs::read_to_string(path).ok()?;
        let secret = raw.trim().to_owned();
        if secret.is_empty() {
            None
        } else {
            Some(secret)
        }
    }

    fn save(&self, secret: &str, provider_id: &str) -> Result<(), CredentialError> {
        fs::create_dir_all(&self.credentials_dir)
            .map_err(|error| CredentialError::Store(error.to_string()))?;
        let path = self.secret_path(provider_id);
        let mut file =
            fs::File::create(&path).map_err(|error| CredentialError::Store(error.to_string()))?;
        file.write_all(secret.as_bytes())
            .map_err(|error| CredentialError::Store(error.to_string()))?;
        restrict_permissions(&path);
        Ok(())
    }

    fn clear(&self, provider_id: &str) -> Result<(), CredentialError> {
        let path = self.secret_path(provider_id);
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CredentialError::Store(error.to_string())),
        }
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_clear_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = FileWorkspaceCredentialStore::new_at(tmp.path());
        assert!(store.secret(OPENROUTER_PROVIDER_ID).is_none());
        store.save("secret-key", OPENROUTER_PROVIDER_ID).unwrap();
        assert_eq!(
            store.secret(OPENROUTER_PROVIDER_ID),
            Some(String::from("secret-key"))
        );
        store.clear(OPENROUTER_PROVIDER_ID).unwrap();
        assert!(store.secret(OPENROUTER_PROVIDER_ID).is_none());
    }
}
