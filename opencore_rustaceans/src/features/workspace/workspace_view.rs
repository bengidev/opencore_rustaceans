//! Workspace chat view — empty state, thread, and composer.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    MouseArea, Space, button, column, container, row, scrollable, stack, text, text_input,
};

use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, RadiusToken, SpacingToken, TypeRole,
};

use super::workspace_messages::WorkspaceMessage;
use super::workspace_model::{ChatMessage, ChatRole};
use super::workspace_overlay::{WorkspaceOverlay, overlay_layer};
use super::workspace_state::WorkspaceState;

const LOGO_BOX: f32 = 48.0;
const COMPOSER_MAX_WIDTH: f32 = 720.0;
const THREAD_MAX_WIDTH: f32 = 720.0;

/// Render the workspace chat screen.
pub fn view(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;

    let body: Element<'_, WorkspaceMessage> = if state.thread.is_empty() {
        empty_state(state)
    } else {
        thread_view(state)
    };

    let project_label = state
        .project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Project")
        .to_owned();

    let base: Element<'_, WorkspaceMessage> = container(
        column![
            container(
                row![
                    text(project_label)
                        .size(TypeRole::LabelMd.size())
                        .style(move |_t: &Theme| text::Style {
                            color: Some(theme.foreground(ForegroundToken::Secondary)),
                        }),
                    Space::new().width(Length::Fill),
                    button(text("Close Project").size(TypeRole::LabelMd.size()))
                        .on_press(WorkspaceMessage::CloseProjectRequested)
                        .padding([SpacingToken::S1.value(), SpacingToken::S3.value()]),
                ]
                .align_y(Vertical::Center)
                .width(Length::Fill),
            )
            .padding([SpacingToken::S3.value(), SpacingToken::S4.value()]),
            container(body)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center),
            composer(state),
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

fn empty_state(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;

    let mark = container(Space::new())
        .width(Length::Fixed(LOGO_BOX))
        .height(Length::Fixed(LOGO_BOX))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Elevated),
            )),
            border: iced::Border {
                color: theme.border(BorderToken::Default),
                width: 1.0,
                radius: RadiusToken::Sm.value().into(),
            },
            ..Default::default()
        });

    let headline = text("Chat with OpenCore")
        .size(TypeRole::DisplayMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let divider = container(Space::new())
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(1.0))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.border(BorderToken::Default))),
            ..Default::default()
        });

    let muted = text("Your custom actions will appear here.")
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        });

    let settings = MouseArea::new(
        text("Configure actions in Settings")
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Accent)),
            }),
    )
    .on_press(WorkspaceMessage::ConfigureActionsPressed)
    .interaction(iced::mouse::Interaction::Pointer);

    column![
        mark,
        Space::new().height(Length::Fixed(SpacingToken::S4.value())),
        headline,
        Space::new().height(Length::Fixed(SpacingToken::S3.value())),
        divider,
        Space::new().height(Length::Fixed(SpacingToken::S3.value())),
        muted,
        Space::new().height(Length::Fixed(SpacingToken::S2.value())),
        settings,
    ]
    .align_x(Horizontal::Center)
    .width(Length::Fill)
    .into()
}

fn thread_view(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;
    let mut messages = column![]
        .spacing(SpacingToken::S3.value())
        .width(Length::Fill);

    for message in state.thread.messages() {
        messages = messages.push(message_bubble(message, theme));
    }

    scrollable(
        container(messages)
            .width(Length::Fixed(THREAD_MAX_WIDTH))
            .padding(SpacingToken::S4.value()),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn message_bubble(
    message: &ChatMessage,
    theme: crate::shared::design::OpenCoreTheme,
) -> Element<'static, WorkspaceMessage> {
    let (align, bubble_color) = match message.role {
        ChatRole::User => (
            Horizontal::Right,
            theme.background(BackgroundToken::Elevated),
        ),
        ChatRole::Assistant => (
            Horizontal::Left,
            theme.background(BackgroundToken::Secondary),
        ),
    };

    let content = text(message.content.clone())
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let bubble = container(content)
        .padding(SpacingToken::S3.value())
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(bubble_color)),
            border: iced::Border {
                radius: RadiusToken::Md.value().into(),
                ..Default::default()
            },
            ..Default::default()
        });

    if align == Horizontal::Right {
        row![Space::new().width(Length::Fill), bubble]
            .width(Length::Fill)
            .into()
    } else {
        row![bubble, Space::new().width(Length::Fill)]
            .width(Length::Fill)
            .into()
    }
}

fn composer(state: &WorkspaceState) -> Element<'_, WorkspaceMessage> {
    let theme = state.theme;

    let mut composer_column = column![]
        .spacing(SpacingToken::S2.value())
        .width(Length::Fill);

    if !state.has_api_key {
        composer_column = composer_column.push(
            MouseArea::new(
                container(
                    text("Add your OpenRouter API key to send messages")
                        .size(TypeRole::LabelMd.size())
                        .style(move |_t: &Theme| text::Style {
                            color: Some(theme.foreground(ForegroundToken::Accent)),
                        }),
                )
                .padding(SpacingToken::S2.value())
                .width(Length::Fill)
                .style(move |_t: &Theme| container::Style {
                    background: Some(iced::Background::Color(
                        theme.background(BackgroundToken::GalaxyTint),
                    )),
                    border: iced::Border {
                        radius: RadiusToken::Sm.value().into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            )
            .on_press(WorkspaceMessage::ApiKeyHintPressed)
            .interaction(iced::mouse::Interaction::Pointer),
        );
    }

    let model_chip = container(
        text(state.model.clone())
            .size(TypeRole::MonoSm.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    )
    .padding([SpacingToken::S1.value(), SpacingToken::S2.value()])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        border: iced::Border {
            radius: RadiusToken::Pill.value().into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let input = text_input("Type a message…", &state.draft)
        .on_input(WorkspaceMessage::DraftChanged)
        .padding(SpacingToken::S3.value())
        .size(TypeRole::BodyMd.size());

    let attach = button(text("+").size(TypeRole::BodyLg.size())).padding(SpacingToken::S2.value());

    let send = button(text("↑").size(TypeRole::BodyLg.size()))
        .on_press(WorkspaceMessage::SendPressed)
        .padding(SpacingToken::S2.value());

    let controls = row![model_chip, Space::new().width(Length::Fill), attach, send]
        .align_y(Vertical::Center)
        .spacing(SpacingToken::S2.value())
        .width(Length::Fill);

    composer_column = composer_column.push(input).push(controls);

    container(composer_column)
        .width(Length::Fixed(COMPOSER_MAX_WIDTH))
        .padding(SpacingToken::S4.value())
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Elevated),
            )),
            border: iced::Border {
                color: theme.border(BorderToken::Subtle),
                width: 1.0,
                radius: RadiusToken::Lg.value().into(),
            },
            ..Default::default()
        })
        .into()
}
