//! OpenCore Rustaceans — composition root.
//!
//! Boots the first-run onboarding window when the sentinel flag is absent.
//! After completion the app exits; a future workspace shell will replace
//! this handoff.

mod features;
mod shared;

use std::sync::Arc;

use features::onboarding::{
    FileOnboardingPersistence, InMemoryOnboardingPersistence, OnboardingPersistence, run,
    should_run,
};
use shared::design::ThemeMode;

fn main() -> iced::Result {
    let persistence = load_persistence();
    if !should_run(persistence.as_ref()) {
        eprintln!("Onboarding already completed; main workspace is not yet available.");
        return Ok(());
    }
    run(persistence, ThemeMode::Dark)
}

fn load_persistence() -> Arc<dyn OnboardingPersistence> {
    match FileOnboardingPersistence::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryOnboardingPersistence::new()),
    }
}
