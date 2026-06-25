//! App shell — routes welcome ↔ workspace and owns the single Iced window.

mod app_effects;
mod app_messages;
mod app_state;

#[allow(unused_imports)]
pub use app_messages::ShellMessage;
#[allow(unused_imports)]
pub use app_state::{ActiveScreen, AppState, ShellUpdate};

use std::sync::Arc;

use iced::{Element, Subscription, Task, Theme, exit, window};

use crate::features::welcome::{
    FileWelcomeHistory, InMemoryWelcomeHistory, WelcomeHistory,
    subscription as welcome_subscription, view as welcome_view,
};
use crate::features::workspace::{
    AiProvider, FileWorkspaceSession, InMemoryWorkspaceCredentialStore, InMemoryWorkspaceSession,
    KeychainWorkspaceCredentialStore, OPENROUTER_PROVIDER_ID, OpenRouterProvider,
    WorkspaceCredentialStore, WorkspaceSession, view as workspace_view,
};
use crate::shared::design::ThemeMode;

use app_effects::{boot_screen, handle_update, persist_session};
use app_messages::ShellMessage as Msg;

/// Launch the full OpenCore application shell.
pub fn run(theme_mode: ThemeMode) -> iced::Result {
    run_with_backends(
        theme_mode,
        load_history(),
        load_session(),
        load_credentials(),
        Arc::new(OpenRouterProvider::new()),
    )
}

/// Launch with explicit backends (tests / embedders).
pub fn run_with_backends(
    theme_mode: ThemeMode,
    history: Arc<dyn WelcomeHistory>,
    session: Arc<dyn WorkspaceSession>,
    credentials: Arc<dyn WorkspaceCredentialStore>,
    ai: Arc<dyn AiProvider>,
) -> iced::Result {
    iced::application(
        move || {
            let session_data = session.load().unwrap_or_default();
            let recent_paths = history.load().unwrap_or_default();
            let has_api_key = credentials.secret(OPENROUTER_PROVIDER_ID).is_some();
            let screen = boot_screen(theme_mode, session_data, recent_paths, has_api_key);
            (
                ShellApp {
                    state: AppState::new(screen, theme_mode),
                    history: history.clone(),
                    session: session.clone(),
                    credentials: credentials.clone(),
                    ai: ai.clone(),
                },
                Task::none(),
            )
        },
        ShellApp::update,
        ShellApp::view,
    )
    .title(ShellApp::title)
    .theme(ShellApp::theme)
    .subscription(ShellApp::subscription)
    .window_size(iced::Size::new(960.0, 680.0))
    .exit_on_close_request(false)
    .run()
}

struct ShellApp {
    state: AppState,
    history: Arc<dyn WelcomeHistory>,
    session: Arc<dyn WorkspaceSession>,
    credentials: Arc<dyn WorkspaceCredentialStore>,
    ai: Arc<dyn AiProvider>,
}

impl ShellApp {
    fn update(&mut self, message: Msg) -> Task<Msg> {
        if message == Msg::WindowCloseRequested {
            persist_session(&self.state, self.session.as_ref());
            return exit();
        }

        handle_update(
            &mut self.state,
            message,
            &self.history,
            &self.session,
            &self.credentials,
            &self.ai,
        )
    }

    fn view(&self) -> Element<'_, Msg> {
        match &self.state.screen {
            ActiveScreen::Welcome(welcome) => welcome_view(welcome).map(Msg::Welcome),
            ActiveScreen::Workspace(workspace) => workspace_view(workspace).map(Msg::Workspace),
        }
    }

    fn title(&self) -> String {
        String::from("OpenCore")
    }

    fn theme(&self) -> Theme {
        match self.state.theme_mode {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            welcome_subscription().map(Msg::Welcome),
            window::close_requests().map(|_| Msg::WindowCloseRequested),
        ])
    }
}

fn load_history() -> Arc<dyn WelcomeHistory> {
    match FileWelcomeHistory::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryWelcomeHistory::new()),
    }
}

fn load_session() -> Arc<dyn WorkspaceSession> {
    match FileWorkspaceSession::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryWorkspaceSession::new()),
    }
}

fn load_credentials() -> Arc<dyn WorkspaceCredentialStore> {
    Arc::new(KeychainWorkspaceCredentialStore::new())
}

/// In-memory session store for embedder tests.
#[allow(dead_code)]
pub fn load_session_for_tests() -> Arc<dyn WorkspaceSession> {
    Arc::new(InMemoryWorkspaceSession::new())
}

/// In-memory credential store for embedder tests.
#[allow(dead_code)]
pub fn load_credentials_for_tests() -> Arc<dyn WorkspaceCredentialStore> {
    Arc::new(InMemoryWorkspaceCredentialStore::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_without_open_project_starts_on_welcome() {
        let session = Arc::new(InMemoryWorkspaceSession::new());
        let screen = boot_screen(ThemeMode::Dark, session.load().unwrap(), vec![], false);
        assert!(matches!(screen, ActiveScreen::Welcome(_)));
    }

    #[test]
    fn boot_with_open_project_restores_workspace() {
        let session = Arc::new(InMemoryWorkspaceSession::new());
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        session
            .save(&crate::features::workspace::WorkspaceSessionData {
                open_project: Some(path.clone()),
                draft: String::new(),
                model: String::from("openai/gpt-4o-mini"),
                messages: vec![],
                activity: vec![],
            })
            .unwrap();
        let screen = boot_screen(ThemeMode::Dark, session.load().unwrap(), vec![], false);
        match screen {
            ActiveScreen::Workspace(workspace) => assert_eq!(workspace.project_path, path),
            _ => panic!("expected workspace restore"),
        }
    }
}
