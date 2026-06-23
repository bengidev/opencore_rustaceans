//! Internal welcome module — home screen shown after onboarding.
//!
//! ## Design patterns (GoF)
//!
//! * **Facade** — this `mod.rs` re-exports the composition-facing API
//!   (`run`, `view`, `WelcomeState`, …) while hiding prefixed siblings.
//! * **Composite** — [`welcome_model::WelcomeScreen`] nests sections and items.
//! * **Command** — [`WelcomeMessage`] encodes user intents; the reducer
//!   dispatches them without knowing UI origin.
//! * **State** — [`WelcomeState::update`] transitions hover/theme locally;
//!   [`WelcomeOutcome`] routes action requests to the host.
//! * **Strategy** — [`WelcomeHistory`] swaps filesystem vs in-memory backends.
//! * **Factory Method** — [`OpenCoreTheme::from_mode`] picks the concrete theme.
//!
//! Tests are colocated per module (TDD); run `cargo test welcome`.
//!
//! Flat layout with `welcome_`-prefixed modules:
//!
//! * [`welcome_model`] — screen catalog (sections + rows).
//! * [`welcome_messages`] — message enum.
//! * [`welcome_outcome`] — routing outcomes.
//! * [`welcome_state`] — state reducer.
//! * [`welcome_view`] — Iced view.
//! * [`welcome_history`] — recent-project persistence trait.
//! * [`welcome_actions`] — file/git helpers.
//! * [`welcome_command_palette`] — palette filtering.

mod welcome_actions;
mod welcome_command_palette;
mod welcome_file_history;
mod welcome_history;
mod welcome_memory_history;
mod welcome_messages;
mod welcome_model;
mod welcome_outcome;
mod welcome_overlay;
mod welcome_state;
mod welcome_view;

pub use welcome_file_history::FileWelcomeHistory;
pub use welcome_history::WelcomeHistory;
pub use welcome_memory_history::InMemoryWelcomeHistory;
#[allow(unused_imports)] // facade re-exports for embedders
pub use welcome_messages::WelcomeMessage;
#[allow(unused_imports)]
pub use welcome_model::{WelcomeItemId, WelcomeScreen, build_screen, default_screen};
#[allow(unused_imports)]
pub use welcome_outcome::WelcomeOutcome;
pub use welcome_state::WelcomeState;
pub use welcome_view::view;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::keyboard::{self, Modifiers, key};
use iced::{Element, Subscription, Task, Theme};

use welcome_actions::{clone_destination, create_empty_file, default_clone_parent, git_clone};
use welcome_messages::WelcomeMessage as Msg;
use welcome_outcome::WelcomeOutcome as Outcome;

/// Launch the welcome Iced application.
pub fn run(theme_mode: crate::shared::design::ThemeMode) -> iced::Result {
    run_with_history(theme_mode, load_history())
}

/// Launch welcome with an explicit history backend (tests / embedders).
pub fn run_with_history(
    theme_mode: crate::shared::design::ThemeMode,
    history: Arc<dyn WelcomeHistory>,
) -> iced::Result {
    iced::application(
        move || {
            let recent_paths = history.load().unwrap_or_default();
            let state = WelcomeState::with_recent_paths(theme_mode, recent_paths);
            (
                WelcomeApp {
                    state,
                    history: history.clone(),
                    last_shift_press: None,
                },
                Task::none(),
            )
        },
        WelcomeApp::update,
        WelcomeApp::view,
    )
    .title(WelcomeApp::title)
    .theme(WelcomeApp::theme)
    .subscription(WelcomeApp::subscription)
    .window_size(iced::Size::new(720.0, 640.0))
    .run()
}

fn load_history() -> Arc<dyn WelcomeHistory> {
    match FileWelcomeHistory::from_project_dirs() {
        Ok(store) => Arc::new(store),
        Err(_) => Arc::new(InMemoryWelcomeHistory::new()),
    }
}

/// Iced application wrapper for the welcome screen.
pub struct WelcomeApp {
    state: WelcomeState,
    history: Arc<dyn WelcomeHistory>,
    last_shift_press: Option<Instant>,
}

impl WelcomeApp {
    fn update(&mut self, message: Msg) -> Task<Msg> {
        if let Msg::ShiftPressed = message {
            return self.handle_shift_pressed();
        }

        if let Msg::CloneSubmit = message {
            return self.start_clone();
        }

        if let Msg::NewFileDialogCompleted(Some(path)) = message {
            return self.finish_new_file(path);
        }

        let outcome = self.state.update(message);
        self.persist_history();

        match outcome {
            Outcome::ActionRequested(WelcomeItemId::NewFile) => WelcomeApp::pick_new_file(),
            Outcome::ActionRequested(WelcomeItemId::OpenProject) => WelcomeApp::pick_open_project(),
            Outcome::ActionRequested(WelcomeItemId::CloneRepository) => Task::none(),
            Outcome::WorkspaceOpened(path) => {
                self.state.status = Some(format!("Opened {}", path.display()));
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn handle_shift_pressed(&mut self) -> Task<Msg> {
        let now = Instant::now();
        let double_tap = self
            .last_shift_press
            .is_some_and(|previous| now.duration_since(previous) < Duration::from_millis(450));
        self.last_shift_press = Some(now);

        if double_tap {
            self.state.update(Msg::CommandPaletteToggle);
        }
        Task::none()
    }

    fn persist_history(&self) {
        if let Err(error) = self.history.save(&self.state.recent_paths) {
            eprintln!("failed to persist welcome history: {error}");
        }
    }

    fn finish_new_file(&mut self, path: PathBuf) -> Task<Msg> {
        match create_empty_file(&path) {
            Ok(()) => {
                let summary = format!("Created {}", path.display());
                let parent = path
                    .parent()
                    .map(std::path::Path::to_path_buf)
                    .unwrap_or(path);
                self.state.update(Msg::ActionCompleted {
                    path: parent,
                    summary,
                });
                self.persist_history();
            }
            Err(error) => {
                self.state.status = Some(error);
            }
        }
        Task::none()
    }

    fn pick_new_file() -> Task<Msg> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Create New File")
                    .save_file()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Msg::NewFileDialogCompleted,
        )
    }

    fn pick_open_project() -> Task<Msg> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Open Project")
                    .pick_folder()
                    .await
                    .map(|handle| handle.path().to_path_buf())
            },
            Msg::OpenProjectDialogCompleted,
        )
    }

    fn start_clone(&mut self) -> Task<Msg> {
        let url = self.state.clone_url.clone();
        Task::perform(
            async move {
                let parent = default_clone_parent();
                let destination = clone_destination(&parent, &url).ok_or_else(|| {
                    String::from("could not derive a repository name from the URL")
                })?;
                git_clone(&url, &destination).map(|()| destination)
            },
            Msg::CloneCompleted,
        )
    }

    fn view(&self) -> Element<'_, Msg> {
        view(&self.state)
    }

    fn subscription(&self) -> Subscription<Msg> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Shift),
                ..
            } => Some(Msg::ShiftPressed),
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            } => Some(Msg::CommandPaletteDismiss),
            keyboard::Event::KeyPressed { key, modifiers, .. } => shortcut_message(key, modifiers),
            _ => None,
        })
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

fn shortcut_message(key: keyboard::Key, mods: Modifiers) -> Option<Msg> {
    if !mods.command() {
        return None;
    }

    match key {
        keyboard::Key::Character(character) => match character.as_str() {
            "n" | "N" => Some(Msg::ItemPressed(WelcomeItemId::NewFile)),
            "o" | "O" => Some(Msg::ItemPressed(WelcomeItemId::OpenProject)),
            "1" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(0))),
            "2" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(1))),
            "3" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(2))),
            "4" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(3))),
            "5" => Some(Msg::ItemPressed(WelcomeItemId::RecentProject(4))),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod shortcut_tests {
    use super::*;

    #[test]
    fn command_n_requests_new_file() {
        let message = shortcut_message(keyboard::Key::Character("n".into()), Modifiers::COMMAND);
        assert_eq!(message, Some(Msg::ItemPressed(WelcomeItemId::NewFile)));
    }

    #[test]
    fn command_o_requests_open_project() {
        let message = shortcut_message(keyboard::Key::Character("o".into()), Modifiers::COMMAND);
        assert_eq!(message, Some(Msg::ItemPressed(WelcomeItemId::OpenProject)));
    }
}
