//! ANC tab: mode chips, level/intensity controls and apply action.

use iced::widget::{column, horizontal_space, row, slider, text};
use iced::{Alignment, Element, Length};

use melobudsnext::protocol::anc::AncScene;

use crate::app::{App, Message};
use crate::options::AncChoice;
use crate::theme::{slider_style, DANGER, MUTED};
use crate::widgets::{
    anc_color, anc_label, chip, info_pill, labeled_slider, primary_button, section,
    unknown_anc_label,
};

pub(super) fn anc_tab(app: &App) -> Element<'_, Message> {
    let choice = |label: &'static str, value: AncChoice| -> Element<'_, Message> {
        chip(label, app.anc_choice == value, Message::SelectAnc(value))
    };

    let mut modes = column![
        row![
            choice("Desligado", AncChoice::Off),
            choice("Interno", AncChoice::Scene(AncScene::Indoor)),
            choice("Diário", AncChoice::Scene(AncScene::Daily)),
            choice("Ruído", AncChoice::Scene(AncScene::Noise)),
        ]
        .spacing(7),
        row![
            choice("Vento", AncChoice::Scene(AncScene::Wind)),
            choice("Adaptativo", AncChoice::Scene(AncScene::Adaptive)),
            choice("Transparência", AncChoice::Transparency),
        ]
        .spacing(7),
    ]
    .spacing(7);

    let details = match app.anc_choice {
        AncChoice::Off => {
            column![
                text("O fone reproduz o áudio sem cancelamento e sem transparência.")
                    .size(11)
                    .color(MUTED),
            ]
        }
        AncChoice::Scene(scene) if scene.supports_levels() => column![
            labeled_slider(
                "Nível de cancelamento",
                format!("{}", app.anc_level),
                slider(1..=3, app.anc_level, Message::AncLevelChanged)
                    .step(1)
                    .width(Length::Fill)
                    .style(slider_style),
            ),
            row![
                text("1 · Suave").size(9).color(MUTED),
                horizontal_space(),
                text("2 · Médio").size(9).color(MUTED),
                horizontal_space(),
                text("3 · Forte").size(9).color(MUTED),
            ],
        ],
        AncChoice::Scene(_) => {
            column![text("Este cenário é adaptativo e não usa um nível manual.")
                .size(11)
                .color(MUTED),]
        }
        AncChoice::Transparency => {
            let mut column = column![crate::widgets::toggle_row(
                "Reforço de voz",
                "Prioriza vozes próximas em vez de aplicar intensidade fixa.",
                app.transparency_voice,
                true,
                Message::VoiceEnhanced,
            ),]
            .spacing(10);

            if !app.transparency_voice {
                column = column.push(labeled_slider(
                    "Intensidade da transparência",
                    format!("{}", app.transparency_intensity),
                    slider(
                        1..=6,
                        app.transparency_intensity,
                        Message::TransparencyIntensityChanged,
                    )
                    .step(1)
                    .width(Length::Fill)
                    .style(slider_style),
                ));
            }

            column
        }
        AncChoice::Unknown(payload) => column![
            text("O fone reporta um modo de cancelamento que esta interface não conhece; nada será reenviado como desligado.")
                .size(11)
                .color(DANGER),
            text(format!("Valor reportado: {}", unknown_anc_label(payload)))
                .size(11)
                .color(MUTED),
            text("Escolha um modo acima para enviar uma configuração.")
                .size(10)
                .color(MUTED),
        ]
        .spacing(6),
    };

    modes = modes.push(details);

    column![
        section(
            "Modo",
            "O modo é aplicado ao fone assim que você confirmar.",
            modes,
        ),
        section(
            "Estado atual",
            "Valor reportado pela última sincronização.",
            row![
                info_pill("ANC", &anc_label(app.state.anc), anc_color(app.state.anc)),
                text("Use Atualizar no cabeçalho para reler o fone.")
                    .size(10)
                    .color(MUTED),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ),
        row![
            primary_button(
                "Aplicar ANC",
                Message::ApplyAnc,
                app.controls_enabled() && !matches!(app.anc_choice, AncChoice::Unknown(_)),
            ),
            text("A alteração é imediata e pode ser sentida nos fones.")
                .size(10)
                .color(MUTED),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(20)
    .into()
}
