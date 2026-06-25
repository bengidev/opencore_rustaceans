//! Internal welcome module — home screen shown after onboarding.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports the composition-facing API
//!   (`run`, `view`, `subscription`, `WelcomeState`, …) while hiding prefixed siblings.
//! * **Composite** — [`welcome_model::WelcomeScreen`] nests sections and items.
//! * **Command** — [`WelcomeMessage`] encodes user intents; the reducer
//!   dispatches them without knowing UI origin.
//! * **State** — [`WelcomeState::update`] owns transitions; [`WelcomeOutcome`]
//!   routes side effects to the host.
//! * **Strategy** — [`WelcomeHistory`] swaps filesystem vs in-memory backends.
//! * **Factory Method** — [`OpenCoreTheme::from_mode`] picks the concrete theme.
//!
//! Tests are colocated per module (TDD); run `cargo test welcome`.
//!
//! Flat layout with `welcome_`-prefixed modules:
//!
//! * [`welcome_model`] — screen catalog (sections + rows).
//! * [`welcome_messages`] — message enum.
//! * [`welcome_outcome`] — routing outcomes.
//! * [`welcome_state`] — state reducer.
//! * [`welcome_view`] — Iced view.
//! * [`welcome_history`] — recent-project persistence trait.
//! * [`welcome_actions`] — file/git helpers.
//! * [`welcome_command_palette`] — palette filtering.

mod welcome_actions;
mod welcome_command_palette;
mod welcome_file_history;
mod welcome_history;
mod welcome_memory_history;
mod welcome_messages;
mod welcome_model;
mod welcome_outcome;
mod welcome_overlay;
mod welcome_state;
mod welcome_view;

pub use welcome_file_history::FileWelcomeHistory;
pub use welcome_history::WelcomeHistory;
pub use welcome_memory_history::InMemoryWelcomeHistory;
#[allow(unused_imports)] // facade re-exports for embedders
pub use welcome_messages::WelcomeMessage;
#[allow(unused_imports)]
pub use welcome_model::{WelcomeItemId, WelcomeScreen, build_screen, default_screen};
#[allow(unused_imports)]
pub use welcome_outcome::WelcomeOutcome;
pub use welcome_state::WelcomeState;
pub use welcome_view::view;

pub use welcome_actions::{clone_destination, create_empty_file, default_clone_parent, git_clone};

use std::sync::Arc;

use iced::Subscription;
use iced::keyboard::{self, Modifiers, key};

use welcome_messages::WelcomeMessage as Msg;

/// Launch the welcome Iced application.
#[allow(dead_code)]
pub fn run(theme_mode: crate::shared::design::ThemeMode) -> iced::Result {
    crate::app::run(theme_mode)
}

/// Launch welcome with an explicit history backend (tests / embedders).
#[allow(dead_code)]
pub fn run_with_history(
    theme_mode: crate::shared::design::ThemeMode,
    history: Arc<dyn WelcomeHistory>,
) -> iced::Result {
    let credentials = crate::app::load_credentials_for_tests();
    crate::app::run_with_backends(
        theme_mode,
        history,
        crate::app::load_session_for_tests(),
        credentials.clone(),
        std::sync::Arc::new(crate::features::workspace::OpenRouterProvider::new(
            credentials,
        )),
    )
}

/// Keyboard shortcuts for embeddable welcome hosts.
pub fn subscription() -> Subscription<Msg> {
    keyboard::listen().filter_map(|event| match event {
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Shift),
            ..
        } => Some(Msg::ShiftPressed),
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Escape),
            ..
        } => Some(Msg::CommandPaletteDismiss),
        keyboard::Event::KeyPressed { key, modifiers, .. } => shortcut_message(key, modifiers),
        _ => None,
    })
}

fn shortcut_message(key: keyboard::Key, mods: Modifiers) -> Option<Msg> {
    if !mods.command() {
        return None;
    }

    match key {
        keyboard::Key::Character(character) => match character.as_str() {
            "n" | "N" => Some(Msg::ItemPressed(WelcomeItemId::NewFile)),
            "o" | "O" => Some(Msg::ItemPressed(WelcomeItemId::OpenProject)),
            "1" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(0))),
            "2" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(1))),
            "3" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(2))),
            "4" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(3))),
            "5" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(4))),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod shortcut_tests {
    use super::*;

    #[test]
    fn command_n_requests_new_file() {
        let message = shortcut_message(keyboard::Key::Character("n".into()), Modifiers::COMMAND);
        assert_eq!(message, Some(Msg::ItemPressed(WelcomeItemId::NewFile)));
    }

    #[test]
    fn command_o_requests_open_project() {
        let message = shortcut_message(keyboard::Key::Character("o".into()), Modifiers::COMMAND);
        assert_eq!(message, Some(Msg::ItemPressed(WelcomeItemId::OpenProject)));
    }
}
