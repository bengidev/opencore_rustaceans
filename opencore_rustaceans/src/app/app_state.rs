//! Active screen and shell state reducer.

use std::path::PathBuf;

use crate::features::welcome::{WelcomeOutcome, WelcomeState};
use crate::features::workspace::{WorkspaceMessage, WorkspaceOutcome, WorkspaceState};
use crate::shared::design::ThemeMode;

use super::app_messages::ShellMessage;

#[derive(Debug)]
pub enum ActiveScreen {
    Welcome(WelcomeState),
    Workspace(WorkspaceState),
}

pub struct AppState {
    pub screen: ActiveScreen,
    pub theme_mode: ThemeMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellUpdate {
    None,
    SessionChanged,
    ProjectClosed,
    StreamRequested(crate::features::workspace::ChatRequest),
    WelcomeAction(WelcomeOutcome),
}

impl AppState {
    pub fn new(screen: ActiveScreen, theme_mode: ThemeMode) -> Self {
        Self { screen, theme_mode }
    }

    #[allow(dead_code)]
    pub fn welcome(&self) -> Option<&WelcomeState> {
        match &self.screen {
            ActiveScreen::Welcome(state) => Some(state),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn welcome_mut(&mut self) -> Option<&mut WelcomeState> {
        match &mut self.screen {
            ActiveScreen::Welcome(state) => Some(state),
            _ => None,
        }
    }

    pub fn workspace(&self) -> Option<&WorkspaceState> {
        match &self.screen {
            ActiveScreen::Workspace(state) => Some(state),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn workspace_mut(&mut self) -> Option<&mut WorkspaceState> {
        match &mut self.screen {
            ActiveScreen::Workspace(state) => Some(state),
            _ => None,
        }
    }

    pub fn open_workspace(&mut self, path: PathBuf, has_api_key: bool) {
        let mut workspace = WorkspaceState::new(path, self.theme_mode);
        workspace.update(WorkspaceMessage::ApiKeyPresenceChanged(has_api_key));
        self.screen = ActiveScreen::Workspace(workspace);
    }

    pub fn open_welcome(&mut self, recent_paths: Vec<PathBuf>) {
        self.screen = ActiveScreen::Welcome(WelcomeState::with_recent_paths(
            self.theme_mode,
            recent_paths,
        ));
    }

    pub fn update(&mut self, message: ShellMessage) -> ShellUpdate {
        match message {
            ShellMessage::Welcome(welcome_message) => {
                if let ActiveScreen::Welcome(state) = &mut self.screen {
                    let outcome = state.update(welcome_message);
                    return map_welcome_outcome(outcome);
                }
                ShellUpdate::None
            }
            ShellMessage::Workspace(workspace_message) => {
                if let ActiveScreen::Workspace(state) = &mut self.screen {
                    let outcome = state.update(workspace_message);
                    return map_workspace_outcome(outcome);
                }
                ShellUpdate::None
            }
            ShellMessage::WindowCloseRequested => ShellUpdate::SessionChanged,
        }
    }
}

fn map_welcome_outcome(outcome: WelcomeOutcome) -> ShellUpdate {
    match outcome {
        WelcomeOutcome::None => ShellUpdate::None,
        WelcomeOutcome::ActionRequested(_)
        | WelcomeOutcome::CloneRequested(_)
        | WelcomeOutcome::WorkspaceOpened(_) => ShellUpdate::WelcomeAction(outcome),
    }
}

fn map_workspace_outcome(outcome: WorkspaceOutcome) -> ShellUpdate {
    match outcome {
        WorkspaceOutcome::None => ShellUpdate::None,
        WorkspaceOutcome::SessionChanged => ShellUpdate::SessionChanged,
        WorkspaceOutcome::ProjectClosed => ShellUpdate::ProjectClosed,
        WorkspaceOutcome::StreamRequested(request) => ShellUpdate::StreamRequested(request),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::welcome::WelcomeItemId;
    use crate::features::welcome::WelcomeMessage;

    #[test]
    fn boot_restore_opens_workspace_screen() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let workspace = WorkspaceState::new(path.clone(), ThemeMode::Dark);
        let state = AppState::new(ActiveScreen::Workspace(workspace), ThemeMode::Dark);
        assert!(state.workspace().is_some());
        assert_eq!(state.workspace().unwrap().project_path, path);
    }

    #[test]
    fn welcome_open_project_yields_workspace_opened_action() {
        let mut state = AppState::new(
            ActiveScreen::Welcome(WelcomeState::new(ThemeMode::Dark)),
            ThemeMode::Dark,
        );
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let update = state.update(ShellMessage::Welcome(
            WelcomeMessage::OpenProjectDialogCompleted(Some(path.clone())),
        ));
        assert_eq!(
            update,
            ShellUpdate::WelcomeAction(WelcomeOutcome::WorkspaceOpened(path))
        );
    }

    #[test]
    fn workspace_close_project_yields_project_closed() {
        let mut state = AppState::new(
            ActiveScreen::Workspace(WorkspaceState::new(
                PathBuf::from("/tmp/project"),
                ThemeMode::Dark,
            )),
            ThemeMode::Dark,
        );
        state.update(ShellMessage::Workspace(
            WorkspaceMessage::CloseProjectRequested,
        ));
        let update = state.update(ShellMessage::Workspace(
            WorkspaceMessage::CloseProjectConfirm,
        ));
        assert_eq!(update, ShellUpdate::ProjectClosed);
    }

    #[test]
    fn open_workspace_switches_screen() {
        let mut state = AppState::new(
            ActiveScreen::Welcome(WelcomeState::new(ThemeMode::Dark)),
            ThemeMode::Dark,
        );
        let path = PathBuf::from("/tmp/project");
        state.open_workspace(path.clone(), false);
        assert_eq!(state.workspace().unwrap().project_path, path);
    }

    #[test]
    fn welcome_item_pressed_returns_action_requested() {
        let mut state = AppState::new(
            ActiveScreen::Welcome(WelcomeState::new(ThemeMode::Dark)),
            ThemeMode::Dark,
        );
        let update = state.update(ShellMessage::Welcome(WelcomeMessage::ItemPressed(
            WelcomeItemId::NewFile,
        )));
        assert_eq!(
            update,
            ShellUpdate::WelcomeAction(WelcomeOutcome::ActionRequested(WelcomeItemId::NewFile))
        );
    }
}
