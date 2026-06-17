//! OpenCore Rustaceans — composition root.
//!
//! Boots the first-run onboarding window when the sentinel flag is absent.
//! After completion the app exits; a future workspace shell will replace
//! this handoff.

mod features;
mod shared;

use std::sync::Arc;

use features::onboarding::onboarding_file_persistence::FileOnboardingPersistence;
use features::onboarding::onboarding_memory_persistence::InMemoryOnboardingPersistence;
use features::onboarding::{OnboardingPersistence, run};
use shared::design::ThemeMode;

fn main() -> iced::Result {
    let persistence = load_persistence();
    run(persistence, ThemeMode::Dark)
}

fn load_persistence() -> Arc<dyn OnboardingPersistence> {
    match FileOnboardingPersistence::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryOnboardingPersistence::new()),
    }
}
