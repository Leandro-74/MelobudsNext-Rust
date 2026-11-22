//! Reusable widgets and human-readable protocol labels.

use iced::widget::{
    button, column, container, horizontal_space, row, text, text_input, toggler, Space,
};
use iced::{Alignment, Border, Color, Element, Length};

use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};
use melobudsnext::protocol::readings::{Battery, BatteryLevel};
use melobudsnext::protocol::touch::TouchFunction;

use crate::app::Message;
use crate::options::Tab;
use crate::theme::{
    bold, chip_style, divider_style, medium, primary_button_style, secondary_button_style,
    soft_card_style, soft_pill_style, track_style, ACCENT, CARD_ALT, DANGER, MUTED, TEXT, VIOLET,
};

pub(crate) fn section<'a>(
    title: &'a str,
    description: &'a str,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(divider_style),
        column![
            text(title).size(14).font(bold()).color(TEXT),
            text(description).size(10).color(MUTED),
        ]
        .spacing(2),
        body.into(),
    ]
    .spacing(10)
    .into()
}

pub(crate) fn info_row(
    label: impl Into<String>,
    value: impl Into<String>,
) -> Element<'static, Message> {
    row![
        text(label.into())
            .size(11)
            .color(MUTED)
            .width(Length::Fixed(150.0)),
        text(value.into()).size(11).color(TEXT).width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub(crate) fn labeled_slider<'a>(
    label: &'a str,
    value: impl Into<String>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        row![
            text(label).size(11).color(MUTED),
            horizontal_space(),
            text(value.into()).size(12).font(bold()).color(ACCENT),
        ]
        .align_y(Alignment::Center),
        control.into(),
    ]
    .spacing(7)
    .into()
}

pub(crate) fn toggle_row(
    title: &'static str,
    description: &'static str,
    value: bool,
    enabled: bool,
    message: impl Fn(bool) -> Message + 'static,
) -> Element<'static, Message> {
    container(
        row![
            column![
                text(title).size(12).font(bold()).color(TEXT),
                text(description).size(10).color(MUTED),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(value)
                .on_toggle_maybe(enabled.then_some(message))
                .size(20)
                .style(move |theme, status| crate::theme::toggler_style(value, theme, status)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .style(move |theme| {
        let mut style = soft_card_style(theme);
        if !enabled {
            style.background = Some(
                Color {
                    a: 0.55,
                    ..CARD_ALT
                }
                .into(),
            );
        }
        style
    })
    .into()
}

pub(crate) fn number_field<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(10).color(MUTED),
        text_input("0", value)
            .on_input(on_input)
            .padding([9, 11])
            .size(12)
            .width(Length::Fixed(160.0))
            .style(crate::theme::input_style),
    ]
    .spacing(5)
    .into()
}

pub(crate) fn battery_chip<'a>(
    label: &'a str,
    level: Option<BatteryLevel>,
) -> Element<'a, Message> {
    let (value, color) = match level {
        Some(level) if !level.is_unknown() => {
            let color = if level.percent() <= 20 { DANGER } else { TEXT };
            (format!("{}%", level.percent()), color)
        }
        _ => ("-/-".to_string(), MUTED),
    };

    container(
        row![
            text(label).size(9).font(bold()).color(MUTED),
            text(value).size(11).font(bold()).color(color),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([5, 8])
    .style(soft_pill_style)
    .into()
}

pub(crate) fn battery_card<'a>(
    label: &'a str,
    level: Option<BatteryLevel>,
) -> Element<'a, Message> {
    let percent = level
        .map(|level| level.percent())
        .filter(|_| level.is_some_and(|l| !l.is_unknown()));
    let value = match level {
        Some(level) if !level.is_unknown() => format!("{}%", level.percent()),
        _ => "-/-".to_string(),
    };
    let color = match percent {
        Some(value) if value <= 20 => DANGER,
        Some(_) => ACCENT,
        None => MUTED,
    };
    let charging = level.is_some_and(|level| level.charging());

    container(
        column![
            row![
                text(label).size(9).font(bold()).color(MUTED),
                horizontal_space(),
                if charging {
                    text("CARREGANDO").size(8).font(bold()).color(ACCENT)
                } else {
                    text("").size(8)
                },
            ]
            .align_y(Alignment::Center),
            text(value).size(25).font(bold()).color(color),
            level_bar(percent, color),
        ]
        .spacing(6),
    )
    .padding(13)
    .width(Length::Fill)
    .style(soft_card_style)
    .into()
}

fn level_bar(percent: Option<u8>, color: Color) -> Element<'static, Message> {
    let amount = f32::from(percent.unwrap_or(0).min(100)) / 100.0;
    container(
        container(Space::new(
            Length::Fixed((86.0 * amount).max(if percent.is_some() { 2.0 } else { 0.0 })),
            Length::Fixed(4.0),
        ))
        .height(Length::Fixed(4.0))
        .style(move |_| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .width(Length::Fixed(86.0))
    .height(Length::Fixed(4.0))
    .style(track_style)
    .into()
}

pub(crate) fn gesture_card(
    title: &'static str,
    left_side: &'static str,
    left_function: &'static str,
    right_side: &'static str,
    right_function: &'static str,
) -> Element<'static, Message> {
    container(
        column![
            text(title).size(9).font(bold()).color(MUTED),
            row![
                text(left_side).size(9).font(bold()).color(ACCENT),
                text(left_function).size(10).color(TEXT),
            ]
            .spacing(5),
            row![
                text(right_side).size(9).font(bold()).color(VIOLET),
                text(right_function).size(10).color(TEXT),
            ]
            .spacing(5),
        ]
        .spacing(4),
    )
    .padding(10)
    .width(Length::Fill)
    .style(soft_card_style)
    .into()
}

pub(crate) fn info_pill(label: &str, value: &str, color: Color) -> Element<'static, Message> {
    let label = label.to_string();
    let value = value.to_string();
    container(
        row![
            text(label).size(8).font(bold()).color(MUTED),
            text(value).size(10).font(bold()).color(color),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([5, 8])
    .style(move |_| crate::theme::pill_style(color))
    .into()
}

pub(crate) fn dot<'a>(color: Color, size: f32) -> Element<'a, Message> {
    container(Space::new(Length::Fixed(size), Length::Fixed(size)))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: (size / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub(crate) fn icon(
    data: &'static [u8],
    width: f32,
    height: f32,
    color: Color,
) -> Element<'static, Message> {
    iced::widget::svg(iced::widget::svg::Handle::from_memory(data))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(move |_, _| iced::widget::svg::Style { color: Some(color) })
        .into()
}

pub(crate) fn chip(
    label: &'static str,
    selected: bool,
    message: Message,
) -> Element<'static, Message> {
    button(text(label).size(11).font(medium()))
        .padding([8, 12])
        .style(move |theme, status| chip_style(selected, theme, status))
        .on_press(message)
        .into()
}

pub(crate) fn tab_button(tab: Tab, active: bool) -> Element<'static, Message> {
    button(text(tab.label()).size(12).font(medium()))
        .padding([9, 14])
        .style(move |theme, status| crate::theme::tab_style(active, theme, status))
        .on_press(Message::Tab(tab))
        .into()
}

pub(crate) fn primary_button(
    label: impl Into<String>,
    message: Message,
    enabled: bool,
) -> Element<'static, Message> {
    button(text(label.into()).size(11).font(bold()))
        .padding([9, 14])
        .style(primary_button_style)
        .on_press_maybe(enabled.then_some(message))
        .into()
}

pub(crate) fn secondary_button(
    label: impl Into<String>,
    message: Message,
    enabled: bool,
) -> Element<'static, Message> {
    button(text(label.into()).size(11).font(medium()))
        .padding([9, 14])
        .style(secondary_button_style)
        .on_press_maybe(enabled.then_some(message))
        .into()
}

pub(crate) fn danger_button(
    label: impl Into<String>,
    message: Message,
    enabled: bool,
) -> Element<'static, Message> {
    button(text(label.into()).size(11).font(medium()))
        .padding([9, 14])
        .style(crate::theme::danger_button_style)
        .on_press_maybe(enabled.then_some(message))
        .into()
}

// ---------------------------------------------------------------------------
// Human-readable protocol values
// ---------------------------------------------------------------------------

pub(crate) fn scene_label(scene: AncScene) -> &'static str {
    match scene {
        AncScene::Indoor => "Interno",
        AncScene::Daily => "Diário",
        AncScene::Noise => "Ruído",
        AncScene::Wind => "Vento",
        AncScene::Adaptive => "Adaptativo",
    }
}

pub(crate) fn anc_label(mode: AncMode) -> String {
    match mode {
        AncMode::Off => "Desligado".to_string(),
        AncMode::Anc {
            scene,
            level: Some(level),
        } => format!("{} · nível {}", scene_label(scene), level),
        AncMode::Anc { scene, level: None } => scene_label(scene).to_string(),
        AncMode::Transparency {
            mode: TransparencyMode::VoiceEnhancement,
        } => "Transparência · voz".to_string(),
        AncMode::Transparency {
            mode: TransparencyMode::Intensity(intensity),
        } => format!("Transparência · {intensity}"),
        AncMode::Unknown(bytes) => format!("Desconhecido ({bytes:02X?})"),
    }
}

/// Formats the raw ANC triple reported by the firmware, e.g. `01 09 00`.
pub(crate) fn unknown_anc_label(payload: [u8; 3]) -> String {
    format!("{:02X} {:02X} {:02X}", payload[0], payload[1], payload[2])
}

pub(crate) fn anc_badge_label(mode: AncMode) -> String {
    match mode {
        AncMode::Off => "Desligado".to_string(),
        AncMode::Anc {
            scene,
            level: Some(level),
        } => format!("{} · {level}", scene_label(scene)),
        AncMode::Anc { scene, level: None } => scene_label(scene).to_string(),
        AncMode::Transparency { .. } => "Transparência".to_string(),
        AncMode::Unknown(_) => "Desconhecido".to_string(),
    }
}

pub(crate) fn anc_color(mode: AncMode) -> Color {
    match mode {
        AncMode::Off | AncMode::Unknown(_) => MUTED,
        AncMode::Anc { .. } => ACCENT,
        AncMode::Transparency { .. } => VIOLET,
    }
}

pub(crate) fn touch_function_label(function: TouchFunction) -> &'static str {
    match function {
        TouchFunction::None => "Nenhuma",
        TouchFunction::PlayPause => "Reproduzir",
        TouchFunction::Previous => "Anterior",
        TouchFunction::Next => "Próxima",
        TouchFunction::VoiceAssistant => "Assistente",
        TouchFunction::VolumeUp => "Volume +",
        TouchFunction::VolumeDown => "Volume −",
        TouchFunction::ToggleGameMode => "Modo jogo",
        TouchFunction::AnswerCall => "Atender",
        TouchFunction::RejectCall => "Rejeitar",
        TouchFunction::HoldCall => "Segurar",
        TouchFunction::AncMode => "ANC",
        TouchFunction::Unknown(_) => "Desconhecida",
    }
}

pub(crate) fn battery_summary(battery: Option<Battery>) -> String {
    match battery {
        Some(battery) => format!("L {} · R {}", battery.left, battery.right),
        None => "-/-".to_string(),
    }
}
