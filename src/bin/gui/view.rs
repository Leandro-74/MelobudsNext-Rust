//! Window layout: header, dashboard panels and footer.

use iced::widget::{column, container, horizontal_space, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{App, Message};
use crate::device_panel::left_panel;
use crate::options::Tab;
use crate::tabs;
use crate::theme::{
    bold, input_style, panel_style, root_style, scrollable_style, ACCENT, AMBER, DANGER, LOGO_SVG,
    MUTED, TEXT,
};
use crate::widgets::{
    battery_chip, danger_button, dot, info_pill, primary_button, secondary_button, tab_button,
};

pub(crate) fn view(app: &App) -> Element<'_, Message> {
    let body = row![left_panel(app), right_panel(app),]
        .spacing(16)
        .height(Length::Fill);

    // The window is fixed at 1200x800; after the 20px outer padding the
    // content is exactly 760 high. With the current iced/tiny-skia setup a
    // `Fill` height here collapses to the content height, leaving an unpainted
    // strip at the bottom, so the dashboard height is explicit.
    container(
        column![header(app), body, footer(app)]
            .spacing(14)
            .height(Length::Fixed(760.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(root_style)
    .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let name = app
        .state
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("QCY Melobuds Pro");

    let (status_text, status_color) = if let Some(label) = &app.busy {
        (label.as_str(), AMBER)
    } else if app.offline {
        ("Offline", MUTED)
    } else if app.connected {
        ("Conectado", ACCENT)
    } else {
        ("Desconectado", MUTED)
    };

    let mut controls = row![].spacing(8).align_y(Alignment::Center);

    if !app.connected {
        controls = controls.push(
            text_input("MAC ou ID do fone", &app.address_input)
                .on_input(Message::AddressChanged)
                .on_submit(Message::Connect)
                .padding([9, 11])
                .size(12)
                .width(Length::Fixed(200.0))
                .style(input_style),
        );
    }

    controls = controls.push(
        container(
            row![
                dot(status_color, 6.0),
                text(status_text).size(11).color(status_color)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .style(move |_| crate::theme::pill_style(status_color)),
    );

    if app.connected {
        controls = controls.push(secondary_button(
            "Atualizar",
            Message::Refresh,
            app.controls_enabled(),
        ));
        controls = controls.push(danger_button(
            "Desconectar",
            Message::Disconnect,
            app.controls_enabled(),
        ));
    } else {
        controls = controls.push(primary_button(
            "Conectar",
            Message::Connect,
            app.controls_enabled() && !app.address_input.trim().is_empty(),
        ));
    }

    row![
        crate::widgets::icon(LOGO_SVG, 38.0, 38.0, ACCENT),
        column![
            text("MELOBUDNEXT").size(10).font(bold()).color(ACCENT),
            text(name).size(21).font(bold()).color(TEXT),
            text(if app.connected {
                format!(
                    "Bluetooth LE · {}",
                    if app.address_input.is_empty() {
                        "conectado"
                    } else {
                        app.address_input.as_str()
                    }
                )
            } else if app.offline {
                "Bluetooth LE · modo offline".to_string()
            } else {
                "Bluetooth LE · pronto para conectar".to_string()
            })
            .size(11)
            .color(MUTED),
        ]
        .spacing(1),
        horizontal_space(),
        row![
            battery_chip("L", app.battery_left()),
            battery_chip("R", app.battery_right()),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        controls,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn right_panel(app: &App) -> Element<'_, Message> {
    let mut tabs = row![].spacing(6);
    for tab in Tab::ALL {
        tabs = tabs.push(tab_button(tab, app.tab == tab));
    }

    let content = scrollable(
        column![
            row![
                column![
                    text(app.tab.label()).size(22).font(bold()).color(TEXT),
                    text(app.tab.subtitle()).size(11).color(MUTED),
                ]
                .spacing(2),
                horizontal_space(),
                if app.connected {
                    info_pill("CONEXÃO", "Ativa", ACCENT)
                } else {
                    info_pill("CONEXÃO", "Offline", MUTED)
                },
            ]
            .align_y(Alignment::Center),
            tabs::content(app),
        ]
        .spacing(18)
        .padding(20),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .style(scrollable_style);

    container(
        column![
            tabs,
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(panel_style)
        ]
        .spacing(12)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(6))
    .height(Length::Fill)
    .into()
}

fn footer(app: &App) -> Element<'_, Message> {
    let (icon, message, color) = if let Some(error) = &app.error {
        ("●", error.as_str(), DANGER)
    } else if let Some(label) = &app.busy {
        ("◌", label.as_str(), AMBER)
    } else if let Some(warning) = app.warnings.first() {
        ("●", warning.as_str(), AMBER)
    } else if let Some(notice) = &app.notice {
        ("●", notice.as_str(), ACCENT)
    } else {
        ("●", "Pronto.", MUTED)
    };

    row![
        text(icon).size(10).color(color),
        text(message).size(11).color(color).width(Length::Fill),
        text(format!("MelobudsNext v{}", env!("CARGO_PKG_VERSION")))
            .size(10)
            .color(MUTED),
    ]
    .spacing(7)
    .align_y(Alignment::Center)
    .into()
}
