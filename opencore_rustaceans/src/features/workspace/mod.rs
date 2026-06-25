//! Internal workspace module — AI chat screen for an open project.
//!
//! Flat layout with `workspace_`-prefixed modules. Tests: `cargo test workspace`.

mod workspace_ai_provider;
mod workspace_credential_store;
mod workspace_file_session;
mod workspace_keychain_store;
mod workspace_memory_credential;
mod workspace_memory_session;
mod workspace_messages;
mod workspace_model;
mod workspace_openrouter_provider;
mod workspace_outcome;
mod workspace_overlay;
mod workspace_session;
mod workspace_sse;
mod workspace_state;
mod workspace_view;

#[allow(unused_imports)]
pub use workspace_ai_provider::{
    AiError, AiProvider, CannedAiProvider, ChatRequest, ChatStreamEvent, DEFAULT_MODEL,
    OPENROUTER_PROVIDER_ID,
};
#[allow(unused_imports)]
pub use workspace_credential_store::{CredentialError, WorkspaceCredentialStore};
pub use workspace_file_session::FileWorkspaceSession;
pub use workspace_keychain_store::KeychainWorkspaceCredentialStore;
pub use workspace_memory_credential::InMemoryWorkspaceCredentialStore;
pub use workspace_memory_session::InMemoryWorkspaceSession;
pub use workspace_messages::WorkspaceMessage;
#[allow(unused_imports)]
pub use workspace_model::{ChatMessage, ChatRole, ChatThread};
pub use workspace_openrouter_provider::OpenRouterProvider;
pub use workspace_outcome::WorkspaceOutcome;
#[allow(unused_imports)]
pub use workspace_overlay::WorkspaceOverlay;
#[allow(unused_imports)]
pub use workspace_session::{SessionError, WorkspaceSession, WorkspaceSessionData};
pub use workspace_state::WorkspaceState;
pub use workspace_view::view;
