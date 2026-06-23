//! Welcome screen state reducer.

use std::path::PathBuf;

use crate::shared::design::{OpenCoreTheme, ThemeMode};

use super::welcome_command_palette::{filter_commands, palette_commands};
use super::welcome_history::touch_project;
use super::welcome_messages::WelcomeMessage;
use super::welcome_model::{WelcomeItemId, build_screen, item_id_at};
use super::welcome_outcome::WelcomeOutcome;
use super::welcome_overlay::WelcomeOverlay;

pub struct WelcomeState {
    pub theme: OpenCoreTheme,
    pub theme_mode: ThemeMode,
    pub hovered_item: Option<usize>,
    pub recent_paths: Vec<PathBuf>,
    pub overlay: WelcomeOverlay,
    pub palette_query: String,
    pub palette_selection: usize,
    pub clone_url: String,
    pub clone_error: Option<String>,
    pub status: Option<String>,
}

impl std::fmt::Debug for WelcomeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WelcomeState")
            .field("theme_mode", &self.theme_mode)
            .field("hovered_item", &self.hovered_item)
            .field("recent_paths", &self.recent_paths)
            .field("overlay", &self.overlay)
            .finish()
    }
}

impl WelcomeState {
    pub fn new(theme_mode: ThemeMode) -> Self {
        Self::with_recent_paths(theme_mode, Vec::new())
    }

    pub fn with_recent_paths(theme_mode: ThemeMode, recent_paths: Vec<PathBuf>) -> Self {
        Self {
            theme: OpenCoreTheme::from_mode(theme_mode),
            theme_mode,
            hovered_item: None,
            recent_paths,
            overlay: WelcomeOverlay::None,
            palette_query: String::new(),
            palette_selection: 0,
            clone_url: String::new(),
            clone_error: None,
            status: None,
        }
    }

    pub fn screen(&self) -> super::welcome_model::WelcomeScreen {
        build_screen(&self.recent_paths)
    }

    pub fn recent_labels(&self) -> Vec<String> {
        self.recent_paths
            .iter()
            .map(|path| super::welcome_history::project_label(path))
            .collect()
    }

    pub fn filtered_palette_commands(&self) -> Vec<super::welcome_command_palette::PaletteCommand> {
        let commands = palette_commands(&self.recent_labels());
        filter_commands(&commands, &self.palette_query)
    }

    pub fn update(&mut self, message: WelcomeMessage) -> WelcomeOutcome {
        let screen = self.screen();
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
            WelcomeMessage::ItemPressed(id) => self.request_action(id),
            WelcomeMessage::HistoryLoaded(paths) => {
                self.recent_paths = paths;
                WelcomeOutcome::None
            }
            WelcomeMessage::NewFileDialogCompleted(_) => WelcomeOutcome::None,
            WelcomeMessage::OpenProjectDialogCompleted(path) => match path {
                Some(path) => self.open_project(path),
                None => WelcomeOutcome::None,
            },
            WelcomeMessage::CloneUrlChanged(url) => {
                self.clone_url = url;
                self.clone_error = None;
                WelcomeOutcome::None
            }
            WelcomeMessage::CloneSubmit => {
                self.clone_error = None;
                WelcomeOutcome::None
            }
            WelcomeMessage::CloneCancel => {
                self.overlay = WelcomeOverlay::None;
                self.clone_error = None;
                WelcomeOutcome::None
            }
            WelcomeMessage::CloneCompleted(result) => match result {
                Ok(path) => {
                    self.overlay = WelcomeOverlay::None;
                    self.clone_url.clear();
                    self.clone_error = None;
                    self.open_project(path)
                }
                Err(error) => {
                    self.clone_error = Some(error);
                    WelcomeOutcome::None
                }
            },
            WelcomeMessage::CommandPaletteToggle => {
                self.toggle_command_palette();
                WelcomeOutcome::None
            }
            WelcomeMessage::CommandPaletteQueryChanged(query) => {
                self.palette_query = query;
                self.palette_selection = 0;
                WelcomeOutcome::None
            }
            WelcomeMessage::CommandPaletteSelect(index) => {
                if let Some(command) = self.filtered_palette_commands().get(index) {
                    let id = command.id;
                    self.overlay = WelcomeOverlay::None;
                    self.palette_query.clear();
                    self.palette_selection = 0;
                    return self.request_action(id);
                }
                WelcomeOutcome::None
            }
            WelcomeMessage::CommandPaletteDismiss => {
                self.overlay = WelcomeOverlay::None;
                self.palette_query.clear();
                self.palette_selection = 0;
                self.clone_error = None;
                WelcomeOutcome::None
            }
            WelcomeMessage::StatusDismiss => {
                self.status = None;
                WelcomeOutcome::None
            }
            WelcomeMessage::ShiftPressed => WelcomeOutcome::None,
            WelcomeMessage::ActionCompleted { summary, path } => {
                self.status = Some(summary);
                touch_project(&mut self.recent_paths, path);
                WelcomeOutcome::None
            }
        }
    }

    fn request_action(&mut self, id: WelcomeItemId) -> WelcomeOutcome {
        match id {
            WelcomeItemId::OpenCommandPalette => {
                self.toggle_command_palette();
                WelcomeOutcome::None
            }
            WelcomeItemId::CloneRepository => {
                self.overlay = WelcomeOverlay::CloneRepository;
                self.clone_error = None;
                WelcomeOutcome::ActionRequested(id)
            }
            WelcomeItemId::RecentProject(index) => self
                .recent_paths
                .get(index)
                .cloned()
                .map(|path| self.open_project(path))
                .unwrap_or(WelcomeOutcome::None),
            _ => WelcomeOutcome::ActionRequested(id),
        }
    }

    fn open_project(&mut self, path: PathBuf) -> WelcomeOutcome {
        touch_project(&mut self.recent_paths, path.clone());
        WelcomeOutcome::WorkspaceOpened(path)
    }

    fn toggle_command_palette(&mut self) {
        if self.overlay == WelcomeOverlay::CommandPalette {
            self.overlay = WelcomeOverlay::None;
            self.palette_query.clear();
            self.palette_selection = 0;
        } else {
            self.overlay = WelcomeOverlay::CommandPalette;
            self.palette_query.clear();
            self.palette_selection = 0;
        }
    }

    pub fn item_id_at(&self, index: usize) -> Option<WelcomeItemId> {
        item_id_at(&self.screen(), index)
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
    fn new_state_has_no_recent_projects() {
        let state = WelcomeState::new(ThemeMode::Dark);
        assert!(state.recent_paths.is_empty());
        assert_eq!(state.screen().sections.len(), 1);
    }

    #[test]
    fn screen_includes_recent_section_when_history_present() {
        let state = WelcomeState::with_recent_paths(
            ThemeMode::Dark,
            vec![PathBuf::from("/tmp/opencore_rustaceans")],
        );
        assert_eq!(state.screen().sections.len(), 2);
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

    #[test]
    fn open_project_updates_history_and_returns_workspace_opened() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        let path = PathBuf::from("/tmp/demo");
        let outcome = state.update(WelcomeMessage::OpenProjectDialogCompleted(Some(path.clone())));
        assert_eq!(outcome, WelcomeOutcome::WorkspaceOpened(path.clone()));
        assert_eq!(state.recent_paths[0], path);
    }

    #[test]
    fn command_palette_toggle_opens_and_closes() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        state.update(WelcomeMessage::CommandPaletteToggle);
        assert_eq!(state.overlay, WelcomeOverlay::CommandPalette);
        state.update(WelcomeMessage::CommandPaletteToggle);
        assert_eq!(state.overlay, WelcomeOverlay::None);
    }

    #[test]
    fn command_palette_select_runs_matching_action() {
        let mut state = WelcomeState::new(ThemeMode::Dark);
        state.overlay = WelcomeOverlay::CommandPalette;
        let outcome =
            state.update(WelcomeMessage::CommandPaletteSelect(0));
        assert_eq!(
            outcome,
            WelcomeOutcome::ActionRequested(WelcomeItemId::NewFile)
        );
        assert_eq!(state.overlay, WelcomeOverlay::None);
    }
}
