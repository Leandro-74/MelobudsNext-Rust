//! Status tab: battery, device summary and rename.

use iced::widget::{column, row, text, text_input};
use iced::{Element, Length};

use melobudsnext::protocol::MAX_DEVICE_NAME_LEN;

use crate::app::{App, Message};
use crate::theme::{input_style, DANGER, MUTED};
use crate::widgets::{anc_label, battery_card, info_row, primary_button, section};

pub(super) fn status_tab(app: &App) -> Element<'_, Message> {
    let firmware = app
        .state
        .firmware
        .map(|firmware| firmware.to_string())
        .unwrap_or_else(|| "não lido".to_string());
    let wear = app
        .state
        .wear_detection
        .map(|wear| {
            if wear.enabled {
                "Ativada"
            } else {
                "Desativada"
            }
        })
        .unwrap_or("não lido");
    let volume = app
        .volume
        .map(|option| option.label())
        .unwrap_or("não lido");

    column![
        row![
            battery_card("FONE ESQUERDO", app.battery_left()),
            battery_card("FONE DIREITO", app.battery_right()),
        ]
        .spacing(10),
        section(
            "Dispositivo",
            "Identificação e estado atual lidos do fone.",
            column![
                info_row("Nome", app.state.name.as_deref().unwrap_or("não definido")),
                info_row("Firmware", &firmware),
                info_row("Endereço", &app.address_input),
                info_row("ANC", anc_label(app.state.anc)),
                info_row(
                    "LDAC",
                    if app.state.ldac {
                        "Ligado"
                    } else {
                        "Desligado"
                    }
                ),
                info_row("Detecção de uso", wear),
                info_row("Volume de notificações", volume),
            ]
            .spacing(9),
        ),
        section(
            "Renomear",
            "Até 32 bytes. O nome é gravado no fone e passa a aparecer no Bluetooth.",
            column![
                row![
                    text_input("Nome do dispositivo", &app.rename_input)
                        .on_input(Message::RenameChanged)
                        .on_submit(Message::ApplyRename)
                        .padding([10, 12])
                        .size(13)
                        .width(Length::Fill)
                        .style(input_style),
                    primary_button("Aplicar", Message::ApplyRename, app.controls_enabled()),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text(format!(
                    "{} / {} bytes",
                    app.rename_input.len(),
                    MAX_DEVICE_NAME_LEN
                ))
                .size(10)
                .color(if app.rename_input.len() > MAX_DEVICE_NAME_LEN {
                    DANGER
                } else {
                    MUTED
                }),
            ]
            .spacing(7),
        ),
    ]
    .spacing(20)
    .into()
}
