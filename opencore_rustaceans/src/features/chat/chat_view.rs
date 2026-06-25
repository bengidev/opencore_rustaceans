//! Chat thread, empty state, and composer widgets.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text_input::Status as InputStatus;
use iced::widget::{
    MouseArea, Space, button, column, container, image, row, scrollable, text, text_input,
};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_tokens::{
    ActionToken, BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::chat_brand;
use super::chat_messages::ChatEvent;
use super::chat_model::{ChatMessage, ChatRole};
use super::chat_state::ChatState;
use super::chat_style::{control_radius, icon_button_style, text_input_style};

const LOGO_BOX: f32 = 72.0;
const LOGO_IMAGE: f32 = 56.0;
const ICON_BUTTON: f32 = 32.0;
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

/// Render the chat composer card contents.
pub fn composer<'a>(
    state: &'a ChatState,
    theme: OpenCoreTheme,
    has_api_key: bool,
    models_loading: bool,
    footer_leading: Element<'a, ChatEvent>,
) -> Element<'a, ChatEvent> {
    let can_send = composer_can_send(state, has_api_key, models_loading);

    let mut composer_column = column![].spacing(SpacingToken::S3.value());

    if !has_api_key {
        composer_column = composer_column.push(api_key_hint(theme));
    }

    let input = text_input("Type a message…", &state.draft)
        .on_input(ChatEvent::DraftChanged)
        .padding(SpacingToken::S3.value())
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme, status: InputStatus| text_input_style(theme, status));

    let attach = icon_button(theme, "+", IconButtonKind::Ghost, None);
    let send = icon_button(
        theme,
        "↑",
        if can_send {
            IconButtonKind::Primary
        } else {
            IconButtonKind::Disabled
        },
        can_send.then_some(ChatEvent::SendPressed),
    );

    let controls = row![
        footer_leading,
        Space::new().width(Length::Fill),
        attach,
        send
    ]
    .align_y(Vertical::Center)
    .spacing(SpacingToken::S2.value())
    .width(Length::Fill);

    composer_column = composer_column.push(input).push(controls);

    container(composer_column)
        .width(Length::Fill)
        .padding(SpacingToken::S4.value())
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

fn api_key_hint(theme: OpenCoreTheme) -> Element<'static, ChatEvent> {
    MouseArea::new(
        container(
            row![
                text("Add your OpenRouter API key to send messages")
                    .size(TypeRole::LabelMd.size())
                    .style(move |_t: &Theme| text::Style {
                        color: Some(theme.foreground(ForegroundToken::Secondary)),
                    }),
                Space::new().width(Length::Fill),
                text("›")
                    .size(TypeRole::BodyMd.size())
                    .style(move |_t: &Theme| text::Style {
                        color: Some(theme.foreground(ForegroundToken::Muted)),
                    }),
            ]
            .align_y(Vertical::Center)
            .width(Length::Fill),
        )
        .padding([SpacingToken::S2.value(), SpacingToken::S3.value()])
        .width(Length::Fill)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Tertiary),
            )),
            border: iced::Border {
                color: theme.border(BorderToken::Subtle),
                width: 1.0,
                radius: control_radius(),
            },
            ..Default::default()
        }),
    )
    .on_press(ChatEvent::ApiKeyHintPressed)
    .interaction(iced::mouse::Interaction::Pointer)
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconButtonKind {
    Primary,
    Ghost,
    Disabled,
}

fn icon_button(
    theme: OpenCoreTheme,
    label: &'static str,
    kind: IconButtonKind,
    event: Option<ChatEvent>,
) -> Element<'static, ChatEvent> {
    let (background, text_color, border_width) = match kind {
        IconButtonKind::Primary => (
            theme.action(ActionToken::Strong),
            theme.action(ActionToken::StrongText),
            0.0,
        ),
        IconButtonKind::Ghost | IconButtonKind::Disabled => (
            theme.background(BackgroundToken::Tertiary),
            theme.foreground(ForegroundToken::Secondary),
            1.0,
        ),
    };

    let content = container(text(label).size(16.0).style(move |_t: &Theme| text::Style {
        color: Some(text_color),
    }))
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center);

    let mut control = button(content)
        .padding(0)
        .width(Length::Fixed(ICON_BUTTON))
        .height(Length::Fixed(ICON_BUTTON))
        .style(move |_t: &Theme, status| {
            icon_button_style(
                theme,
                background,
                text_color,
                border_width,
                kind == IconButtonKind::Primary,
                status,
            )
        });

    if let Some(event) = event {
        control = control.on_press(event);
    }

    control.into()
}

fn composer_can_send(state: &ChatState, has_api_key: bool, models_loading: bool) -> bool {
    !state.draft.trim().is_empty() && has_api_key && !state.is_streaming && !models_loading
}
