//! Internal welcome module — home screen shown after onboarding.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports the composition-facing API
//!   (`run`, `view`, `WelcomeState`, …) while hiding prefixed siblings.
//! * **Composite** — [`welcome_model::WelcomeScreen`] nests sections and items.
//! * **Command** — [`WelcomeMessage`] encodes user intents; the reducer
//!   dispatches them without knowing UI origin.
//! * **State** — [`WelcomeState::update`] transitions hover/theme locally;
//!   [`WelcomeOutcome`] routes action requests to the host.
//! * **Factory Method** — [`OpenCoreTheme::from_mode`] picks the concrete theme.
//!
//! Tests are colocated per module (TDD); run `cargo test welcome`.
//!
//! Flat layout with `welcome_`-prefixed modules:
//!
//! * [`welcome_model`] — static screen catalog (sections + rows).
//! * [`welcome_messages`] — message enum.
//! * [`welcome_outcome`] — routing outcomes.
//! * [`welcome_state`] — state reducer.
//! * [`welcome_view`] — Iced view.

mod welcome_messages;
mod welcome_model;
mod welcome_outcome;
mod welcome_state;
mod welcome_view;

pub use welcome_messages::WelcomeMessage;
pub use welcome_model::{WelcomeItemId, WelcomeScreen, default_screen};
pub use welcome_outcome::WelcomeOutcome;
pub use welcome_state::WelcomeState;
pub use welcome_view::view;

use iced::{Element, Task, Theme};

/// Launch the welcome Iced application.
pub fn run(theme_mode: crate::shared::design::ThemeMode) -> iced::Result {
    iced::application(
        move || {
            let state = WelcomeState::new(theme_mode);
            (WelcomeApp { state }, Task::none())
        },
        WelcomeApp::update,
        WelcomeApp::view,
    )
    .title(WelcomeApp::title)
    .theme(WelcomeApp::theme)
    .window_size(iced::Size::new(720.0, 640.0))
    .run()
}

/// Iced application wrapper for the welcome screen.
pub struct WelcomeApp {
    state: WelcomeState,
}

impl WelcomeApp {
    fn update(&mut self, message: WelcomeMessage) -> Task<WelcomeMessage> {
        let outcome = self.state.update(message);
        match outcome {
            WelcomeOutcome::ActionRequested(id) => {
                eprintln!("welcome action not yet implemented: {id:?}");
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, WelcomeMessage> {
        view(&self.state)
    }

    fn title(&self) -> String {
        String::from("OpenCore")
    }

    fn theme(&self) -> Theme {
        match self.state.theme_mode {
            crate::shared::design::ThemeMode::Dark => Theme::Dark,
            crate::shared::design::ThemeMode::Light => Theme::Light,
        }
    }
}
