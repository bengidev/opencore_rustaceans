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
    WorkspaceMessage, WorkspaceSession, WorkspaceSessionData,
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
            draft: workspace.draft.clone(),
            model: workspace.model.clone(),
            messages: workspace.thread.messages().to_vec(),
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
    if let ShellMessage::Workspace(WorkspaceMessage::ApiKeySave) = &message
        && let Some(workspace) = state.workspace()
    {
        let secret = workspace.api_key_input.trim().to_owned();
        if !secret.is_empty()
            && let Err(error) = credentials.save(&secret, OPENROUTER_PROVIDER_ID)
        {
            eprintln!("failed to save API key: {error}");
        }
    }

    if let ShellMessage::Workspace(WorkspaceMessage::ApiKeyRemove) = &message
        && let Err(error) = credentials.clear(OPENROUTER_PROVIDER_ID)
    {
        eprintln!("failed to clear API key: {error}");
    }

    if let ShellMessage::Welcome(WelcomeMessage::NewFileDialogCompleted(Some(path))) = &message {
        let path = path.clone();
        return Task::perform(
            async move { create_empty_file(&path).map(|()| path) },
            |result| ShellMessage::Welcome(WelcomeMessage::NewFileResult(result)),
        );
    }

    let update = state.update(message);

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
        ShellUpdate::ProjectClosed => {
            if let Err(error) = session.clear_open_project() {
                eprintln!("failed to clear workspace session: {error}");
            }
            let recent_paths = history.load().unwrap_or_default();
            state.open_welcome(recent_paths);
            Task::none()
        }
        ShellUpdate::StreamRequested(mut request) => {
            if let Some(key) = credentials.secret(OPENROUTER_PROVIDER_ID) {
                request.api_key = key;
                start_ai_stream(request, ai.clone())
            } else {
                Task::done(ShellMessage::Workspace(WorkspaceMessage::StreamFailed(
                    String::from("missing API key"),
                )))
            }
        }
        ShellUpdate::WelcomeAction(outcome) => {
            handle_welcome_outcome(state, outcome, session, credentials)
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
            let has_api_key = credentials.secret(OPENROUTER_PROVIDER_ID).is_some();
            state.open_workspace(path, has_api_key);
            persist_session(state, session.as_ref());
            Task::none()
        }
        WelcomeOutcome::ActionRequested(_) => Task::none(),
    }
}

pub fn start_ai_stream(request: ChatRequest, ai: Arc<dyn AiProvider>) -> Task<ShellMessage> {
    Task::run(
        ai.stream_chat(request).map(|event| match event {
            Ok(ChatStreamEvent::Delta { content }) => {
                ShellMessage::Workspace(WorkspaceMessage::StreamDelta(content))
            }
            Ok(ChatStreamEvent::Done) => ShellMessage::Workspace(WorkspaceMessage::StreamCompleted),
            Ok(ChatStreamEvent::Error(error)) => {
                ShellMessage::Workspace(WorkspaceMessage::StreamFailed(error))
            }
            Err(error) => {
                ShellMessage::Workspace(WorkspaceMessage::StreamFailed(error.to_string()))
            }
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
        let mut workspace = crate::features::workspace::WorkspaceState::restore(
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
