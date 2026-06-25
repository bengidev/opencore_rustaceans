//! Modal overlays for API key settings and close-project confirmation.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::Vertical;
use iced::widget::{MouseArea, Space, button, column, container, row, stack, text, text_input};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, RadiusToken, SpacingToken, TypeRole,
};

use super::workspace_messages::WorkspaceMessage;
use super::workspace_state::WorkspaceState;

/// Active overlay on the workspace screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceOverlay {
    #[default]
    None,
    ApiKeySettings,
    CloseProjectConfirm,
}

const PANEL_WIDTH: f32 = 420.0;

/// Render the active workspace overlay above the chat canvas.
pub fn overlay_layer(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;
    match state.overlay {
        WorkspaceOverlay::None => Space::new().into(),
        WorkspaceOverlay::ApiKeySettings => modal_overlay(
            api_key_settings_panel(state, theme),
            WorkspaceMessage::ApiKeyDismiss,
            theme,
        ),
        WorkspaceOverlay::CloseProjectConfirm => modal_overlay(
            close_project_panel(theme),
            WorkspaceMessage::CloseProjectCancel,
            theme,
        ),
    }
}

fn modal_overlay(
    panel: Element<'_, WorkspaceMessage>,
    dismiss: WorkspaceMessage,
    _theme: OpenCoreTheme,
) -> Element<'_, WorkspaceMessage> {
    stack![
        MouseArea::new(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_t: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.0, 0.0, 0.0, 0.55,
                    ))),
                    ..Default::default()
                }),
        )
        .on_press(dismiss)
        .interaction(iced::mouse::Interaction::Pointer),
        column![
            Space::new().height(Length::Fill),
            row![
                Space::new().width(Length::Fill),
                panel,
                Space::new().width(Length::Fill),
            ],
            Space::new().height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn api_key_settings_panel(
    state: &WorkspaceState,
    theme: OpenCoreTheme,
) -> Element<'_, WorkspaceMessage> {
    let title = text("API Key Settings")
        .size(TypeRole::BodyLg.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let hint = text("Enter your OpenRouter API key. It is stored securely and never shown again.")
        .size(TypeRole::LabelMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Secondary)),
        });

    let field = text_input("sk-or-…", &state.api_key_input)
        .on_input(WorkspaceMessage::ApiKeyInputChanged)
        .secure(true)
        .padding(SpacingToken::S3.value())
        .size(TypeRole::BodyMd.size());

    let save = button(text("Save").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeySave)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    let remove = button(text("Remove").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeyRemove)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    let done = button(text("Done").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeyDismiss)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    container(
        column![
            title,
            hint,
            field,
            row![save, remove, Space::new().width(Length::Fill), done]
                .spacing(SpacingToken::S2.value())
                .align_y(Vertical::Center),
        ]
        .spacing(SpacingToken::S4.value())
        .width(Length::Fill),
    )
    .width(Length::Fixed(PANEL_WIDTH))
    .padding(SpacingToken::S5.value())
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Elevated),
        )),
        border: iced::Border {
            color: theme.border(BorderToken::Default),
            width: 1.0,
            radius: RadiusToken::Md.value().into(),
        },
        ..Default::default()
    })
    .into()
}

fn close_project_panel(theme: OpenCoreTheme) -> Element<'static, WorkspaceMessage> {
    let title = text("Close project?")
        .size(TypeRole::BodyLg.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let body = text(
        "You will return to the welcome screen. Chat history for this session will be cleared.",
    )
    .size(TypeRole::LabelMd.size())
    .style(move |_t: &Theme| text::Style {
        color: Some(theme.foreground(ForegroundToken::Secondary)),
    });

    let cancel = button(text("Cancel").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::CloseProjectCancel)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    let confirm = button(text("Close project").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::CloseProjectConfirm)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    container(
        column![
            title,
            body,
            row![Space::new().width(Length::Fill), cancel, confirm]
                .spacing(SpacingToken::S2.value())
                .align_y(Vertical::Center),
        ]
        .spacing(SpacingToken::S4.value())
        .width(Length::Fill),
    )
    .width(Length::Fixed(PANEL_WIDTH))
    .padding(SpacingToken::S5.value())
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Elevated),
        )),
        border: iced::Border {
            color: theme.border(BorderToken::Default),
            width: 1.0,
            radius: RadiusToken::Md.value().into(),
        },
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::shared::design::ThemeMode;

    #[test]
    fn default_overlay_is_none() {
        let state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        assert_eq!(state.overlay, WorkspaceOverlay::None);
    }

    #[test]
    fn api_key_save_requires_non_empty_input() {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.overlay = WorkspaceOverlay::ApiKeySettings;
        state.api_key_input = String::from("   ");
        let outcome = state.update(WorkspaceMessage::ApiKeySave);
        assert_eq!(
            outcome,
            super::super::workspace_outcome::WorkspaceOutcome::None
        );
        assert_eq!(state.overlay, WorkspaceOverlay::ApiKeySettings);
        assert!(!state.has_api_key);
    }
}
