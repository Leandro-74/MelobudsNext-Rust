//! Wear tab: wear detection and its warning sound configuration.

use iced::widget::{column, row, text};
use iced::Element;

use crate::app::{App, Message};
use crate::theme::MUTED;
use crate::widgets::{number_field, primary_button, section, toggle_row};

pub(super) fn wear_tab(app: &App) -> Element<'_, Message> {
    column![
        section(
            "Detecção de uso",
            "Pausa a música quando os fones são removidos.",
            column![
                toggle_row(
                    "Ativar detecção",
                    "Usa os sensores ópticos para saber se o fone está no ouvido.",
                    app.wear_enabled,
                    true,
                    Message::WearEnabled,
                ),
                toggle_row(
                    "Manter ANC ao remover",
                    "Não desliga o cancelamento quando o fone sai do ouvido.",
                    app.wear_keep_anc,
                    true,
                    Message::WearKeepAnc,
                ),
            ]
            .spacing(9),
        ),
        section(
            "Aviso sonoro",
            "Índice de música e tom enviados com a configuração de uso.",
            column![
                row![
                    number_field("Índice da música", &app.wear_music, Message::WearMusicChanged),
                    number_field("Tom", &app.wear_tone, Message::WearToneChanged),
                ]
                .spacing(10),
                primary_button(
                    "Aplicar configuração",
                    Message::ApplyWear,
                    app.controls_enabled()
                ),
            ]
            .spacing(10),
        ),
        section(
            "Como funciona",
            "A detecção de uso é opcional no Melobuds Pro. Quando ativa, o fone pausa a reprodução ao ser removido e pode manter o ANC ligado, dependendo das opções acima.",
            text("A leitura atual é atualizada junto com o restante do estado.")
                .size(10)
                .color(MUTED),
        ),
    ]
    .spacing(20)
    .into()
}
