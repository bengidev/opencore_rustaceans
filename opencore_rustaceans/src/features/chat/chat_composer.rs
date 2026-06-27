//! Chat composer — config toolbar, input card, and token meter.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text_input::Status as InputStatus;
use iced::widget::{MouseArea, Space, button, column, container, row, text, text_input};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_chip::centered_glyph;
use crate::shared::design::design_controls::icon_button_style;
use crate::shared::design::design_radius::{control_radius, surface_radius};
use crate::shared::design::design_tokens::{
    ActionToken, BackgroundToken, BorderToken, ForegroundToken, RadiusToken, SpacingToken, TypeRole,
};

use super::chat_messages::ChatEvent;
use super::chat_state::{ChatState, TokenEstimate};
use super::chat_style::composer_input_style;

const ICON_BUTTON: f32 = 28.0;
const ICON_GLYPH_SIZE: f32 = 14.0;
const TOKEN_BAR_WIDTH: f32 = 112.0;

/// Render the chat composer card contents.
pub fn composer<'a>(
    state: &'a ChatState,
    theme: OpenCoreTheme,
    has_api_key: bool,
    models_loading: bool,
    selector_row: Element<'a, ChatEvent>,
    context_label: &'a str,
    tokens: TokenEstimate,
) -> Element<'a, ChatEvent> {
    let can_send = composer_can_send(state, has_api_key, models_loading);

    let mut composer_column = column![].spacing(SpacingToken::Hairline.value());

    if !has_api_key {
        composer_column = composer_column.push(api_key_hint(theme));
    }

    composer_column = composer_column
        .push(config_toolbar(theme, selector_row, context_label, tokens))
        .push(composer_card(state, theme, can_send));

    composer_column.width(Length::Fill).into()
}

fn config_toolbar<'a>(
    theme: OpenCoreTheme,
    selector_row: Element<'a, ChatEvent>,
    context_label: &'a str,
    tokens: TokenEstimate,
) -> Element<'a, ChatEvent> {
    let directory = row![
        text("⌗")
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            }),
        text(context_label)
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    ]
    .spacing(SpacingToken::S1.value())
    .align_y(Vertical::Center);

    let token_meter = column![
        token_meter_bar(theme, tokens),
        token_meter_label(theme, tokens)
    ]
    .spacing(SpacingToken::S1.value())
    .align_x(Horizontal::Right);

    let trailing = column![directory, token_meter]
        .spacing(SpacingToken::S1.value())
        .align_x(Horizontal::Right);

    row![selector_row, Space::new().width(Length::Fill), trailing]
        .align_y(Vertical::Top)
        .width(Length::Fill)
        .into()
}

fn composer_card(
    state: &ChatState,
    theme: OpenCoreTheme,
    can_send: bool,
) -> Element<'_, ChatEvent> {
    let input = text_input("Message or attach files…", &state.draft)
        .on_input(ChatEvent::DraftChanged)
        .padding([SpacingToken::S3.value(), SpacingToken::S4.value()])
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme, status: InputStatus| composer_input_style(theme, status));

    let muted = theme.foreground(ForegroundToken::Muted);
    let attach = upcoming_action_glyph("⌁", muted);
    let commands = upcoming_action_glyph("/", muted);
    let voice = upcoming_action_glyph("◌", muted);

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

fn token_meter_bar(theme: OpenCoreTheme, tokens: TokenEstimate) -> Element<'static, ChatEvent> {
    let ratio = if tokens.budget == 0 || tokens.used == 0 {
        0.0
    } else {
        (tokens.used as f32 / tokens.budget as f32).clamp(0.0, 1.0)
    };
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

fn token_meter_label(theme: OpenCoreTheme, tokens: TokenEstimate) -> Element<'static, ChatEvent> {
    let label = format!(
        "~{} / {} tokens (est.)",
        format_token_count(tokens.used),
        format_token_count(tokens.budget),
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
        format!("{count}")
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
    Disabled,
}

fn icon_button(
    theme: OpenCoreTheme,
    label: &'static str,
    kind: IconButtonKind,
    event: Option<ChatEvent>,
) -> Element<'static, ChatEvent> {
    let (background, text_color, border_width, primary) = match kind {
        IconButtonKind::Primary => (
            theme.action(ActionToken::Strong),
            theme.action(ActionToken::StrongText),
            0.0,
            true,
        ),
        IconButtonKind::Disabled => (
            theme.background(BackgroundToken::Tertiary),
            theme.foreground(ForegroundToken::Muted),
            0.0,
            false,
        ),
    };

    let content = centered_glyph(label, text_color, ICON_BUTTON, ICON_GLYPH_SIZE);

    let mut control = button(content)
        .padding(0)
        .width(Length::Fixed(ICON_BUTTON))
        .height(Length::Fixed(ICON_BUTTON))
        .style(move |_t: &Theme, status| {
            icon_button_style(theme, background, text_color, border_width, primary, status)
        });

    if let Some(event) = event {
        control = control.on_press(event);
    }

    control.into()
}

fn upcoming_action_glyph(label: &'static str, color: iced::Color) -> Element<'static, ChatEvent> {
    container(centered_glyph(label, color, ICON_BUTTON, ICON_GLYPH_SIZE))
        .width(Length::Fixed(ICON_BUTTON))
        .height(Length::Fixed(ICON_BUTTON))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
}

fn composer_can_send(state: &ChatState, has_api_key: bool, models_loading: bool) -> bool {
    !state.draft.trim().is_empty() && has_api_key && !state.is_streaming && !models_loading
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_token_count_scales_thousands() {
        assert_eq!(format_token_count(500), "500");
        assert_eq!(format_token_count(1_500), "1.5k");
        assert_eq!(format_token_count(12_000), "12k");
    }

    #[test]
    fn token_meter_bar_ratio_is_zero_when_empty() {
        let theme = OpenCoreTheme::from_mode(crate::shared::design::ThemeMode::Dark);
        let tokens = TokenEstimate {
            used: 0,
            budget: 128_000,
        };
        let _bar = token_meter_bar(theme, tokens);
    }
}
