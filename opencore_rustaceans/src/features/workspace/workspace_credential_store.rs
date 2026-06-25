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

    /// Trimmed, non-empty secret suitable for Authorization headers.
    fn resolved_secret(&self, provider_id: &str) -> Option<String> {
        non_empty_trimmed_secret(self.secret(provider_id))
    }
}

pub(crate) fn non_empty_trimmed_secret(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_trimmed_secret_rejects_blank_values() {
        assert_eq!(non_empty_trimmed_secret(Some(String::from("  "))), None);
        assert_eq!(
            non_empty_trimmed_secret(Some(String::from(" sk-or-x "))),
            Some(String::from("sk-or-x"))
        );
    }
}
