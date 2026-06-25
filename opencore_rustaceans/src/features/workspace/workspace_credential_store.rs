//! Credential store strategy for AI provider secrets.

/// Errors from credential persistence backends.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CredentialError {
    #[error("credential store error: {0}")]
    Store(String),
}

/// Strategy for reading and writing provider API keys.
pub trait WorkspaceCredentialStore: Send + Sync {
    fn secret(&self, provider_id: &str) -> Option<String>;
    fn save(&self, secret: &str, provider_id: &str) -> Result<(), CredentialError>;
    fn clear(&self, provider_id: &str) -> Result<(), CredentialError>;
}
