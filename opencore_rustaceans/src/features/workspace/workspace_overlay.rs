//! Modal overlays for API key settings and close-project confirmation.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::Vertical;
use iced::widget::{
    MouseArea, Space, button, column, container, row, scrollable, stack, text, text_input,
};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::workspace_messages::WorkspaceMessage;
use super::workspace_state::WorkspaceState;
use crate::features::chat::{
    chip_button_style, control_radius, primary_button_style, text_input_style,
};

/// Active overlay on the workspace screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceOverlay {
    #[default]
    None,
    ApiKeySettings,
    ModelPicker,
    CloseProjectConfirm,
}

const PANEL_WIDTH: f32 = 420.0;
const MODEL_PANEL_WIDTH: f32 = 480.0;
const MODEL_PANEL_MAX_HEIGHT: f32 = 360.0;

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
        WorkspaceOverlay::ModelPicker => modal_overlay(
            model_picker_panel(state, theme),
            WorkspaceMessage::ModelPickerDismiss,
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
    theme: OpenCoreTheme,
) -> Element<'_, WorkspaceMessage> {
    let scrim = scrim_color(theme);
    stack![
        MouseArea::new(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_t: &Theme| container::Style {
                    background: Some(iced::Background::Color(scrim)),
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
    let title = text("OpenRouter API Key")
        .size(TypeRole::BodyLg.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let hint = text(
        "Enter your OpenRouter API key to load models and send messages. Keys are stored on this device (Keychain when available, otherwise a local app-data file with restricted permissions). The key is never shown again after saving.",
    )
        .size(TypeRole::LabelMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Secondary)),
        });

    let field = text_input("sk-or-…", &state.api_key_input)
        .on_input(WorkspaceMessage::ApiKeyInputChanged)
        .secure(true)
        .padding(SpacingToken::S3.value())
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme, status| text_input_style(theme, status));

    let mut fields = column![title, hint, field].spacing(SpacingToken::S4.value());
    if let Some(status) = &state.api_key_status {
        fields = fields.push(text(status.clone()).size(TypeRole::LabelMd.size()).style(
            move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            },
        ));
    }

    let save = button(text("Save").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeySave)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()])
        .style(move |_t: &Theme, status| primary_button_style(theme, status));

    let remove = button(text("Remove").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeyRemove)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()])
        .style(move |_t: &Theme, status| chip_button_style(theme, status));

    let done = button(text("Done").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::ApiKeyDismiss)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()])
        .style(move |_t: &Theme, status| chip_button_style(theme, status));

    container(
        column![
            fields,
            row![save, remove, Space::new().width(Length::Fill), done]
                .spacing(SpacingToken::S2.value())
                .align_y(Vertical::Center),
        ]
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
            radius: control_radius(),
        },
        ..Default::default()
    })
    .into()
}

fn model_picker_panel(
    state: &WorkspaceState,
    theme: OpenCoreTheme,
) -> Element<'_, WorkspaceMessage> {
    let title = text("OpenRouter Models")
        .size(TypeRole::BodyLg.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let field = text_input("Search models…", &state.model_query)
        .on_input(WorkspaceMessage::ModelPickerQueryChanged)
        .padding(SpacingToken::S3.value())
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme, status| text_input_style(theme, status));

    let models = state.filtered_models();
    let mut rows = column![].spacing(SpacingToken::S1.value());
    if models.is_empty() {
        rows = rows.push(
            text("No models match your search.")
                .size(TypeRole::LabelMd.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                }),
        );
    } else {
        for (index, option) in models.iter().enumerate() {
            let selected = option.id == state.model;
            let name = text(option.name.clone())
                .size(TypeRole::LabelMd.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Primary)),
                });
            let id =
                text(option.id.clone())
                    .size(TypeRole::MonoXs.size())
                    .style(move |_t: &Theme| text::Style {
                        color: Some(theme.foreground(ForegroundToken::Muted)),
                    });
            let row_content = row![
                column![name, id].spacing(SpacingToken::S1.value()),
                Space::new().width(Length::Fill),
                if selected {
                    Element::from(text("✓").size(TypeRole::LabelMd.size()).style(
                        move |_t: &Theme| text::Style {
                            color: Some(theme.foreground(ForegroundToken::Secondary)),
                        },
                    ))
                } else {
                    Space::new().width(Length::Fixed(12.0)).into()
                },
            ]
            .align_y(Vertical::Center)
            .width(Length::Fill);

            rows = rows.push(
                button(row_content)
                    .width(Length::Fill)
                    .padding([SpacingToken::S2.value(), SpacingToken::S3.value()])
                    .on_press(WorkspaceMessage::ModelPickerSelect(index))
                    .style(move |_t: &Theme, status| model_row_style(theme, selected, status)),
            );
        }
    }

    let list = scrollable(
        container(rows)
            .width(Length::Fill)
            .padding(SpacingToken::S1.value()),
    )
    .height(Length::Fixed(MODEL_PANEL_MAX_HEIGHT));

    container(
        column![title, field, list]
            .spacing(SpacingToken::S3.value())
            .width(Length::Fill),
    )
    .width(Length::Fixed(MODEL_PANEL_WIDTH))
    .padding(SpacingToken::S5.value())
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Elevated),
        )),
        border: iced::Border {
            color: theme.border(BorderToken::Default),
            width: 1.0,
            radius: control_radius(),
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
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()])
        .style(move |_t: &Theme, status| chip_button_style(theme, status));

    let confirm = button(text("Close project").size(TypeRole::LabelMd.size()))
        .on_press(WorkspaceMessage::CloseProjectConfirm)
        .padding([SpacingToken::S2.value(), SpacingToken::S4.value()])
        .style(move |_t: &Theme, status| primary_button_style(theme, status));

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
            radius: control_radius(),
        },
        ..Default::default()
    })
    .into()
}

fn with_alpha(color: iced::Color, alpha: f32) -> iced::Color {
    iced::Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

fn scrim_color(theme: OpenCoreTheme) -> iced::Color {
    with_alpha(theme.background(BackgroundToken::Primary), 0.72)
}

fn model_row_style(theme: OpenCoreTheme, selected: bool, status: button::Status) -> button::Style {
    let background = if selected {
        theme.background(BackgroundToken::Secondary)
    } else {
        theme.background(BackgroundToken::Primary)
    };
    let base = button::Style {
        background: Some(iced::Background::Color(background)),
        text_color: theme.foreground(ForegroundToken::Primary),
        border: iced::Border {
            radius: control_radius(),
            width: 1.0,
            color: theme.border(BorderToken::Subtle),
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Tertiary),
            )),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Secondary),
            )),
            ..base
        },
        _ => base,
    }
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
