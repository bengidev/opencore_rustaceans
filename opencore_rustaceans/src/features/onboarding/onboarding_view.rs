//! Onboarding view — immersive monochrome landing.
//!
//! Layered scene with animated backdrop, large interactive galaxy orb,
//! and centered hero copy.

use iced::Element;
use iced::Font;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::Canvas;
use iced::widget::text::Wrapping;
use iced::widget::{MouseArea, Space, Stack, button, column, container, row, text};

use crate::shared::design::{
    chip_button_style, primary_button_style, with_alpha,
};
use crate::shared::design::OpenCoreTheme;
use crate::shared::design::ThemeMode;
use crate::shared::design::design_radius::control_radius;
use crate::shared::design::design_tokens::{
    ActionToken, BackgroundToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::onboarding_feature_card_icon::{FeatureCardIcon, FeatureKind};
use super::onboarding_galaxy_orb::GalaxyOrbProgram;
use super::onboarding_messages::OnboardingMessage;
use super::onboarding_scene_backdrop::SceneBackdrop;
use super::onboarding_state::{FEATURE_COUNT, OnboardingState};

const HERO_MAX_WIDTH: f32 = 600.0;
const ORB_HEIGHT: f32 = 360.0;
const FEATURE_CARD_SIZE: f32 = 88.0;
const EDGE_INSET_H: f32 = 16.0;
const EDGE_INSET_V: f32 = 20.0;

const FEATURES: [(FeatureKind, &str); FEATURE_COUNT] = [
    (FeatureKind::Chat, "Chat"),
    (FeatureKind::Terminal, "Terminal"),
    (FeatureKind::TextEditor, "Editor"),
    (FeatureKind::Rust, "Rust"),
];

/// Render the onboarding view.
pub fn view(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let theme = state.theme;

    let backdrop = Canvas::new(SceneBackdrop::new(theme, state.started_at, state.now))
        .width(Length::Fill)
        .height(Length::Fill);

    let main = container(
        column![
            header_row(state),
            Space::new().height(Length::Fixed(SpacingToken::S4.value())),
            row![
                Space::new().width(Length::Fill),
                container(hero_block(state))
                    .width(Length::Fixed(HERO_MAX_WIDTH))
                    .align_x(Horizontal::Center),
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill),
            Space::new().height(Length::Fixed(SpacingToken::S4.value())),
            feature_cards_row(state),
            Space::new().height(Length::Fill),
            action_row(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([EDGE_INSET_V, EDGE_INSET_H])
    .align_y(Vertical::Top);

    let scene = Stack::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .push(main)
        .push_under(backdrop);

    container(scene)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            ..Default::default()
        })
        .into()
}

fn header_row(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let theme = state.theme;

    let brand = column![
        text("OpenCore")
            .size(TypeRole::LabelMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Primary)),
            }),
        text("LOCAL AI WORKSPACE")
            .size(9)
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            }),
    ]
    .spacing(2);

    let controls = theme_toggle_button(state);

    row![brand, Space::new().width(Length::Fill), controls,]
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .into()
}

fn theme_toggle_button(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let theme = state.theme;
    let label = match state.theme_mode {
        ThemeMode::Dark => "Light",
        ThemeMode::Light => "Dark",
    };

    button(
        row![
            theme_mode_icon(state.theme_mode, theme),
            Space::new().width(Length::Fixed(6.0)),
            text(label).size(10).style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([6, 10])
    .on_press(OnboardingMessage::ToggleTheme)
    .style(move |_t: &Theme, status| chip_button_style(theme, status, false))
    .into()
}

fn theme_mode_icon(mode: ThemeMode, theme: OpenCoreTheme) -> Element<'static, OnboardingMessage> {
    let stroke = theme.foreground(ForegroundToken::Accent);
    let fill = with_alpha(stroke, 0.18);

    let (symbol, bg) = match mode {
        ThemeMode::Dark => ("◐", fill),
        ThemeMode::Light => ("◑", with_alpha(stroke, 0.28)),
    };

    container(text(symbol).size(11).style(move |_t: &Theme| text::Style {
        color: Some(stroke),
    }))
    .padding([2, 4])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            radius: control_radius(),
            width: 1.0,
            color: with_alpha(stroke, 0.35),
        },
        ..Default::default()
    })
    .into()
}

fn hero_block(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let theme = state.theme;

    let orb = Canvas::new(GalaxyOrbProgram::with_dynamics(
        theme,
        state.started_at,
        state.now,
        state.displayed_speed,
        state.displayed_zoom,
    ))
    .width(Length::Fill)
    .height(Length::Fixed(ORB_HEIGHT));

    let interactive_orb = MouseArea::new(orb)
        .on_press(OnboardingMessage::OrbPressed)
        .on_release(OnboardingMessage::OrbReleased)
        .interaction(iced::mouse::Interaction::Pointer);

    let headline = text("Your local AI command workspace")
        .size(TypeRole::DisplayMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let subhead = text(
        "OpenCore combines chat, terminal, editing, and Rust-native performance in one permissioned desktop environment. To leave the crowded cloud, polluted by leaks and unconsciousness, to return to a workspace that stays on your machine.",
    )
    .font(Font::MONOSPACE)
    .size(TypeRole::MonoSm.size())
    .line_height(iced::widget::text::LineHeight::Relative(
        TypeRole::MonoSm.line_height(),
    ))
    .width(Length::Fill)
    .wrapping(Wrapping::Word)
    .style(move |_t: &Theme| text::Style {
        color: Some(theme.foreground(ForegroundToken::Secondary)),
    });

    column![
        container(interactive_orb)
            .width(Length::Fill)
            .height(Length::Fixed(ORB_HEIGHT)),
        Space::new().height(Length::Fixed(28.0)),
        headline,
        Space::new().height(Length::Fixed(10.0)),
        subhead,
    ]
    .spacing(0)
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn feature_cards_row(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let mut cards = row![].spacing(SpacingToken::S3.value());
    for (index, (kind, label)) in FEATURES.iter().enumerate() {
        cards = cards.push(feature_card(state, index, *kind, label));
    }

    row![
        Space::new().width(Length::Fill),
        cards,
        Space::new().width(Length::Fill),
    ]
    .align_y(Vertical::Center)
    .width(Length::Fill)
    .into()
}

fn feature_card<'a>(
    state: &'a OnboardingState,
    index: usize,
    kind: FeatureKind,
    label: &'a str,
) -> Element<'a, OnboardingMessage> {
    let theme = state.theme;
    let glow = state.feature_glow[index];
    let selected = state.selected_feature == index;

    let icon = Canvas::new(FeatureCardIcon::new(
        kind,
        glow,
        state.now,
        state.started_at,
        theme,
    ))
    .width(Length::Fixed(FEATURE_CARD_SIZE))
    .height(Length::Fixed(FEATURE_CARD_SIZE));

    let label_color = if selected {
        theme.foreground(ForegroundToken::Primary)
    } else {
        theme.foreground(ForegroundToken::Muted)
    };

    let content = column![
        icon,
        Space::new().height(Length::Fixed(6.0)),
        text(label).size(10).style(move |_t: &Theme| text::Style {
            color: Some(label_color),
        }),
    ]
    .align_x(Horizontal::Center)
    .width(Length::Fixed(FEATURE_CARD_SIZE));

    MouseArea::new(content)
        .on_press(OnboardingMessage::FeatureSelected(index))
        .on_enter(OnboardingMessage::FeatureHovered(Some(index)))
        .on_exit(OnboardingMessage::FeatureHovered(None))
        .interaction(iced::mouse::Interaction::Pointer)
        .into()
}

fn action_row(state: &OnboardingState) -> Element<'_, OnboardingMessage> {
    let theme = state.theme;

    let primary = button(
        row![
            text("Enter OpenCore")
                .size(13)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.action(ActionToken::StrongText)),
                }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([14, 28])
    .on_press(OnboardingMessage::EnterPressed)
    .style(move |_t: &Theme, status| primary_button_style(theme, status));

    let skip = button(text("Skip").size(12).style(move |_t: &Theme| text::Style {
        color: Some(theme.foreground(ForegroundToken::Muted)),
    }))
    .padding([14, 18])
    .on_press(OnboardingMessage::Skipped)
    .style(move |_t: &Theme, status| chip_button_style(theme, status, false));

    row![
        Space::new().width(Length::Fill),
        skip,
        Space::new().width(Length::Fixed(12.0)),
        primary,
        Space::new().width(Length::Fill),
    ]
    .align_y(Vertical::Center)
    .width(Length::Fill)
    .into()
}
