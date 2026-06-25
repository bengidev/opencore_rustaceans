//! In-memory credential store for tests.

use std::collections::HashMap;
use std::sync::Mutex;

use super::workspace_credential_store::{CredentialError, WorkspaceCredentialStore};

#[derive(Debug, Default)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct InMemoryWorkspaceCredentialStore {
    secrets: Mutex<HashMap<String, String>>,
}

impl InMemoryWorkspaceCredentialStore {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new() -> Self {
        Self::default()
    }
}

impl WorkspaceCredentialStore for InMemoryWorkspaceCredentialStore {
    fn secret(&self, provider_id: &str) -> Option<String> {
        self.secrets.lock().unwrap().get(provider_id).cloned()
    }

    fn save(&self, secret: &str, provider_id: &str) -> Result<(), CredentialError> {
        self.secrets
            .lock()
            .unwrap()
            .insert(provider_id.to_owned(), secret.to_owned());
        Ok(())
    }

    fn clear(&self, provider_id: &str) -> Result<(), CredentialError> {
        self.secrets.lock().unwrap().remove(provider_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::workspace_ai_provider::OPENROUTER_PROVIDER_ID;
    use super::*;

    #[test]
    fn save_load_clear_round_trip() {
        let store = InMemoryWorkspaceCredentialStore::new();
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
