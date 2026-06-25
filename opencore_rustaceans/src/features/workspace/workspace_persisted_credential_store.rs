//! Keychain + file credential store — file is source of truth when Keychain fails.

use super::workspace_credential_store::{
    CredentialError, WorkspaceCredentialStore, non_empty_trimmed_secret,
};
use super::workspace_file_credential_store::FileWorkspaceCredentialStore;
use super::workspace_keychain_store::KeychainWorkspaceCredentialStore;

#[derive(Debug, Clone)]
pub struct PersistedWorkspaceCredentialStore {
    keychain: KeychainWorkspaceCredentialStore,
    file: FileWorkspaceCredentialStore,
}

impl PersistedWorkspaceCredentialStore {
    pub fn from_project_dirs() -> Result<Self, CredentialError> {
        Ok(Self {
            keychain: KeychainWorkspaceCredentialStore::new(),
            file: FileWorkspaceCredentialStore::from_project_dirs()?,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new_at<P: AsRef<std::path::Path>>(dir: P) -> Self {
        Self {
            keychain: KeychainWorkspaceCredentialStore::new(),
            file: FileWorkspaceCredentialStore::new_at(dir),
        }
    }
}

impl WorkspaceCredentialStore for PersistedWorkspaceCredentialStore {
    fn secret(&self, provider_id: &str) -> Option<String> {
        non_empty_trimmed_secret(
            self.file
                .secret(provider_id)
                .or_else(|| self.keychain.secret(provider_id)),
        )
    }

    fn save(&self, secret: &str, provider_id: &str) -> Result<(), CredentialError> {
        let trimmed = secret.trim();
        if trimmed.is_empty() {
            return Err(CredentialError::Store(String::from(
                "API key cannot be empty",
            )));
        }
        self.file.save(trimmed, provider_id)?;
        if let Err(error) = self.keychain.save(trimmed, provider_id) {
            eprintln!("keychain save failed (file copy saved): {error}");
        }
        Ok(())
    }

    fn clear(&self, provider_id: &str) -> Result<(), CredentialError> {
        self.file.clear(provider_id)?;
        self.keychain.clear(provider_id)?;
        if self.secret(provider_id).is_some() {
            return Err(CredentialError::Store(String::from(
                "API key could not be fully removed",
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::workspace_ai_provider::OPENROUTER_PROVIDER_ID;
    use super::*;

    #[test]
    fn file_backstop_when_keychain_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = PersistedWorkspaceCredentialStore::new_at(tmp.path());
        store.save("secret-key", OPENROUTER_PROVIDER_ID).unwrap();
        assert_eq!(
            store.secret(OPENROUTER_PROVIDER_ID),
            Some(String::from("secret-key"))
        );
    }

    #[test]
    fn clear_removes_file_backed_secret() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = PersistedWorkspaceCredentialStore::new_at(tmp.path());
        store.save("secret-key", OPENROUTER_PROVIDER_ID).unwrap();
        store.clear(OPENROUTER_PROVIDER_ID).unwrap();
        assert!(store.secret(OPENROUTER_PROVIDER_ID).is_none());
    }
}
