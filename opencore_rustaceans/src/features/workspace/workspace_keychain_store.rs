//! macOS Keychain-backed credential store via keyring.

use super::workspace_ai_provider::OPENROUTER_PROVIDER_ID;
use super::workspace_credential_store::{CredentialError, WorkspaceCredentialStore};

const SERVICE: &str = "com.opencore.opencore";

#[derive(Debug, Default, Clone)]
pub struct KeychainWorkspaceCredentialStore;

impl KeychainWorkspaceCredentialStore {
    pub fn new() -> Self {
        Self
    }

    fn account(provider_id: &str) -> String {
        match provider_id {
            OPENROUTER_PROVIDER_ID => String::from("openrouter-api-key"),
            other => format!("{other}-api-key"),
        }
    }
}

impl WorkspaceCredentialStore for KeychainWorkspaceCredentialStore {
    fn secret(&self, provider_id: &str) -> Option<String> {
        let entry = keyring::Entry::new(SERVICE, &Self::account(provider_id)).ok()?;
        entry.get_password().ok()
    }

    fn save(&self, secret: &str, provider_id: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(SERVICE, &Self::account(provider_id))
            .map_err(|error| CredentialError::Store(error.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|error| CredentialError::Store(error.to_string()))
    }

    fn clear(&self, provider_id: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(SERVICE, &Self::account(provider_id))
            .map_err(|error| CredentialError::Store(error.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(CredentialError::Store(error.to_string())),
        }
    }
}
