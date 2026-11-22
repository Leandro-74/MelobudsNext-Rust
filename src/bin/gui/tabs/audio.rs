//! Audio tab: LDAC, game/sleep modes, balance and notification volume.

use iced::widget::{column, horizontal_space, pick_list, row, slider, text};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::options::VolumeOption;
use crate::theme::{bold, menu_style, pick_list_style, slider_style, MUTED};
use crate::widgets::{labeled_slider, primary_button, section, toggle_row};

pub(super) fn audio_tab(app: &App) -> Element<'_, Message> {
    column![
        section(
            "Modos e codec",
            "LDAC reinicia a conexão; os outros modos são aplicados na hora.",
            column![
                toggle_row(
                    "LDAC",
                    "Áudio de alta resolução. Alternar reinicia a conexão BLE.",
                    app.state.ldac,
                    app.controls_enabled(),
                    Message::ToggleLdac,
                ),
                toggle_row(
                    "Modo jogo",
                    "Reduz a latência para jogos e vídeos.",
                    app.state.game_mode,
                    app.controls_enabled(),
                    Message::ToggleGame,
                ),
                toggle_row(
                    "Modo sono",
                    "Perfil de áudio suave para dormir.",
                    app.state.sleep_mode,
                    app.controls_enabled(),
                    Message::ToggleSleep,
                ),
            ]
            .spacing(9),
        ),
        section(
            "Balanço de canais",
            "0 deixa todo o som à esquerda; 100, à direita.",
            column![
                labeled_slider(
                    "Equilíbrio",
                    format!("{} / 100", app.balance),
                    slider(0..=100, app.balance, Message::BalanceChanged)
                        .step(1)
                        .width(Length::Fill)
                        .style(slider_style),
                ),
                row![
                    text("ESQUERDO").size(9).font(bold()).color(MUTED),
                    horizontal_space(),
                    text("DIREITO").size(9).font(bold()).color(MUTED),
                ],
                primary_button(
                    "Aplicar balanço",
                    Message::ApplyBalance,
                    app.controls_enabled()
                ),
            ]
            .spacing(10),
        ),
        section(
            "Volume de notificações",
            "Tom usado pelo fone para avisos e notificações.",
            pick_list(
                VolumeOption::ALL.to_vec(),
                app.volume,
                Message::VolumeSelected,
            )
            .padding([10, 12])
            .text_size(13)
            .width(Length::Fixed(220.0))
            .style(pick_list_style)
            .menu_style(menu_style),
        ),
    ]
    .spacing(20)
    .into()
}
