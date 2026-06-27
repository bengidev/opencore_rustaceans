//! Chat thread and empty state widgets.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{MouseArea, Space, column, container, image, row, scrollable, text};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_radius::control_radius;
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::chat_brand;
use super::chat_messages::ChatEvent;
use super::chat_model::{ChatMessage, ChatRole};
use super::chat_state::ChatState;

const LOGO_BOX: f32 = 72.0;
const LOGO_IMAGE: f32 = 56.0;
const BUBBLE_MAX_WIDTH: f32 = 520.0;

/// Render the chat body — empty state or message thread.
pub fn body(state: &ChatState, theme: OpenCoreTheme) -> Element<'_, ChatEvent> {
    if state.thread.is_empty() {
        container(empty_state(theme))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    } else {
        thread_view(state, theme)
    }
}

fn empty_state(theme: OpenCoreTheme) -> Element<'static, ChatEvent> {
    let headline = text("Chat with OpenRouter")
        .size(TypeRole::DisplayMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let divider = container(Space::new())
        .width(Length::Fixed(160.0))
        .height(Length::Fixed(1.0))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.border(BorderToken::Default))),
            ..Default::default()
        });

    let muted = text("OpenRouter models and custom actions will appear here.")
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        });

    let settings = MouseArea::new(
        text("Configure OpenRouter API key")
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    )
    .on_press(ChatEvent::ConfigureActionsPressed)
    .interaction(iced::mouse::Interaction::Pointer);

    column![
        logo_mark(theme),
        Space::new().height(Length::Fixed(SpacingToken::S5.value())),
        headline,
        Space::new().height(Length::Fixed(SpacingToken::S4.value())),
        divider,
        Space::new().height(Length::Fixed(SpacingToken::S4.value())),
        muted,
        Space::new().height(Length::Fixed(SpacingToken::S2.value())),
        settings,
    ]
    .align_x(Horizontal::Center)
    .width(Length::Fill)
    .into()
}

fn logo_mark(theme: OpenCoreTheme) -> Element<'static, ChatEvent> {
    let logo: Element<'static, ()> = image(chat_brand::openrouter_logo())
        .width(Length::Fixed(LOGO_IMAGE))
        .height(Length::Fixed(LOGO_IMAGE))
        .content_fit(iced::ContentFit::Contain)
        .into();

    container(logo.map(|()| ChatEvent::Noop))
        .id(iced::widget::Id::new("openrouter-logo"))
        .width(Length::Fixed(LOGO_BOX))
        .height(Length::Fixed(LOGO_BOX))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
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

fn thread_view(state: &ChatState, theme: OpenCoreTheme) -> Element<'_, ChatEvent> {
    let mut messages = column![]
        .spacing(SpacingToken::S3.value())
        .width(Length::Fill);

    for message in state.thread.messages() {
        messages = messages.push(message_bubble(message, theme, state.is_streaming));
    }

    scrollable(
        container(messages)
            .width(Length::Fill)
            .padding([SpacingToken::S4.value(), 0.0]),
    )
    .id(iced::widget::Id::new("chat-thread"))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn message_bubble(
    message: &ChatMessage,
    theme: OpenCoreTheme,
    is_streaming: bool,
) -> Element<'static, ChatEvent> {
    let is_user = message.role == ChatRole::User;

    let bubble_color = if is_user {
        theme.background(BackgroundToken::Elevated)
    } else {
        theme.background(BackgroundToken::Secondary)
    };

    let display = if message.content.is_empty() && is_streaming && !is_user {
        "…".to_owned()
    } else {
        message.content.clone()
    };

    let content = text(display)
        .size(TypeRole::BodyMd.size())
        .width(Length::Fill)
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let bubble = container(content)
        .padding([SpacingToken::S2.value(), SpacingToken::S3.value()])
        .width(Length::Shrink)
        .max_width(BUBBLE_MAX_WIDTH)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(bubble_color)),
            border: iced::Border {
                color: theme.border(BorderToken::Default),
                width: 1.0,
                radius: control_radius(),
            },
            ..Default::default()
        });

    if is_user {
        row![Space::new().width(Length::Fill), bubble]
            .width(Length::Fill)
            .into()
    } else {
        row![bubble, Space::new().width(Length::Fill)]
            .width(Length::Fill)
            .into()
    }
}
