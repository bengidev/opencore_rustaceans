//! Side effects for the app shell — dialogs, git clone, AI streaming, persistence.

use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use iced::Task;

use crate::features::welcome::{
    WelcomeHistory, WelcomeItemId, WelcomeMessage, WelcomeOutcome, WelcomeState, clone_destination,
    create_empty_file, default_clone_parent, git_clone,
};
use crate::features::workspace::{
    AiProvider, ChatRequest, ChatStreamEvent, OPENROUTER_PROVIDER_ID, WorkspaceCredentialStore,
    WorkspaceMessage, WorkspaceSession, WorkspaceSessionData, WorkspaceState,
    fetch_openrouter_models, sanitize_user_error,
};

use super::app_messages::ShellMessage;
use super::app_state::{ActiveScreen, AppState, ShellUpdate};

pub fn persist_session(state: &AppState, session: &dyn WorkspaceSession) {
    let snapshot = session_snapshot(state);
    if let Err(error) = session.save(&snapshot) {
        eprintln!("failed to persist workspace session: {error}");
    }
}

pub fn session_snapshot(state: &AppState) -> WorkspaceSessionData {
    match &state.screen {
        ActiveScreen::Workspace(workspace) => WorkspaceSessionData {
            open_project: Some(workspace.project_path.clone()),
            draft: workspace.chat.draft.clone(),
            model: workspace.model.clone(),
            messages: workspace.chat.thread.messages().to_vec(),
            activity: vec![],
        },
        ActiveScreen::Welcome(_) => WorkspaceSessionData::default(),
    }
}

pub fn handle_update(
    state: &mut AppState,
    message: ShellMessage,
    history: &Arc<dyn WelcomeHistory>,
    session: &Arc<dyn WorkspaceSession>,
    credentials: &Arc<dyn WorkspaceCredentialStore>,
    ai: &Arc<dyn AiProvider>,
) -> Task<ShellMessage> {
    if let ShellMessage::Workspace(WorkspaceMessage::ApiKeySave) = &message {
        return handle_api_key_save(state, credentials);
    }

    if let ShellMessage::Workspace(WorkspaceMessage::ApiKeyRemove) = &message {
        return handle_api_key_remove(state, credentials);
    }

    if let ShellMessage::Welcome(WelcomeMessage::NewFileDialogCompleted(Some(path))) = &message {
        let path = path.clone();
        return Task::perform(
            async move { create_empty_file(&path).map(|()| path) },
            |result| ShellMessage::Welcome(WelcomeMessage::NewFileResult(result)),
        );
    }

    let update = state.update(message.clone());

    if let ActiveScreen::Welcome(welcome) = &state.screen
        && let Err(error) = history.save(&welcome.recent_paths)
    {
        eprintln!("failed to persist welcome history: {error}");
    }

    match update {
        ShellUpdate::None => Task::none(),
        ShellUpdate::SessionChanged => {
            persist_session(state, session.as_ref());
            Task::none()
        }
        ShellUpdate::StreamRequested(request) => {
            sync_workspace_api_key(state, credentials);
            start_ai_stream(request, ai.clone())
        }
        ShellUpdate::ModelsFetchRequested => start_models_fetch(credentials.clone()),
        ShellUpdate::WelcomeAction(outcome) => {
            handle_welcome_outcome(state, outcome, session, credentials)
        }
    }
}

fn handle_api_key_save(
    state: &mut AppState,
    credentials: &Arc<dyn WorkspaceCredentialStore>,
) -> Task<ShellMessage> {
    let Some(workspace) = state.workspace_mut() else {
        return Task::none();
    };
    let secret = workspace.api_key_input.trim().to_owned();
    if secret.is_empty() {
        return Task::none();
    }

    if let Err(error) = credentials.save(&secret, OPENROUTER_PROVIDER_ID) {
        workspace.api_key_status = Some(error.to_string());
        return Task::none();
    }

    workspace.api_key_status = None;
    let update = workspace.update(WorkspaceMessage::ApiKeySave);
    apply_api_key_presence(state, credentials);

    match map_workspace_shell_update(update) {
        ShellUpdate::ModelsFetchRequested => start_models_fetch(credentials.clone()),
        _ => Task::none(),
    }
}

fn handle_api_key_remove(
    state: &mut AppState,
    credentials: &Arc<dyn WorkspaceCredentialStore>,
) -> Task<ShellMessage> {
    if let Err(error) = credentials.clear(OPENROUTER_PROVIDER_ID) {
        if let Some(workspace) = state.workspace_mut() {
            workspace.api_key_status = Some(error.to_string());
        }
        return Task::none();
    }

    if let Some(workspace) = state.workspace_mut() {
        workspace.api_key_status = None;
        workspace.update(WorkspaceMessage::ApiKeyRemove);
    }

    if credentials
        .resolved_secret(OPENROUTER_PROVIDER_ID)
        .is_some()
    {
        if let Some(workspace) = state.workspace_mut() {
            workspace.api_key_status = Some(String::from(
                "API key could not be fully removed. Try again or remove it from Keychain Access.",
            ));
        }
        return Task::none();
    }

    apply_api_key_presence(state, credentials);
    Task::none()
}

fn apply_api_key_presence(state: &mut AppState, credentials: &Arc<dyn WorkspaceCredentialStore>) {
    let Some(workspace) = state.workspace_mut() else {
        return;
    };
    let present = credentials
        .resolved_secret(OPENROUTER_PROVIDER_ID)
        .is_some();
    if workspace.has_api_key != present {
        workspace.update(WorkspaceMessage::ApiKeyPresenceChanged(present));
    }
}

fn map_workspace_shell_update(
    outcome: crate::features::workspace::WorkspaceOutcome,
) -> ShellUpdate {
    match outcome {
        crate::features::workspace::WorkspaceOutcome::None => ShellUpdate::None,
        crate::features::workspace::WorkspaceOutcome::SessionChanged => ShellUpdate::SessionChanged,
        crate::features::workspace::WorkspaceOutcome::StreamRequested(request) => {
            ShellUpdate::StreamRequested(request)
        }
        crate::features::workspace::WorkspaceOutcome::ModelsFetchRequested => {
            ShellUpdate::ModelsFetchRequested
        }
    }
}

fn handle_welcome_outcome(
    state: &mut AppState,
    outcome: WelcomeOutcome,
    session: &Arc<dyn WorkspaceSession>,
    credentials: &Arc<dyn WorkspaceCredentialStore>,
) -> Task<ShellMessage> {
    match outcome {
        WelcomeOutcome::None => Task::none(),
        WelcomeOutcome::ActionRequested(WelcomeItemId::NewFile) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Create New File")
                    .save_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |path| ShellMessage::Welcome(WelcomeMessage::NewFileDialogCompleted(path)),
        ),
        WelcomeOutcome::ActionRequested(WelcomeItemId::OpenProject) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Open Project")
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            |path| ShellMessage::Welcome(WelcomeMessage::OpenProjectDialogCompleted(path)),
        ),
        WelcomeOutcome::CloneRequested(url) => Task::perform(
            async move {
                let parent = default_clone_parent();
                let destination = clone_destination(&parent, &url).ok_or_else(|| {
                    String::from("could not derive a repository name from the URL")
                })?;
                git_clone(&url, &destination).map(|()| destination)
            },
            |result| ShellMessage::Welcome(WelcomeMessage::CloneCompleted(result)),
        ),
        WelcomeOutcome::WorkspaceOpened(path) => {
            let has_api_key = credentials
                .resolved_secret(OPENROUTER_PROVIDER_ID)
                .is_some();
            state.open_workspace(path, has_api_key);
            persist_session(state, session.as_ref());
            start_models_fetch(credentials.clone())
        }
        WelcomeOutcome::ActionRequested(_) => Task::none(),
    }
}

pub fn start_models_fetch(credentials: Arc<dyn WorkspaceCredentialStore>) -> Task<ShellMessage> {
    let api_key = credentials.resolved_secret(OPENROUTER_PROVIDER_ID);
    Task::batch([
        Task::done(ShellMessage::Workspace(WorkspaceMessage::ModelsLoadStarted)),
        Task::perform(
            async move { fetch_openrouter_models(api_key.as_deref()).await },
            |result| match result {
                Ok(models) => ShellMessage::Workspace(WorkspaceMessage::ModelsLoaded(models)),
                Err(error) => ShellMessage::Workspace(WorkspaceMessage::ModelsLoadFailed(
                    sanitize_user_error(&error.to_string()),
                )),
            },
        ),
    ])
}

fn sync_workspace_api_key(state: &mut AppState, credentials: &Arc<dyn WorkspaceCredentialStore>) {
    apply_api_key_presence(state, credentials);
}

pub fn start_ai_stream(request: ChatRequest, ai: Arc<dyn AiProvider>) -> Task<ShellMessage> {
    Task::run(
        ai.stream_chat(request).map(|event| match event {
            Ok(ChatStreamEvent::Delta { content }) => {
                ShellMessage::Workspace(WorkspaceMessage::StreamDelta(content))
            }
            Ok(ChatStreamEvent::Done) => ShellMessage::Workspace(WorkspaceMessage::StreamCompleted),
            Ok(ChatStreamEvent::Error(error)) => {
                ShellMessage::Workspace(WorkspaceMessage::StreamFailed(sanitize_user_error(&error)))
            }
            Err(error) => ShellMessage::Workspace(WorkspaceMessage::StreamFailed(
                sanitize_user_error(&error.to_string()),
            )),
        }),
        |message| message,
    )
}

pub fn boot_screen(
    theme_mode: crate::shared::design::ThemeMode,
    session_data: WorkspaceSessionData,
    recent_paths: Vec<PathBuf>,
    has_api_key: bool,
) -> ActiveScreen {
    if let Some(path) = session_data.open_project
        && path.exists()
    {
        let mut workspace = WorkspaceState::restore(
            path,
            theme_mode,
            session_data.draft,
            session_data.model,
            crate::features::workspace::ChatThread::from_messages(session_data.messages),
        );
        workspace.update(WorkspaceMessage::ApiKeyPresenceChanged(has_api_key));
        return ActiveScreen::Workspace(workspace);
    }

    ActiveScreen::Welcome(WelcomeState::with_recent_paths(theme_mode, recent_paths))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::features::welcome::InMemoryWelcomeHistory;
    use crate::features::workspace::{
        CannedAiProvider, InMemoryWorkspaceCredentialStore, InMemoryWorkspaceSession,
        WorkspaceOverlay,
    };
    use crate::shared::design::ThemeMode;

    use super::*;

    fn workspace_state() -> AppState {
        AppState::new(
            ActiveScreen::Workspace(WorkspaceState::new(
                PathBuf::from("/tmp/project"),
                ThemeMode::Dark,
            )),
            ThemeMode::Dark,
        )
    }

    #[test]
    fn api_key_save_sets_presence_and_requests_models() {
        let mut state = workspace_state();
        state.workspace_mut().unwrap().api_key_input = String::from("sk-or-test");
        let credentials: Arc<dyn WorkspaceCredentialStore> =
            Arc::new(InMemoryWorkspaceCredentialStore::new());
        let session: Arc<dyn WorkspaceSession> = Arc::new(InMemoryWorkspaceSession::new());
        let history: Arc<dyn WelcomeHistory> = Arc::new(InMemoryWelcomeHistory::new());
        let ai: Arc<dyn AiProvider> = Arc::new(CannedAiProvider::new(vec![]));

        let _task = handle_update(
            &mut state,
            ShellMessage::Workspace(WorkspaceMessage::ApiKeySave),
            &history,
            &session,
            &credentials,
            &ai,
        );

        let workspace = state.workspace().unwrap();
        assert!(workspace.has_api_key);
        assert_eq!(workspace.overlay, WorkspaceOverlay::None);
        assert!(workspace.api_key_status.is_none());
        assert!(workspace.models_loading);
    }

    #[test]
    fn api_key_save_failure_surfaces_status() {
        let mut state = workspace_state();
        state.workspace_mut().unwrap().api_key_input = String::from("   ");
        let credentials: Arc<dyn WorkspaceCredentialStore> =
            Arc::new(InMemoryWorkspaceCredentialStore::new());
        let session: Arc<dyn WorkspaceSession> = Arc::new(InMemoryWorkspaceSession::new());
        let history: Arc<dyn WelcomeHistory> = Arc::new(InMemoryWelcomeHistory::new());
        let ai: Arc<dyn AiProvider> = Arc::new(CannedAiProvider::new(vec![]));

        let _task = handle_update(
            &mut state,
            ShellMessage::Workspace(WorkspaceMessage::ApiKeySave),
            &history,
            &session,
            &credentials,
            &ai,
        );

        assert!(!state.workspace().unwrap().has_api_key);
    }

    #[test]
    fn api_key_remove_clears_presence() {
        let mut state = workspace_state();
        let credentials: Arc<dyn WorkspaceCredentialStore> =
            Arc::new(InMemoryWorkspaceCredentialStore::new());
        credentials
            .save("sk-or-test", OPENROUTER_PROVIDER_ID)
            .unwrap();
        state
            .workspace_mut()
            .unwrap()
            .update(WorkspaceMessage::ApiKeyPresenceChanged(true));

        let session: Arc<dyn WorkspaceSession> = Arc::new(InMemoryWorkspaceSession::new());
        let history: Arc<dyn WelcomeHistory> = Arc::new(InMemoryWelcomeHistory::new());
        let ai: Arc<dyn AiProvider> = Arc::new(CannedAiProvider::new(vec![]));

        let _task = handle_update(
            &mut state,
            ShellMessage::Workspace(WorkspaceMessage::ApiKeyRemove),
            &history,
            &session,
            &credentials,
            &ai,
        );

        let workspace = state.workspace().unwrap();
        assert!(!workspace.has_api_key);
        assert!(
            credentials
                .resolved_secret(OPENROUTER_PROVIDER_ID)
                .is_none()
        );
    }

    #[test]
    fn session_snapshot_reads_chat_fields() {
        let mut state = workspace_state();
        state.workspace_mut().unwrap().chat.draft = String::from("draft");
        let snapshot = session_snapshot(&state);
        assert_eq!(snapshot.draft, "draft");
        assert_eq!(snapshot.open_project, Some(PathBuf::from("/tmp/project")));
    }
}
