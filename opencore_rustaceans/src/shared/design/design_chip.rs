//! Selector chips and centered glyph helpers.

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Space, button, container, row, text};

use super::design_controls::chip_button_style;
use super::design_radius::control_radius;
use super::design_tokens::{ForegroundToken, SpacingToken, TypeRole};
use super::OpenCoreTheme;

pub const CHIP_GLYPH_BOX: f32 = 16.0;
pub const CHIP_GLYPH_SIZE: f32 = 12.0;
pub const STATUS_DOT: f32 = 6.0;

pub fn centered_glyph<M: Clone + 'static>(
    label: &'static str,
    color: iced::Color,
    box_size: f32,
    font_size: f32,
) -> Element<'static, M> {
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

pub fn status_dot<M: Clone + 'static>(
    theme: OpenCoreTheme,
    active: bool,
) -> Element<'static, M> {
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

/// Pressable selector chip with optional icon, trailing chevron, and status dot.
pub fn selector_chip<'a, M: Clone + 'static>(
    theme: OpenCoreTheme,
    active: bool,
    icon: Option<&'static str>,
    label: Element<'a, M>,
    trailing: Option<&'static str>,
    show_status_dot: bool,
    on_press: Option<M>,
) -> Element<'a, M> {
    let icon_color = theme.foreground(ForegroundToken::Muted);

    let mut content = row![].spacing(SpacingToken::S1.value()).align_y(Vertical::Center);

    if show_status_dot {
        content = content.push(status_dot::<M>(theme, active));
    }

    if let Some(icon) = icon {
        content = content.push(centered_glyph(
            icon,
            icon_color,
            CHIP_GLYPH_BOX,
            CHIP_GLYPH_SIZE,
        ));
    }

    content = content.push(label);

    if let Some(chevron) = trailing {
        content = content.push(centered_glyph(
            chevron,
            icon_color,
            CHIP_GLYPH_BOX,
            CHIP_GLYPH_SIZE,
        ));
    }

    let chip_body = container(content)
        .padding([SpacingToken::S1.value(), SpacingToken::S3.value()])
        .align_y(Vertical::Center)
        .align_x(Horizontal::Left);

    let mut chip = button(chip_body)
        .padding(0)
        .style(move |_t: &Theme, status| chip_button_style(theme, status, active));

    if let Some(event) = on_press {
        chip = chip.on_press(event);
    }

    chip.into()
}
