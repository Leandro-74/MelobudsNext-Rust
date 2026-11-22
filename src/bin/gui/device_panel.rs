//! Left dashboard panel with the headphone illustration and live summary.

use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

use melobudsnext::protocol::touch::TouchKey;

use crate::app::{App, Message};
use crate::theme::{
    bold, illustration_style, panel_style, pill_style, ACCENT, CARD_DEEP, EARBUD_LEFT_SVG,
    EARBUD_RIGHT_SVG, MUTED, TEXT, VIOLET,
};
use crate::widgets::{
    anc_badge_label, anc_color, battery_summary, dot, gesture_card, icon, info_pill,
    touch_function_label,
};

pub(crate) fn left_panel(app: &App) -> Element<'_, Message> {
    let info = row![
        info_pill(
            "ANC",
            &anc_badge_label(app.state.anc),
            anc_color(app.state.anc)
        ),
        info_pill(
            "LDAC",
            if app.state.ldac {
                "Ligado"
            } else {
                "Desligado"
            },
            if app.state.ldac { VIOLET } else { MUTED },
        ),
        horizontal_space(),
        text(battery_summary(app.state.battery))
            .size(10)
            .color(MUTED),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let bud_color = if app.connected { ACCENT } else { MUTED };
    let illustration = container(
        row![
            bud_label("L", bud_color),
            icon(EARBUD_LEFT_SVG, 104.0, 154.0, bud_color),
            horizontal_space(),
            icon(EARBUD_RIGHT_SVG, 104.0, 154.0, bud_color),
            bud_label("R", bud_color),
        ]
        .width(Length::Fill)
        .spacing(14)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(12)
    .style(illustration_style);

    let touch = app.state.touch.clone().unwrap_or_default();

    let gestures = column![
        row![
            gesture_card(
                "1 toque",
                "L",
                touch_function_label(touch.get(TouchKey::LeftSingle)),
                "R",
                touch_function_label(touch.get(TouchKey::RightSingle))
            ),
            gesture_card(
                "2 toques",
                "L",
                touch_function_label(touch.get(TouchKey::LeftDouble)),
                "R",
                touch_function_label(touch.get(TouchKey::RightDouble))
            ),
        ]
        .spacing(8),
        row![
            gesture_card(
                "3 toques",
                "L",
                touch_function_label(touch.get(TouchKey::LeftTriple)),
                "R",
                touch_function_label(touch.get(TouchKey::RightTriple))
            ),
            gesture_card(
                "4 toques",
                "L",
                touch_function_label(touch.get(TouchKey::LeftQuad)),
                "R",
                touch_function_label(touch.get(TouchKey::RightQuad))
            ),
        ]
        .spacing(8),
    ]
    .spacing(8);

    container(
        column![
            row![
                text("SEUS FONES").size(10).font(bold()).color(MUTED),
                horizontal_space(),
                container(
                    row![
                        dot(if app.connected { ACCENT } else { MUTED }, 5.0),
                        text(if app.connected { "AO VIVO" } else { "OFFLINE" })
                            .size(9)
                            .font(bold())
                            .color(if app.connected { ACCENT } else { MUTED }),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .padding([4, 8])
                .style(move |_| pill_style(if app.connected { ACCENT } else { MUTED })),
            ]
            .align_y(Alignment::Center),
            column![
                text(
                    app.state
                        .name
                        .as_deref()
                        .filter(|name| !name.trim().is_empty())
                        .unwrap_or("QCY Melobuds Pro")
                )
                .size(24)
                .font(bold())
                .color(TEXT),
                text("QCY Melobuds Pro · MelobudsNext")
                    .size(11)
                    .color(MUTED),
            ]
            .spacing(2),
            info,
            illustration,
            column![
                text("VISÃO DE TOQUE").size(10).font(bold()).color(MUTED),
                gestures,
            ]
            .spacing(9),
        ]
        .spacing(14)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(5))
    .height(Length::Fill)
    .padding(18)
    .style(panel_style)
    .into()
}

fn circle_style(
    fill: Color,
    border: Color,
    border_width: f32,
    radius: f32,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(fill)),
        border: Border {
            color: border,
            width: border_width,
            radius: radius.into(),
        },
        ..container::Style::default()
    }
}

fn bud_label(label: &'static str, accent: Color) -> Element<'static, Message> {
    container(text(label).size(15).font(bold()).color(TEXT))
        .width(42.0)
        .height(42.0)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(circle_style(CARD_DEEP, accent, 1.5, 21.0))
        .into()
}
