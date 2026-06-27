//! Internal chat module — thread, composer, and streaming state.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports embedder-facing widgets (`body`, `composer`)
//!   and state types while hiding prefixed siblings.
//! * **Command** — [`ChatEvent`] encodes user intents from the composer and host stream.
//! * **State** — [`ChatState::update`] owns thread/draft/streaming transitions;
//!   [`ChatOutcome`] routes side effects to the host.
//! * **Composite** — [`ChatThread`] nests ordered [`ChatMessage`] rows.
//!
//! Composed by [`crate::features::workspace`]; no standalone `run` entry point.
//! Tests are colocated per module (TDD); run `cargo test chat`.
//!
//! Flat layout with `chat_`-prefixed modules:
//!
//! * [`chat_model`] — message, role, and thread types.
//! * [`chat_messages`] — event enum (`ChatEvent`).
//! * [`chat_outcome`] — host routing outcomes.
//! * [`chat_state`] — reducer for draft, thread, and streaming.
//! * [`chat_view`] — empty state and thread bubbles.
//! * [`chat_composer`] — config toolbar, input card, and token meter.
//! * [`chat_style`] — composer input styling.
//! * [`chat_brand`] — OpenRouter logo handle.

mod chat_brand;
mod chat_composer;
mod chat_messages;
mod chat_model;
mod chat_outcome;
mod chat_state;
mod chat_style;
mod chat_view;

pub use chat_composer::composer;
pub use chat_messages::ChatEvent;
pub use chat_model::{ChatMessage, ChatRole, ChatThread};
pub use chat_outcome::ChatOutcome;
pub use chat_state::{ChatState, DEFAULT_TOKEN_BUDGET};
pub use chat_view::body;
