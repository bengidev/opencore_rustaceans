//! Welcome view — centered home screen with get-started and recent-project rows.
//!
//! Layout follows a narrow vertical stack: branded header, section titles
//! with dividers, and icon / label / shortcut rows.

use iced::Element;
use iced::Font;
use iced::Length;
use iced::Theme;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{MouseArea, Space, button, column, container, row, stack, text, text_input};

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_radius::{control_radius, surface_radius};
use crate::shared::design::design_tokens::{
    BackgroundToken, BorderToken, ForegroundToken, SpacingToken, TypeRole,
};

use super::welcome_messages::WelcomeMessage;
use super::welcome_model::{WelcomeIcon, WelcomeItem, WelcomeScreen, WelcomeSection};
use super::welcome_overlay::WelcomeOverlay;
use super::welcome_state::WelcomeState;

const CONTENT_MAX_WIDTH: f32 = 480.0;
const LOGO_BOX: f32 = 48.0;
const ROW_MIN_HEIGHT: f32 = 36.0;
const ICON_CELL: f32 = 20.0;
const SHORTCUT_CELL: f32 = 56.0;
const ICON_LABEL_GAP: f32 = 12.0;
const HEADER_TO_SECTIONS: f32 = 40.0;
const SECTION_GAP: f32 = 24.0;
const SECTION_TITLE_TO_RULE: f32 = 8.0;
const RULE_TO_ROWS: f32 = 4.0;
const ROW_GAP: f32 = 2.0;
const LOGO_TO_HEADLINE: f32 = 16.0;
const HEADLINE_TO_SUBTITLE: f32 = 8.0;
const ROW_PAD_H: f32 = 12.0;
const ROW_PAD_V: f32 = 8.0;

/// Render the welcome screen.
pub fn view(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let screen = state.screen();
    let theme = state.theme;

    let content = column![
        header_block(screen.headline, screen.subtitle, theme),
        Space::new().height(Length::Fixed(HEADER_TO_SECTIONS)),
        sections_column(screen, theme),
    ]
    .width(Length::Fill)
    .align_x(Horizontal::Center);

    let centered = row![
        Space::new().width(Length::Fill),
        container(content)
            .width(Length::Fixed(CONTENT_MAX_WIDTH))
            .align_x(Horizontal::Center),
        Space::new().width(Length::Fill),
    ]
    .width(Length::Fill);

    let base: Element<'_, WelcomeMessage> = container(centered)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Vertical::Center)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            ..Default::default()
        })
        .into();

    let mut layered = stack![base];

    if let Some(status) = &state.status {
        layered = layered.push(
            container(status_banner(status.clone(), theme))
                .align_y(Vertical::Bottom)
                .height(Length::Fill)
                .width(Length::Fill),
        );
    }

    if state.overlay != WelcomeOverlay::None {
        layered = layered.push(overlay_layer(state, theme));
    }

    layered.width(Length::Fill).height(Length::Fill).into()
}

fn status_banner(summary: String, theme: OpenCoreTheme) -> Element<'static, WelcomeMessage> {
    container(
        row![
            text(summary)
                .size(TypeRole::BodyMd.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Primary)),
                }),
            Space::new().width(Length::Fill),
            button(text("×").size(16.0))
                .on_press(WelcomeMessage::StatusDismiss)
                .style(|_t: &Theme, _s| button::Style {
                    background: None,
                    text_color: iced::Color::WHITE,
                    ..Default::default()
                }),
        ]
        .align_y(Vertical::Center)
        .width(Length::Fill),
    )
    .padding([8.0, 16.0])
    .width(Length::Fill)
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Secondary),
        )),
        ..Default::default()
    })
    .into()
}

fn overlay_layer(state: &WelcomeState, theme: OpenCoreTheme) -> Element<'static, WelcomeMessage> {
    match state.overlay {
        WelcomeOverlay::CommandPalette => command_palette_overlay(state, theme),
        WelcomeOverlay::CloneRepository => {
            let panel = clone_repository_panel(state, theme);
            stack![
                MouseArea::new(
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(move |_t: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.0, 0.0, 0.0, 0.55,
                            ))),
                            ..Default::default()
                        }),
                )
                .on_press(WelcomeMessage::CloneCancel)
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
        WelcomeOverlay::None => Space::new().into(),
    }
}

fn command_palette_overlay(
    state: &WelcomeState,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let panel = command_palette_panel(state, theme);

    stack![
        MouseArea::new(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_t: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.0, 0.0, 0.0, 0.55,
                    ))),
                    ..Default::default()
                }),
        )
        .on_press(WelcomeMessage::CommandPaletteDismiss)
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

fn command_palette_panel(
    state: &WelcomeState,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let commands = state.filtered_palette_commands();
    let mut rows = column![].spacing(ROW_GAP);

    for (index, command) in commands.iter().enumerate() {
        let label = text(command.label.clone())
            .size(TypeRole::LabelMd.size())
            .width(Length::Fill)
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Primary)),
            });

        let detail: Element<'static, WelcomeMessage> = match command.detail.as_ref() {
            Some(detail) => text(detail.clone())
                .size(TypeRole::MonoXs.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                })
                .into(),
            None => Space::new().width(Length::Shrink).into(),
        };

        let row_content = row![label, detail].spacing(8.0).width(Length::Fill);

        rows = rows.push(
            button(row_content)
                .width(Length::Fill)
                .padding([ROW_PAD_V, ROW_PAD_H])
                .on_press(WelcomeMessage::CommandPaletteSelect(index))
                .style(move |_t: &Theme, status| welcome_row_style(theme, status)),
        );
    }

    let search = text_input("Type a command…", &state.palette_query)
        .on_input(WelcomeMessage::CommandPaletteQueryChanged)
        .padding(10.0)
        .size(TypeRole::BodyMd.size());

    container(
        column![
            text("Command Palette")
                .size(TypeRole::BodyLg.size())
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Primary)),
                }),
            search,
            rows,
        ]
        .spacing(12.0)
        .width(Length::Fill),
    )
    .width(Length::Fixed(CONTENT_MAX_WIDTH))
    .padding(20.0)
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Secondary),
        )),
        border: iced::Border {
            radius: surface_radius(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    })
    .into()
}

fn clone_repository_panel(
    state: &WelcomeState,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let error = state.clone_error.as_ref().map(|error| {
        text(error.clone())
            .size(TypeRole::BodyMd.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Accent)),
            })
    });

    let mut body = column![
        text("Clone Repository")
            .size(TypeRole::BodyLg.size())
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Primary)),
            }),
        text_input("https://example.com/org/repo.git", &state.clone_url)
            .on_input(WelcomeMessage::CloneUrlChanged)
            .padding(10.0)
            .size(TypeRole::BodyMd.size()),
    ]
    .spacing(12.0)
    .width(Length::Fill);

    if let Some(error) = error {
        body = body.push(error);
    }

    body = body.push(
        row![
            button(text("Cancel"))
                .on_press(WelcomeMessage::CloneCancel)
                .padding([8.0, 16.0]),
            Space::new().width(Length::Fill),
            button(text("Clone"))
                .on_press(WelcomeMessage::CloneSubmit)
                .padding([8.0, 16.0]),
        ]
        .width(Length::Fill),
    );

    container(body)
        .width(Length::Fixed(CONTENT_MAX_WIDTH))
        .padding(20.0)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Secondary),
            )),
            border: iced::Border {
                radius: surface_radius(),
                width: 1.0,
                color: theme.border(BorderToken::Default),
            },
            ..Default::default()
        })
        .into()
}

fn header_block(
    headline: &'static str,
    subtitle: &'static str,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let headline_text = text(headline)
        .size(TypeRole::BodyLg.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let subtitle_text = text(subtitle)
        .font(Font {
            style: iced::font::Style::Italic,
            ..Font::DEFAULT
        })
        .size(TypeRole::BodyMd.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Secondary)),
        });

    column![
        logo_mark(theme),
        Space::new().height(Length::Fixed(LOGO_TO_HEADLINE)),
        headline_text,
        Space::new().height(Length::Fixed(HEADLINE_TO_SUBTITLE)),
        subtitle_text,
    ]
    .spacing(0)
    .align_x(Horizontal::Center)
    .into()
}

fn logo_mark(theme: OpenCoreTheme) -> Element<'static, WelcomeMessage> {
    container(text("◆").size(22.0).style(move |_t: &Theme| text::Style {
        color: Some(theme.foreground(ForegroundToken::Accent)),
    }))
    .width(Length::Fixed(LOGO_BOX))
    .height(Length::Fixed(LOGO_BOX))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        border: iced::Border {
            radius: surface_radius(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    })
    .into()
}

fn sections_column(
    screen: WelcomeScreen,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let mut sections = column![].spacing(SECTION_GAP);
    let mut flat_index = 0usize;

    for section in screen.sections {
        let mut rows = column![].spacing(ROW_GAP);
        for item in &section.items {
            let index = flat_index;
            rows = rows.push(welcome_row(item.clone(), index, theme));
            flat_index += 1;
        }

        sections = sections.push(section_block(section, rows.into(), theme));
    }

    sections.width(Length::Fill).into()
}

fn section_block(
    section: WelcomeSection,
    rows: Element<'static, WelcomeMessage>,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    column![
        section_header(section.title, theme),
        Space::new().height(Length::Fixed(RULE_TO_ROWS)),
        rows,
    ]
    .spacing(SECTION_TITLE_TO_RULE)
    .width(Length::Fill)
    .into()
}

fn section_header(title: &'static str, theme: OpenCoreTheme) -> Element<'static, WelcomeMessage> {
    let title_text = text(title)
        .size(TypeRole::MonoXs.size())
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        });

    let rule = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.border(BorderToken::Default))),
            ..Default::default()
        });

    row![
        title_text,
        Space::new().width(Length::Fixed(SpacingToken::S2.value())),
        rule,
    ]
    .align_y(Vertical::Center)
    .width(Length::Fill)
    .into()
}

fn welcome_row(
    item: WelcomeItem,
    index: usize,
    theme: OpenCoreTheme,
) -> Element<'static, WelcomeMessage> {
    let label = text(item.label)
        .size(TypeRole::LabelMd.size())
        .width(Length::Fill)
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let shortcut: Element<'static, WelcomeMessage> = match item.shortcut {
        Some(shortcut) => container(text(shortcut).size(TypeRole::MonoXs.size()).style(
            move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            },
        ))
        .width(Length::Fixed(SHORTCUT_CELL))
        .align_x(Horizontal::Right)
        .into(),
        None => Space::new().width(Length::Fixed(SHORTCUT_CELL)).into(),
    };

    let row_content = row![
        container(row_icon(item.icon, theme))
            .width(Length::Fixed(ICON_CELL))
            .align_x(Horizontal::Center),
        Space::new().width(Length::Fixed(ICON_LABEL_GAP)),
        label,
        shortcut,
    ]
    .align_y(Vertical::Center)
    .width(Length::Fill);

    let pressed = button(row_content)
        .width(Length::Fill)
        .padding([ROW_PAD_V, ROW_PAD_H])
        .on_press(WelcomeMessage::ItemPressed(item.id))
        .style(move |_t: &Theme, status| welcome_row_style(theme, status));

    let interactive = MouseArea::new(pressed)
        .on_enter(WelcomeMessage::ItemHovered(Some(index)))
        .on_exit(WelcomeMessage::ItemHovered(None))
        .interaction(iced::mouse::Interaction::Pointer);

    container(interactive)
        .width(Length::Fill)
        .height(Length::Fixed(ROW_MIN_HEIGHT))
        .align_y(Vertical::Center)
        .into()
}

fn row_icon(icon: WelcomeIcon, theme: OpenCoreTheme) -> Element<'static, WelcomeMessage> {
    let symbol = match icon {
        WelcomeIcon::Plus => "+",
        WelcomeIcon::Folder | WelcomeIcon::RecentFolder => "▣",
        WelcomeIcon::CloudDownload => "↓",
        WelcomeIcon::CommandPalette => "⌘",
    };

    text(symbol)
        .size(13.0)
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Secondary)),
        })
        .into()
}

fn welcome_row_style(theme: OpenCoreTheme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
        text_color: theme.foreground(ForegroundToken::Primary),
        border: iced::Border {
            radius: control_radius(),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
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
