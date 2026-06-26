//! Workspace shell view — header, chat panel, and overlays.

use iced::Element;
use iced::Length;
use iced::Padding;
use iced::Theme;
use iced::alignment::Vertical;
use iced::widget::{Space, button, column, container, row, stack, text};

use crate::features::chat::{ChatEvent, body, chip_button_style, composer, control_radius};
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::workspace_chat::workspace_message_from;
use super::workspace_messages::WorkspaceMessage;
use super::workspace_overlay::{WorkspaceOverlay, overlay_layer};
use super::workspace_state::WorkspaceState;

const HORIZONTAL_PAD: f32 = 24.0;

/// Render the workspace screen.
pub fn view(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;

    let chat_body = body(&state.chat, theme).map(workspace_message_from);

    let model_chip = model_chip_control(state);
    let directory_label = state
        .project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Project");
    let chat_composer = composer(
        &state.chat,
        theme,
        state.has_api_key,
        state.models_loading,
        model_chip,
        directory_label,
    )
    .map(workspace_message_from);

    let project_label = state
        .project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Project")
        .to_owned();

    let header = container(
        row![
            text(project_label)
                .size(TypeRole::LabelMd.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Secondary)),
                }),
            Space::new().width(Length::Fill),
            button(text("Close Project").size(TypeRole::LabelMd.size()).style(
                move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Primary)),
                }
            ),)
            .on_press(WorkspaceMessage::CloseProjectRequested)
            .padding([SpacingToken::S2.value(), SpacingToken::S3.value()])
            .style(move |_t: &Theme, status| chip_button_style(theme, status)),
        ]
        .align_y(Vertical::Center)
        .width(Length::Fill),
    )
    .padding([SpacingToken::S3.value(), HORIZONTAL_PAD])
    .width(Length::Fill)
    .style(move |_t: &Theme| container::Style {
        border: iced::Border {
            width: 1.0,
            color: theme.border(BorderToken::Subtle),
            ..Default::default()
        },
        ..Default::default()
    });

    let base: Element<'_, WorkspaceMessage> = container(
        column![
            header,
            container(chat_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([0.0, HORIZONTAL_PAD]),
            container(chat_composer).width(Length::Fill).padding(
                Padding::ZERO
                    .top(SpacingToken::Hairline.value())
                    .bottom(SpacingToken::S4.value())
                    .horizontal(HORIZONTAL_PAD),
            ),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Primary),
        )),
        ..Default::default()
    })
    .into();

    let mut layered = stack![base];
    if state.overlay != WorkspaceOverlay::None {
        layered = layered.push(overlay_layer(state));
    }

    layered.width(Length::Fill).height(Length::Fill).into()
}

const CHIP_GLYPH_BOX: f32 = 16.0;
const CHIP_GLYPH_SIZE: f32 = 12.0;
const STATUS_DOT: f32 = 6.0;

fn model_chip_control(state: &WorkspaceState) -> Element<'_, ChatEvent> {
    let theme = state.theme;
    let label = state.model_chip_label();
    let active = state.has_api_key && !state.models_loading;
    let text_color = if state.has_api_key {
        theme.foreground(ForegroundToken::Secondary)
    } else {
        theme.foreground(ForegroundToken::Muted)
    };

    let status_dot = container(Space::new())
        .width(Length::Fixed(STATUS_DOT))
        .height(Length::Fixed(STATUS_DOT))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(if active {
                theme.foreground(ForegroundToken::Primary)
            } else {
                theme.foreground(ForegroundToken::Muted)
            })),
            border: iced::Border {
                radius: control_radius(),
                ..Default::default()
            },
            ..Default::default()
        });

    let eye = chip_glyph("◎", theme.foreground(ForegroundToken::Muted));

    let chevron: Element<'_, ChatEvent> = if state.has_api_key && !state.models_loading {
        chip_glyph("⌄", theme.foreground(ForegroundToken::Muted))
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let content = container(
        row![
            status_dot,
            eye,
            text(label)
                .size(TypeRole::MonoSm.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(text_color),
                }),
            chevron,
        ]
        .align_y(Vertical::Center)
        .spacing(SpacingToken::S1.value()),
    )
    .padding([SpacingToken::S1.value(), SpacingToken::S3.value()])
    .align_y(Vertical::Center);

    let chip = button(content)
        .padding(0)
        .style(move |_t: &Theme, status| chip_button_style(theme, status));

    if state.models_loading {
        chip.into()
    } else {
        chip.on_press(ChatEvent::ModelChipPressed).into()
    }
}

fn chip_glyph(label: &'static str, color: iced::Color) -> Element<'static, ChatEvent> {
    use iced::alignment::Horizontal;

    container(
        text(label)
            .size(CHIP_GLYPH_SIZE)
            .style(move |_t: &Theme| text::Style { color: Some(color) }),
    )
    .width(Length::Fixed(CHIP_GLYPH_BOX))
    .height(Length::Fixed(CHIP_GLYPH_BOX))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}
