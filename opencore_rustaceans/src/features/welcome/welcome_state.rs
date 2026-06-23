//! Welcome screen state reducer.

use crate::shared::design::{OpenCoreTheme, ThemeMode};

use super::welcome_messages::WelcomeMessage;
use super::welcome_model::{WelcomeItemId, default_screen, item_id_at};
use super::welcome_outcome::WelcomeOutcome;

pub struct WelcomeState {
    pub theme: OpenCoreTheme,
    pub theme_mode: ThemeMode,
    pub hovered_item: Option<usize>,
}

impl std::fmt::Debug for WelcomeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WelcomeState")
            .field("theme_mode", &self.theme_mode)
            .field("hovered_item", &self.hovered_item)
            .finish()
    }
}

impl WelcomeState {
    pub fn new(theme_mode: ThemeMode) -> Self {
        Self {
            theme: OpenCoreTheme::from_mode(theme_mode),
            theme_mode,
            hovered_item: None,
        }
    }

    pub fn screen(&self) -> super::welcome_model::WelcomeScreen {
        default_screen()
    }

    pub fn update(&mut self, message: WelcomeMessage) -> WelcomeOutcome {
        let screen = default_screen();
        let item_count = super::welcome_model::all_items(&screen).len();

        match message {
            WelcomeMessage::ToggleTheme => {
                self.theme_mode = self.theme_mode.toggle();
                self.theme = OpenCoreTheme::from_mode(self.theme_mode);
                WelcomeOutcome::ThemeToggled(self.theme_mode)
            }
            WelcomeMessage::ItemHovered(index) => {
                self.hovered_item = index.filter(|i| *i < item_count);
                WelcomeOutcome::None
            }
            WelcomeMessage::ItemPressed(id) => WelcomeOutcome::ActionRequested(id),
        }
    }

    pub fn item_id_at(&self, index: usize) -> Option<WelcomeItemId> {
        item_id_at(&default_screen(), index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_uses_requested_theme_mode() {
        let state = WelcomeState::new(ThemeMode::Dark);
        assert_eq!(state.theme_mode, ThemeMode::Dark);
        assert!(state.hovered_item.is_none());
    }

    #[test]
    fn toggle_theme_flips_mode_and_returns_outcome() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::ToggleTheme);
        assert_eq!(state.theme_mode, ThemeMode::Light);
        assert_eq!(outcome, WelcomeOutcome::ThemeToggled(ThemeMode::Light));
    }

    #[test]
    fn item_hovered_clamps_to_valid_indices() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        assert_eq!(
            state.update(WelcomeMessage::ItemHovered(Some(3))),
            WelcomeOutcome::None
        );
        assert_eq!(state.hovered_item, Some(3));

        assert_eq!(
            state.update(WelcomeMessage::ItemHovered(Some(99))),
            WelcomeOutcome::None
        );
        assert_eq!(state.hovered_item, None);
    }

    #[test]
    fn item_pressed_returns_action_requested() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::ItemPressed(WelcomeItemId::NewFile));
        assert_eq!(
            outcome,
            WelcomeOutcome::ActionRequested(WelcomeItemId::NewFile)
        );
    }
}
