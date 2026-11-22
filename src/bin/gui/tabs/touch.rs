//! Touch tab: gesture mapping for both earbuds.

use iced::widget::{column, container, pick_list, row, text};
use iced::{Alignment, Element, Length};

use melobudsnext::protocol::touch::TouchKey;

use crate::app::{App, Message};
use crate::options::TouchOption;
use crate::theme::{bold, menu_style, pick_list_style, soft_card_style, MUTED, TEXT};
use crate::widgets::{primary_button, secondary_button, section};

pub(super) fn touch_tab(app: &App) -> Element<'_, Message> {
    let column_for = |title: &'static str, keys: [TouchKey; 4]| -> Element<'_, Message> {
        let mut rows = column![].spacing(9);
        for (index, key) in keys.into_iter().enumerate() {
            let current = app.touch_draft.get(key);
            let options = TouchOption::options(current);
            let selected = options.iter().copied().find(|option| option.0 == current);
            rows = rows.push(
                row![
                    text(format!(
                        "{} toque{}",
                        index + 1,
                        if index == 0 { "" } else { "s" }
                    ))
                    .size(11)
                    .color(MUTED)
                    .width(Length::Fixed(66.0)),
                    pick_list(options, selected, move |option| {
                        Message::TouchChanged(key, option)
                    })
                    .padding([8, 10])
                    .text_size(12)
                    .width(Length::Fill)
                    .style(pick_list_style)
                    .menu_style(menu_style),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }

        container(column![text(title).size(13).font(bold()).color(TEXT), rows,].spacing(11))
            .width(Length::Fill)
            .padding(13)
            .style(soft_card_style)
            .into()
    };

    column![
        section(
            "Gestos",
            "Cada gesto pode executar uma função diferente em cada fone.",
            row![
                column_for(
                    "Fone esquerdo",
                    [
                        TouchKey::LeftSingle,
                        TouchKey::LeftDouble,
                        TouchKey::LeftTriple,
                        TouchKey::LeftQuad,
                    ],
                ),
                column_for(
                    "Fone direito",
                    [
                        TouchKey::RightSingle,
                        TouchKey::RightDouble,
                        TouchKey::RightTriple,
                        TouchKey::RightQuad,
                    ],
                ),
            ]
            .spacing(10),
        ),
        row![
            primary_button(
                "Aplicar mapa de toque",
                Message::ApplyTouch,
                app.controls_enabled()
            ),
            secondary_button(
                "Descartar alterações",
                Message::ResetTouch,
                app.controls_enabled()
            ),
            text("O mapa é gravado direto na characteristic 0000000D.")
                .size(10)
                .color(MUTED),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(20)
    .into()
}
