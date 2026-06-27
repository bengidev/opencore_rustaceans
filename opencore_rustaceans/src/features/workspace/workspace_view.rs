//! Workspace shell view — header, chat panel, and overlays.

use iced::Element;
use iced::Length;
use iced::Padding;
use iced::Theme;
use iced::alignment::Vertical;
use iced::widget::{Space, button, column, container, row, stack, text};

use crate::features::chat::{ChatEvent, DEFAULT_TOKEN_BUDGET, body, composer};
use crate::shared::design::chip_button_style;
use crate::shared::design::design_chip::selector_chip;
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::workspace_chat::workspace_message_from;
use super::workspace_messages::WorkspaceMessage;
use super::workspace_overlay::{WorkspaceOverlay, overlay_layer};
use super::workspace_scope::ComposerScope;
use super::workspace_state::WorkspaceState;

const HORIZONTAL_PAD: f32 = 24.0;

/// Render the workspace screen.
pub fn view(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;

    let chat_body = body(&state.chat, theme).map(workspace_message_from);

    let tokens = state.chat.token_estimate(DEFAULT_TOKEN_BUDGET);
    let chat_composer = composer(
        &state.chat,
        theme,
        state.has_api_key,
        state.models_loading,
        selector_row(state),
        state.composer_context_label(),
        tokens,
    )
    .map(workspace_message_from);

    let header = container(
        row![
            text(state.project_basename())
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
            .style(move |_t: &Theme, status| chip_button_style(theme, status, false)),
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

fn selector_row(state: &WorkspaceState) -> Element<'_, ChatEvent> {
    row![
        model_chip_control(state),
        scope_chip(
            state,
            ComposerScope::Sandbox,
            "□",
            "Sandbox",
            false,
            true,
            ChatEvent::SandboxScopePressed,
        ),
        scope_chip(
            state,
            ComposerScope::Folder,
            "▤",
            "Folder",
            true,
            false,
            ChatEvent::FolderScopePressed,
        ),
    ]
    .spacing(SpacingToken::S2.value())
    .align_y(Vertical::Center)
    .into()
}

fn scope_chip<'a>(
    state: &'a WorkspaceState,
    scope: ComposerScope,
    icon: &'static str,
    label: &'static str,
    show_chevron: bool,
    show_status_dot: bool,
    event: ChatEvent,
) -> Element<'a, ChatEvent> {
    let theme = state.theme;
    let active = state.composer_scope == scope;
    let text_color = if active {
        theme.foreground(ForegroundToken::Primary)
    } else {
        theme.foreground(ForegroundToken::Secondary)
    };

    let label = text(label)
        .size(TypeRole::LabelMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(text_color),
        })
        .into();

    let trailing = show_chevron.then_some("⌄");

    selector_chip(
        theme,
        active,
        Some(icon),
        label,
        trailing,
        show_status_dot,
        Some(event),
    )
}

fn model_chip_control(state: &WorkspaceState) -> Element<'_, ChatEvent> {
    let theme = state.theme;
    let label_text = state.model_chip_label();
    let active = state.has_api_key && !state.models_loading;
    let text_color = if state.has_api_key {
        if active {
            theme.foreground(ForegroundToken::Primary)
        } else {
            theme.foreground(ForegroundToken::Secondary)
        }
    } else {
        theme.foreground(ForegroundToken::Muted)
    };

    let label = text(label_text)
        .size(TypeRole::MonoSm.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(text_color),
        })
        .into();

    let trailing = if state.has_api_key && !state.models_loading {
        Some("⌄")
    } else {
        None
    };

    let on_press = if state.models_loading {
        None
    } else {
        Some(ChatEvent::ModelChipPressed)
    };

    selector_chip(theme, active, Some("◎"), label, trailing, true, on_press)
}
