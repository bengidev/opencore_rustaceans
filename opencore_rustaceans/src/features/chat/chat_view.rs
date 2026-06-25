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
    ActionToken, BackgroundToken, BorderToken, ForegroundToken, RadiusToken, SpacingToken, TypeRole,
};

use super::chat_brand;
use super::chat_messages::ChatEvent;
use super::chat_model::{ChatMessage, ChatRole};
use super::chat_state::ChatState;
use super::chat_style::{
    chip_button_style, composer_input_style, control_radius, icon_button_style, surface_radius,
};

const LOGO_BOX: f32 = 72.0;
const LOGO_IMAGE: f32 = 56.0;
const ICON_BUTTON: f32 = 28.0;
const CHIP_GLYPH_BOX: f32 = 16.0;
const ICON_GLYPH_SIZE: f32 = 14.0;
const CHIP_GLYPH_SIZE: f32 = 12.0;
const BUBBLE_MAX_WIDTH: f32 = 520.0;
const TOKEN_BAR_WIDTH: f32 = 112.0;
const TOKEN_BUDGET: u32 = 128_000;
const STATUS_DOT: f32 = 6.0;

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
    model_chip: Element<'a, ChatEvent>,
    selected_directory: &'a str,
) -> Element<'a, ChatEvent> {
    let can_send = composer_can_send(state, has_api_key, models_loading);

    let mut composer_column = column![].spacing(SpacingToken::Hairline.value());

    if !has_api_key {
        composer_column = composer_column.push(api_key_hint(theme));
    }

    composer_column = composer_column
        .push(config_toolbar(
            theme,
            model_chip,
            selected_directory,
            estimated_tokens(state),
        ))
        .push(composer_card(state, theme, can_send));

    composer_column.width(Length::Fill).into()
}

fn config_toolbar<'a>(
    theme: OpenCoreTheme,
    model_chip: Element<'a, ChatEvent>,
    selected_directory: &'a str,
    token_count: u32,
) -> Element<'a, ChatEvent> {
    let selectors = row![
        model_chip,
        mock_pill(theme, true, "□", "Sandbox", false),
        mock_pill(theme, false, "▤", "Folder", true),
    ]
    .spacing(SpacingToken::S2.value())
    .align_y(Vertical::Center);

    let directory = row![
        text("⌗")
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            }),
        text(selected_directory)
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    ]
    .spacing(SpacingToken::S1.value())
    .align_y(Vertical::Center);

    let token_meter = column![token_meter_bar(theme, token_count), token_meter_label(theme, token_count)]
        .spacing(SpacingToken::S1.value())
        .align_x(Horizontal::Right);

    let trailing = column![directory, token_meter]
        .spacing(SpacingToken::S1.value())
        .align_x(Horizontal::Right);

    row![selectors, Space::new().width(Length::Fill), trailing]
        .align_y(Vertical::Top)
        .width(Length::Fill)
        .into()
}

fn composer_card(state: &ChatState, theme: OpenCoreTheme, can_send: bool) -> Element<'_, ChatEvent> {
    let input = text_input("Message or attach files…", &state.draft)
        .on_input(ChatEvent::DraftChanged)
        .padding([SpacingToken::S3.value(), SpacingToken::S4.value()])
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme, status: InputStatus| composer_input_style(theme, status));

    let attach = icon_button(theme, "⌁", IconButtonKind::Ghost, None);
    let commands = icon_button(theme, "/", IconButtonKind::Ghost, None);
    let voice = icon_button(theme, "◌", IconButtonKind::Ghost, None);

    let send_hint = text("↵ to send")
        .size(TypeRole::LabelMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        });

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

    let action_bar = row![
        attach,
        commands,
        voice,
        Space::new().width(Length::Fill),
        send_hint,
        send,
    ]
    .align_y(Vertical::Center)
    .spacing(SpacingToken::S2.value())
    .width(Length::Fill)
    .padding([SpacingToken::S2.value(), SpacingToken::S4.value()]);

    container(column![input, action_bar].width(Length::Fill))
        .width(Length::Fill)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Elevated),
            )),
            border: iced::Border {
                color: theme.border(BorderToken::Default),
                width: 1.0,
                radius: surface_radius(),
            },
            ..Default::default()
        })
        .into()
}

fn mock_pill(
    theme: OpenCoreTheme,
    active: bool,
    icon: &'static str,
    label: &'static str,
    show_chevron: bool,
) -> Element<'static, ChatEvent> {
    let text_color = theme.foreground(ForegroundToken::Secondary);
    let icon_color = theme.foreground(ForegroundToken::Muted);

    let mut content = row![status_dot(theme, active)]
        .spacing(SpacingToken::S1.value())
        .align_y(Vertical::Center);

    content = content
        .push(centered_glyph(icon, icon_color, CHIP_GLYPH_BOX, CHIP_GLYPH_SIZE))
        .push(
            text(label)
                .size(TypeRole::LabelMd.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(text_color),
                }),
        );

    if show_chevron {
        content = content.push(centered_glyph(
            "⌄",
            theme.foreground(ForegroundToken::Muted),
            CHIP_GLYPH_BOX,
            CHIP_GLYPH_SIZE,
        ));
    }

    button(
        container(content)
            .padding([SpacingToken::S1.value(), SpacingToken::S3.value()])
            .align_y(Vertical::Center)
            .align_x(Horizontal::Left),
    )
    .padding(0)
    .on_press(ChatEvent::Noop)
    .style(move |_t: &Theme, status| chip_button_style(theme, status))
    .into()
}

fn status_dot(theme: OpenCoreTheme, active: bool) -> Element<'static, ChatEvent> {
    let color = if active {
        theme.foreground(ForegroundToken::Primary)
    } else {
        theme.foreground(ForegroundToken::Muted)
    };

    container(Space::new())
        .width(Length::Fixed(STATUS_DOT))
        .height(Length::Fixed(STATUS_DOT))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: control_radius(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn token_meter_bar(theme: OpenCoreTheme, used: u32) -> Element<'static, ChatEvent> {
    let ratio = (used as f32 / TOKEN_BUDGET as f32).clamp(0.02, 1.0);
    let filled = TOKEN_BAR_WIDTH * ratio;

    let fill = container(Space::new())
        .width(Length::Fixed(filled))
        .height(Length::Fixed(3.0))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.foreground(ForegroundToken::Secondary),
            )),
            border: iced::Border {
                radius: RadiusToken::Xs.value().into(),
                ..Default::default()
            },
            ..Default::default()
        });

    container(fill)
        .width(Length::Fixed(TOKEN_BAR_WIDTH))
        .height(Length::Fixed(3.0))
        .align_x(Horizontal::Left)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Tertiary),
            )),
            border: iced::Border {
                radius: RadiusToken::Xs.value().into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn token_meter_label(theme: OpenCoreTheme, used: u32) -> Element<'static, ChatEvent> {
    let label = format!(
        "{} / {} tokens",
        format_token_count(used),
        format_token_count(TOKEN_BUDGET)
    );

    text(label)
        .size(TypeRole::MonoSm.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        })
        .into()
}

fn format_token_count(count: u32) -> String {
    if count >= 10_000 {
        format!("{:.0}k", count as f32 / 1000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f32 / 1000.0)
    } else {
        format!("~{count}")
    }
}

fn estimated_tokens(state: &ChatState) -> u32 {
    let chars: usize = state
        .thread
        .messages()
        .iter()
        .map(|message| message.content.len())
        .sum::<usize>()
        + state.draft.len();
    ((chars as u32) / 4).max(1)
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

    let content = centered_glyph(label, text_color, ICON_BUTTON, ICON_GLYPH_SIZE);

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

fn centered_glyph(
    label: &'static str,
    color: iced::Color,
    box_size: f32,
    font_size: f32,
) -> Element<'static, ChatEvent> {
    container(
        text(label)
            .size(font_size)
            .style(move |_t: &Theme| text::Style { color: Some(color) }),
    )
    .width(Length::Fixed(box_size))
    .height(Length::Fixed(box_size))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}

fn composer_can_send(state: &ChatState, has_api_key: bool, models_loading: bool) -> bool {
    !state.draft.trim().is_empty() && has_api_key && !state.is_streaming && !models_loading
}
