//! Internal workspace module — project shell, OpenRouter integration, and session.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports the composition-facing API (`view`,
//!   `WorkspaceState`, persistence traits, AI provider) while hiding prefixed siblings.
//! * **Command** — [`WorkspaceMessage`] encodes user intents; the reducer dispatches
//!   them without knowing UI origin.
//! * **State** — [`WorkspaceState::update`] owns project/model/overlay transitions;
//!   [`WorkspaceOutcome`] routes side effects to the app shell.
//! * **Strategy** — [`WorkspaceSession`], [`WorkspaceCredentialStore`], and
//!   [`AiProvider`] swap filesystem, keychain, memory, and OpenRouter backends.
//! * **Adapter** — [`workspace_chat`] translates between workspace messages and
//!   [`crate::features::chat`] events/outcomes.
//!
//! Chat UI and thread logic live in [`crate::features::chat`].
//! Tests are colocated per module (TDD); run `cargo test workspace`.
//!
//! Flat layout with `workspace_`-prefixed modules:
//!
//! * [`workspace_state`] — project shell reducer (embeds `ChatState`).
//! * [`workspace_messages`] — message enum.
//! * [`workspace_outcome`] — routing outcomes.
//! * [`workspace_view`] — header, chat composition, model chip.
//! * [`workspace_overlay`] — API key, model picker, close-project overlays.
//! * [`workspace_chat`] — chat module adapter.
//! * [`workspace_session`] — session persistence trait and data types.
//! * [`workspace_credential_store`] — credential store trait.
//! * [`workspace_ai_provider`] — AI provider trait and canned test double.
//! * [`workspace_openrouter_provider`] — OpenRouter SSE streaming adapter.
//! * [`workspace_openrouter_catalog`] — public model catalog fetcher.
//! * [`workspace_sse`] — SSE frame parser.

mod workspace_ai_provider;
mod workspace_chat;
mod workspace_credential_store;
mod workspace_file_credential_store;
mod workspace_file_session;
mod workspace_keychain_store;
mod workspace_memory_credential;
mod workspace_memory_session;
mod workspace_messages;
mod workspace_openrouter_catalog;
mod workspace_openrouter_provider;
mod workspace_outcome;
mod workspace_overlay;
mod workspace_persisted_credential_store;
mod workspace_scope;
mod workspace_session;
mod workspace_sse;
mod workspace_state;
mod workspace_view;

pub use crate::features::chat::ChatThread;
#[allow(unused_imports)]
pub use crate::features::chat::{ChatMessage, ChatRole};
#[allow(unused_imports)]
pub use workspace_ai_provider::{
    AiError, AiProvider, CannedAiProvider, ChatRequest, ChatStreamEvent, DEFAULT_MODEL,
    OPENROUTER_PROVIDER_ID, format_http_error, sanitize_user_error,
};
#[allow(unused_imports)]
pub use workspace_credential_store::{CredentialError, WorkspaceCredentialStore};
pub use workspace_file_session::FileWorkspaceSession;
pub use workspace_memory_credential::InMemoryWorkspaceCredentialStore;
pub use workspace_memory_session::InMemoryWorkspaceSession;
pub use workspace_messages::WorkspaceMessage;
#[allow(unused_imports)]
pub use workspace_openrouter_catalog::{ModelOption, fetch_openrouter_models};
pub use workspace_openrouter_provider::OpenRouterProvider;
pub use workspace_outcome::WorkspaceOutcome;
#[allow(unused_imports)]
pub use workspace_overlay::WorkspaceOverlay;
pub use workspace_persisted_credential_store::PersistedWorkspaceCredentialStore;
#[allow(unused_imports)]
pub use workspace_session::{SessionError, WorkspaceSession, WorkspaceSessionData};
pub use workspace_state::WorkspaceState;
pub use workspace_view::view;
