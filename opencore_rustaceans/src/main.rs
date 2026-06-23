//! OpenCore Rustaceans — composition root.
//!
//! Boots onboarding on first run when needed, then the welcome home screen.

mod features;
mod shared;

use std::sync::Arc;

use features::onboarding::{
    FileOnboardingPersistence, InMemoryOnboardingPersistence, OnboardingPersistence,
    run as run_onboarding, should_run,
};
use features::welcome::run as run_welcome;
use shared::design::ThemeMode;

fn main() -> iced::Result {
    let persistence = load_persistence();
    if should_run(persistence.as_ref()) {
        run_onboarding(persistence, ThemeMode::Dark)?;
    }
    run_welcome(ThemeMode::Dark)
}

fn load_persistence() -> Arc<dyn OnboardingPersistence> {
    match FileOnboardingPersistence::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryOnboardingPersistence::new()),
    }
}
