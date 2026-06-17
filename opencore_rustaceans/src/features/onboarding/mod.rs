//! Internal onboarding module — first-run onboarding flow.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports the composition-facing API
//!   (`run`, `view`, `OnboardingState`, …) while hiding prefixed siblings.
//! * **Strategy** — [`OnboardingPersistence`] swaps filesystem vs in-memory
//!   backends without touching the reducer or view.
//! * **Command** — [`OnboardingMessage`] encodes user intents; the reducer
//!   dispatches them without knowing UI origin.
//! * **State** — [`OnboardingState::update`] transitions local animation and
//!   selection; [`OnboardingOutcome`] routes completion to the host.
//! * **Factory Method** — [`OpenCoreTheme::from_mode`] and persistence
//!   constructors (`from_project_dirs`, `new_at`) pick concrete products.
//! * **Template Method** — canvas [`Program`] impls (`GalaxyOrbProgram`,
//!   `SceneBackdrop`, `FeatureCardIcon`) share the iced draw lifecycle.
//!
//! Tests are colocated per module (TDD); run `cargo test onboarding`.
//!
//! Flat layout with `onboarding_`-prefixed modules:
//!
//! * [`onboarding_outcome`] — pure routing outcomes.
//! * [`onboarding_persistence`] — persistence trait contract.
//! * [`onboarding_state`] — state reducer.
//! * [`onboarding_messages`] — message enum.
//! * [`onboarding_dynamics`] — orb animation dynamics.
//! * [`onboarding_feature_card_dynamics`] — feature card animation helpers.
//! * [`onboarding_file_persistence`] — filesystem persistence backend.
//! * [`onboarding_memory_persistence`] — in-memory persistence backend.
//! * [`onboarding_view`] — Iced view.
//! * [`onboarding_feature_card_icon`] — wireframe feature icons.
//! * [`onboarding_galaxy_orb`] — galaxy orb canvas program.
//! * [`onboarding_scene_backdrop`] — animated scene backdrop.
//!
//! The module exposes only the composition-facing façade.

mod onboarding_dynamics;
mod onboarding_feature_card_dynamics;
mod onboarding_feature_card_icon;
mod onboarding_file_persistence;
mod onboarding_galaxy_orb;
mod onboarding_memory_persistence;
mod onboarding_messages;
mod onboarding_outcome;
mod onboarding_persistence;
mod onboarding_scene_backdrop;
mod onboarding_state;
mod onboarding_view;

pub use onboarding_file_persistence::FileOnboardingPersistence;
pub use onboarding_memory_persistence::InMemoryOnboardingPersistence;
pub use onboarding_messages::OnboardingMessage;
pub use onboarding_outcome::OnboardingOutcome;
pub use onboarding_persistence::OnboardingPersistence;
pub use onboarding_state::{OnboardingState, mark_completed};
pub use onboarding_view::view;

/// Returns `true` when the onboarding window should be shown on launch.
pub fn should_run(persistence: &dyn OnboardingPersistence) -> bool {
    !persistence.is_completed()
}

use iced::{Element, Subscription, Task, Theme};
use std::sync::Arc;

/// Launch the onboarding Iced application.
///
/// This is the high-level entry point for the composition root. It
/// wires the state, subscription, and view into an Iced window.
pub fn run(
    persistence: Arc<dyn OnboardingPersistence>,
    theme_mode: crate::shared::design::ThemeMode,
) -> iced::Result {
    iced::application(
        move || {
            let state = OnboardingState::new(persistence.clone(), theme_mode);
            (OnboardingApp { state }, Task::none())
        },
        OnboardingApp::update,
        OnboardingApp::view,
    )
    .title(OnboardingApp::title)
    .subscription(OnboardingApp::subscription)
    .theme(OnboardingApp::theme)
    .window_size(iced::Size::new(960.0, 680.0))
    .run()
}

/// Iced application wrapper for the onboarding flow.
pub struct OnboardingApp {
    state: OnboardingState,
}

impl OnboardingApp {
    fn update(&mut self, message: OnboardingMessage) -> Task<OnboardingMessage> {
        let outcome = self.state.update(message);
        match outcome {
            OnboardingOutcome::Completed | OnboardingOutcome::Skipped => {
                if let Err(error) = mark_completed(&self.state.persistence) {
                    eprintln!("failed to persist onboarding completion: {error}");
                }
                iced::exit()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, OnboardingMessage> {
        view(&self.state)
    }

    fn subscription(&self) -> Subscription<OnboardingMessage> {
        self.state.subscription()
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

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn should_run_when_onboarding_incomplete() {
        let persistence = InMemoryOnboardingPersistence::new();
        assert!(should_run(&persistence));
    }

    #[test]
    fn should_not_run_when_onboarding_complete() {
        let persistence = InMemoryOnboardingPersistence::new();
        persistence.mark_completed().unwrap();
        assert!(!should_run(&persistence));
    }
}
