//! Graphical interface for the QCY Melobuds Pro.
//!
//! The window is intentionally close to the reference layout: the headphones
//! illustration lives on the left, while the controls are grouped in tabs on
//! the right. Every action still goes through the same protocol and BLE layer
//! used by the CLI.

#[path = "gui/ble_task.rs"]
mod ble_task;

use std::fmt;

use ble_task::{Action, ActionResult, Session};
use melobudsnext::config::Config;
use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};
use melobudsnext::protocol::commands::{self, NotificationVolume, WearDetection};
use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};
use melobudsnext::DeviceState;

use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, scrollable, slider, text,
    text_input, toggler, Space,
};
use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Shadow, Size, Task, Theme, Vector,
};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

const BG: Color = Color::from_rgb(0.047, 0.051, 0.059);
const PANEL: Color = Color::from_rgb(0.071, 0.078, 0.090);
const CARD: Color = Color::from_rgb(0.090, 0.098, 0.114);
const CARD_ALT: Color = Color::from_rgb(0.125, 0.137, 0.157);
const CARD_DEEP: Color = Color::from_rgb(0.055, 0.061, 0.071);
const TEXT: Color = Color::from_rgb(0.941, 0.949, 0.961);
const MUTED: Color = Color::from_rgb(0.553, 0.584, 0.635);
const BORDER: Color = Color::from_rgb(0.173, 0.188, 0.216);
const ACCENT: Color = Color::from_rgb(0.722, 0.949, 0.294);
const ACCENT_HOVER: Color = Color::from_rgb(0.792, 0.988, 0.396);
const ACCENT_PRESSED: Color = Color::from_rgb(0.643, 0.855, 0.235);
const VIOLET: Color = Color::from_rgb(0.604, 0.549, 1.0);
const AMBER: Color = Color::from_rgb(1.0, 0.784, 0.341);
const DANGER: Color = Color::from_rgb(1.0, 0.478, 0.478);

fn bold() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    }
}

fn medium() -> Font {
    Font {
        weight: iced::font::Weight::Medium,
        ..Font::DEFAULT
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Status,
    Anc,
    Audio,
    Touch,
    Wear,
}

impl Tab {
    const ALL: [Self; 5] = [
        Self::Status,
        Self::Anc,
        Self::Audio,
        Self::Touch,
        Self::Wear,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Anc => "ANC",
            Self::Audio => "Áudio",
            Self::Touch => "Toque",
            Self::Wear => "Uso",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Status => "Resumo do fone, bateria e conexão.",
            Self::Anc => "Cancelamento de ruído e transparência.",
            Self::Audio => "Codec, modos de baixa latência e balanço.",
            Self::Touch => "Funções dos gestos em cada fone.",
            Self::Wear => "Detecção de uso e comportamento ao remover.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AncChoice {
    Off,
    Scene(AncScene),
    Transparency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VolumeOption(NotificationVolume);

impl VolumeOption {
    const ALL: [Self; 4] = [
        Self(NotificationVolume::Low),
        Self(NotificationVolume::Medium),
        Self(NotificationVolume::High),
        Self(NotificationVolume::Max),
    ];

    fn label(self) -> &'static str {
        match self.0 {
            NotificationVolume::Low => "Baixo",
            NotificationVolume::Medium => "Médio",
            NotificationVolume::High => "Alto",
            NotificationVolume::Max => "Máximo",
        }
    }
}

impl fmt::Display for VolumeOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TouchOption(TouchFunction);

impl TouchOption {
    const ALL: [Self; 12] = [
        Self(TouchFunction::None),
        Self(TouchFunction::PlayPause),
        Self(TouchFunction::Previous),
        Self(TouchFunction::Next),
        Self(TouchFunction::VoiceAssistant),
        Self(TouchFunction::VolumeUp),
        Self(TouchFunction::VolumeDown),
        Self(TouchFunction::ToggleGameMode),
        Self(TouchFunction::AnswerCall),
        Self(TouchFunction::RejectCall),
        Self(TouchFunction::HoldCall),
        Self(TouchFunction::AncMode),
    ];
}

impl fmt::Display for TouchOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            TouchFunction::None => "Nenhuma",
            TouchFunction::PlayPause => "Reproduzir / pausar",
            TouchFunction::Previous => "Faixa anterior",
            TouchFunction::Next => "Próxima faixa",
            TouchFunction::VoiceAssistant => "Assistente de voz",
            TouchFunction::VolumeUp => "Aumentar volume",
            TouchFunction::VolumeDown => "Diminuir volume",
            TouchFunction::ToggleGameMode => "Alternar modo jogo",
            TouchFunction::AnswerCall => "Atender chamada",
            TouchFunction::RejectCall => "Rejeitar chamada",
            TouchFunction::HoldCall => "Segurar chamada",
            TouchFunction::AncMode => "Alternar ANC",
            TouchFunction::Unknown(_) => "Desconhecida",
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tab(Tab),
    AddressChanged(String),
    Connect,
    Refresh,
    Disconnect,
    Finished(ActionResult),
    SelectAnc(AncChoice),
    AncLevelChanged(u8),
    TransparencyIntensityChanged(u8),
    VoiceEnhanced(bool),
    ApplyAnc,
    ToggleGame(bool),
    ToggleSleep(bool),
    ToggleLdac(bool),
    BalanceChanged(u8),
    ApplyBalance,
    VolumeSelected(VolumeOption),
    TouchChanged(TouchKey, TouchOption),
    ApplyTouch,
    ResetTouch,
    WearEnabled(bool),
    WearKeepAnc(bool),
    WearMusicChanged(String),
    WearToneChanged(String),
    ApplyWear,
    RenameChanged(String),
    ApplyRename,
}

struct App {
    session: Session,
    connected: bool,
    offline: bool,
    busy: Option<String>,
    address_input: String,
    state: DeviceState,
    tab: Tab,
    notice: Option<String>,
    error: Option<String>,
    warnings: Vec<String>,

    anc_choice: AncChoice,
    anc_level: u8,
    transparency_voice: bool,
    transparency_intensity: u8,

    balance: u8,
    volume: Option<VolumeOption>,
    touch_draft: TouchMapping,

    wear_enabled: bool,
    wear_keep_anc: bool,
    wear_music: String,
    wear_tone: String,

    rename_input: String,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (config, config_error) = match Config::load() {
            Ok(config) => (config, None),
            Err(error) => (
                Config::default(),
                Some(format!("não foi possível ler a configuração: {error}")),
            ),
        };

        let target = config.mac_address.clone();
        let address_input = target.clone().unwrap_or_default();
        let offline = std::env::args().any(|argument| argument == "--offline")
            || std::env::var_os("MELOBUDNEXT_GUI_OFFLINE").is_some();

        let mut app = Self {
            session: Session::new(config.uuids.clone(), target.clone()),
            connected: false,
            offline,
            busy: None,
            address_input,
            state: DeviceState::default(),
            tab: Tab::Status,
            notice: if offline {
                Some("Modo offline: conexões BLE desativadas para demonstração.".to_string())
            } else {
                None
            },
            error: config_error,
            warnings: Vec::new(),
            anc_choice: AncChoice::Off,
            anc_level: 2,
            transparency_voice: false,
            transparency_intensity: 3,
            balance: 50,
            volume: None,
            touch_draft: TouchMapping::default(),
            wear_enabled: false,
            wear_keep_anc: false,
            wear_music: "1".to_string(),
            wear_tone: "0".to_string(),
            rename_input: String::new(),
        };

        let task = match target {
            Some(address) if !offline && !address.trim().is_empty() => {
                app.dispatch("Conectando ao fone…", Action::Connect { address })
            }
            _ => Task::none(),
        };

        (app, task)
    }

    fn dispatch(&mut self, label: impl Into<String>, action: Action) -> Task<Message> {
        if self.busy.is_some() {
            return Task::none();
        }

        self.busy = Some(label.into());
        self.error = None;
        self.notice = None;
        self.warnings.clear();

        let session = self.session.clone();
        Task::perform(async move { session.run(action).await }, Message::Finished)
    }

    fn finish(&mut self, result: ActionResult) -> Task<Message> {
        self.busy = None;
        self.connected = result.connected;
        self.state = result.state;
        self.warnings = result.warnings;

        if let Some(address) = result.address {
            self.address_input = address;
        }

        if let Some(error) = result.error {
            self.error = Some(error);
            self.notice = None;
        } else {
            self.error = None;
            self.notice = result.notice;
        }

        if result.refresh_drafts {
            self.load_drafts();
        }

        Task::none()
    }

    fn load_drafts(&mut self) {
        self.anc_choice = match self.state.anc {
            AncMode::Off | AncMode::Unknown(_) => AncChoice::Off,
            AncMode::Anc { scene, .. } => AncChoice::Scene(scene),
            AncMode::Transparency { .. } => AncChoice::Transparency,
        };

        if let AncMode::Anc {
            level: Some(level), ..
        } = self.state.anc
        {
            self.anc_level = level;
        }

        if let AncMode::Transparency { mode } = self.state.anc {
            match mode {
                TransparencyMode::VoiceEnhancement => {
                    self.transparency_voice = true;
                }
                TransparencyMode::Intensity(intensity) => {
                    self.transparency_voice = false;
                    self.transparency_intensity = intensity;
                }
            }
        }

        self.balance = self.state.channel_balance;
        self.volume = self
            .state
            .notification_volume
            .and_then(NotificationVolume::from_byte)
            .map(VolumeOption);
        self.touch_draft = self.state.touch.clone().unwrap_or_default();

        let wear = self.state.wear_detection.unwrap_or_default();
        self.wear_enabled = wear.enabled;
        self.wear_keep_anc = wear.keep_anc;
        self.wear_music = wear.music_index.to_string();
        self.wear_tone = wear.tone.to_string();

        self.rename_input = self.state.name.clone().unwrap_or_default();
    }

    fn active_anc_mode(&self) -> Result<AncMode, String> {
        match self.anc_choice {
            AncChoice::Off => Ok(AncMode::Off),
            AncChoice::Scene(scene) => {
                if scene.supports_levels() {
                    AncMode::anc(scene, Some(self.anc_level)).map_err(|error| error.to_string())
                } else {
                    AncMode::anc(scene, None).map_err(|error| error.to_string())
                }
            }
            AncChoice::Transparency => {
                let mode = if self.transparency_voice {
                    TransparencyMode::VoiceEnhancement
                } else {
                    TransparencyMode::Intensity(self.transparency_intensity)
                };
                AncMode::transparency(mode).map_err(|error| error.to_string())
            }
        }
    }

    fn battery_left(&self) -> Option<melobudsnext::protocol::readings::BatteryLevel> {
        self.state.battery.map(|battery| battery.left)
    }

    fn battery_right(&self) -> Option<melobudsnext::protocol::readings::BatteryLevel> {
        self.state.battery.map(|battery| battery.right)
    }

    fn battery_case(&self) -> Option<melobudsnext::protocol::readings::BatteryLevel> {
        self.state.battery.map(|battery| battery.case)
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

fn update(app: &mut App, message: Message) -> Task<Message> {
    if app.busy.is_some() {
        match message {
            Message::Tab(tab) => {
                app.tab = tab;
                return Task::none();
            }
            Message::Finished(result) => return app.finish(result),
            _ => return Task::none(),
        }
    }

    match message {
        Message::Tab(tab) => {
            app.tab = tab;
            Task::none()
        }
        Message::AddressChanged(value) => {
            app.address_input = value;
            Task::none()
        }
        Message::Connect => {
            if app.offline {
                app.notice =
                    Some("Modo offline: conexões BLE desativadas para demonstração.".to_string());
                return Task::none();
            }
            let address = app.address_input.trim().to_string();
            if address.is_empty() {
                app.error = Some("Informe o MAC ou o ID do fone para conectar.".to_string());
                return Task::none();
            }
            app.dispatch("Conectando ao fone…", Action::Connect { address })
        }
        Message::Refresh => app.dispatch("Sincronizando estado…", Action::Refresh),
        Message::Disconnect => app.dispatch("Desconectando…", Action::Disconnect),
        Message::Finished(result) => app.finish(result),

        Message::SelectAnc(choice) => {
            app.anc_choice = choice;
            Task::none()
        }
        Message::AncLevelChanged(level) => {
            app.anc_level = level;
            Task::none()
        }
        Message::TransparencyIntensityChanged(intensity) => {
            app.transparency_intensity = intensity;
            Task::none()
        }
        Message::VoiceEnhanced(enabled) => {
            app.transparency_voice = enabled;
            Task::none()
        }
        Message::ApplyAnc => match app.active_anc_mode() {
            Ok(mode) => app.dispatch(
                "Aplicando ANC…",
                Action::Command {
                    packet: commands::anc(mode),
                    reconnect: false,
                    refresh_drafts: true,
                    label: "Modo de cancelamento atualizado.",
                },
            ),
            Err(error) => {
                app.error = Some(error);
                Task::none()
            }
        },

        Message::ToggleGame(enabled) => app.dispatch(
            if enabled {
                "Ativando modo jogo…"
            } else {
                "Desativando modo jogo…"
            },
            Action::Command {
                packet: commands::game_mode(enabled),
                reconnect: false,
                refresh_drafts: false,
                label: if enabled {
                    "Modo jogo ativado."
                } else {
                    "Modo jogo desativado."
                },
            },
        ),
        Message::ToggleSleep(enabled) => app.dispatch(
            if enabled {
                "Ativando modo sono…"
            } else {
                "Desativando modo sono…"
            },
            Action::Command {
                packet: commands::sleep_mode(enabled),
                reconnect: false,
                refresh_drafts: false,
                label: if enabled {
                    "Modo sono ativado."
                } else {
                    "Modo sono desativado."
                },
            },
        ),
        Message::ToggleLdac(enabled) => app.dispatch(
            "Alterando LDAC e reconectando…",
            Action::Command {
                packet: commands::ldac(enabled),
                reconnect: true,
                refresh_drafts: true,
                label: if enabled {
                    "LDAC ativado; o fone foi reconectado."
                } else {
                    "LDAC desativado; o fone foi reconectado."
                },
            },
        ),
        Message::BalanceChanged(value) => {
            app.balance = value;
            Task::none()
        }
        Message::ApplyBalance => app.dispatch(
            "Aplicando balanço…",
            Action::Command {
                packet: commands::channel_balance(app.balance),
                reconnect: false,
                refresh_drafts: false,
                label: "Balanço de canais atualizado.",
            },
        ),
        Message::VolumeSelected(option) => {
            app.volume = Some(option);
            app.dispatch(
                "Aplicando volume de notificações…",
                Action::Command {
                    packet: commands::notification_volume(option.0),
                    reconnect: false,
                    refresh_drafts: false,
                    label: "Volume de notificações atualizado.",
                },
            )
        }

        Message::TouchChanged(key, option) => {
            app.touch_draft.set(key, option.0);
            Task::none()
        }
        Message::ApplyTouch => app.dispatch(
            "Gravando mapa de toque…",
            Action::Touch {
                mapping: app.touch_draft.clone(),
            },
        ),
        Message::ResetTouch => {
            app.touch_draft = app.state.touch.clone().unwrap_or_default();
            app.notice = Some("Alterações do mapa de toque descartadas.".to_string());
            Task::none()
        }

        Message::WearEnabled(enabled) => {
            app.wear_enabled = enabled;
            Task::none()
        }
        Message::WearKeepAnc(enabled) => {
            app.wear_keep_anc = enabled;
            Task::none()
        }
        Message::WearMusicChanged(value) => {
            app.wear_music = value;
            Task::none()
        }
        Message::WearToneChanged(value) => {
            app.wear_tone = value;
            Task::none()
        }
        Message::ApplyWear => {
            let music_index = match app.wear_music.trim().parse::<u8>() {
                Ok(value) => value,
                Err(_) => {
                    app.error =
                        Some("O índice da música deve ser um número de 0 a 255.".to_string());
                    return Task::none();
                }
            };
            let tone = match app.wear_tone.trim().parse::<u8>() {
                Ok(value) => value,
                Err(_) => {
                    app.error = Some("O tom deve ser um número de 0 a 255.".to_string());
                    return Task::none();
                }
            };

            let configuration = WearDetection {
                enabled: app.wear_enabled,
                music_index,
                keep_anc: app.wear_keep_anc,
                tone,
            };

            app.dispatch(
                "Aplicando detecção de uso…",
                Action::Command {
                    packet: commands::wear_detection(configuration),
                    reconnect: false,
                    refresh_drafts: false,
                    label: "Detecção de uso atualizada.",
                },
            )
        }

        Message::RenameChanged(value) => {
            app.rename_input = value;
            Task::none()
        }
        Message::ApplyRename => {
            let name = app.rename_input.trim().to_string();
            if name.is_empty() {
                app.error = Some("O nome do dispositivo não pode ficar vazio.".to_string());
                return Task::none();
            }
            if name.len() > melobudsnext::protocol::MAX_DEVICE_NAME_LEN {
                app.error = Some(format!(
                    "O nome deve ter no máximo {} bytes (o texto atual usa {}).",
                    melobudsnext::protocol::MAX_DEVICE_NAME_LEN,
                    name.len()
                ));
                return Task::none();
            }
            app.dispatch("Renomeando dispositivo…", Action::Rename { name })
        }
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

fn view(app: &App) -> Element<'_, Message> {
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
        .style(move |_| pill_style(status_color)),
    );

    if app.connected {
        controls = controls.push(secondary_button(
            "Atualizar",
            Message::Refresh,
            app.busy.is_none(),
        ));
        controls = controls.push(danger_button(
            "Desconectar",
            Message::Disconnect,
            app.busy.is_none(),
        ));
    } else {
        controls = controls.push(primary_button(
            "Conectar",
            Message::Connect,
            app.busy.is_none() && !app.offline && !app.address_input.trim().is_empty(),
        ));
    }

    row![
        container(text("MB").size(14).font(bold()).color(BG))
            .width(Length::Fixed(38.0))
            .height(Length::Fixed(38.0))
            .center_x(Length::Fixed(38.0))
            .center_y(Length::Fixed(38.0))
            .style(logo_style),
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
            battery_chip("CASE", app.battery_case()),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        controls,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn left_panel(app: &App) -> Element<'_, Message> {
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

    let accent = if app.connected {
        anc_color(app.state.anc)
    } else {
        MUTED
    };
    let active = app.connected && !matches!(app.state.anc, AncMode::Off | AncMode::Unknown(_));
    let illustration = container(
        column![
            row![
                bud_label("L", accent),
                art_bud(accent, active),
                horizontal_space(),
                art_bud(accent, active),
                bud_label("R", accent),
            ]
            .width(Length::Fill)
            .spacing(14)
            .align_y(Alignment::Center),
            art_case(),
        ]
        .spacing(6)
        .align_x(Alignment::Center)
        .width(Length::Fill),
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
) -> impl Fn(&Theme) -> container::Style {
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

fn transparent_shadow() -> Shadow {
    Shadow {
        color: Color::TRANSPARENT,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 0.0,
    }
}

fn art_bud(accent: Color, active: bool) -> Element<'static, Message> {
    let head = container(
        container(Space::new(20.0, 20.0))
            .width(20.0)
            .height(20.0)
            .style(circle_style(
                BG,
                Color::from_rgb(0.10, 0.11, 0.13),
                1.0,
                10.0,
            )),
    )
    .width(66.0)
    .height(66.0)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.19, 0.20, 0.24))),
        border: Border {
            color: Color::from_rgb(0.36, 0.39, 0.44),
            width: 1.5,
            radius: 33.0.into(),
        },
        shadow: if active {
            Shadow {
                color: Color { a: 0.16, ..accent },
                offset: Vector::new(0.0, 0.0),
                blur_radius: 26.0,
            }
        } else {
            transparent_shadow()
        },
        ..container::Style::default()
    });

    let stem = container(
        container(Space::new(6.0, 42.0))
            .width(6.0)
            .height(42.0)
            .style(circle_style(accent, accent, 0.0, 3.0)),
    )
    .width(26.0)
    .height(76.0)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(circle_style(
        Color::from_rgb(0.17, 0.18, 0.22),
        Color::from_rgb(0.30, 0.33, 0.37),
        1.2,
        13.0,
    ));

    column![head, stem]
        .spacing(0)
        .align_x(Alignment::Center)
        .width(112.0)
        .into()
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

fn art_case() -> Element<'static, Message> {
    container(column![
        container(
            row![
                horizontal_space(),
                container(Space::new(26.0, 4.0))
                    .width(26.0)
                    .height(4.0)
                    .style(circle_style(BG, BG, 0.0, 2.0)),
                horizontal_space(),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(38.0)
        .align_y(Alignment::Center),
        container(Space::new(Length::Fill, 1.0))
            .width(Length::Fill)
            .height(1.0)
            .style(|_| container::Style {
                background: Some(Background::Color(BORDER)),
                ..container::Style::default()
            }),
        container(
            column![
                text("QCY").size(13).font(medium()).color(MUTED),
                text("CASE")
                    .size(7)
                    .font(bold())
                    .color(Color::from_rgb(0.40, 0.43, 0.48)),
            ]
            .spacing(1)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(61.0)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    ])
    .width(210.0)
    .height(100.0)
    .style(|_| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
        border: Border {
            color: Color::from_rgb(0.26, 0.28, 0.32),
            width: 1.2,
            radius: 26.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    })
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
            tab_content(app),
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

fn tab_content(app: &App) -> Element<'_, Message> {
    match app.tab {
        Tab::Status => status_tab(app),
        Tab::Anc => anc_tab(app),
        Tab::Audio => audio_tab(app),
        Tab::Touch => touch_tab(app),
        Tab::Wear => wear_tab(app),
    }
}

fn status_tab(app: &App) -> Element<'_, Message> {
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
    let volume = app.volume.map(VolumeOption::label).unwrap_or("não lido");

    column![
        row![
            battery_card("FONE ESQUERDO", app.battery_left()),
            battery_card("FONE DIREITO", app.battery_right()),
            battery_card("ESTOJO", app.battery_case()),
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
                    primary_button("Aplicar", Message::ApplyRename, app.busy.is_none()),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(format!("{} / 32 bytes", app.rename_input.len()))
                    .size(10)
                    .color(if app.rename_input.len() > 32 {
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

fn anc_tab(app: &App) -> Element<'_, Message> {
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
            let mut column = column![toggle_row(
                "Reforço de voz",
                "Prioriza vozes próximas em vez de aplicar intensidade fixa.",
                app.transparency_voice,
                app.busy.is_none(),
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
            primary_button("Aplicar ANC", Message::ApplyAnc, app.busy.is_none()),
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

fn audio_tab(app: &App) -> Element<'_, Message> {
    column![
        section(
            "Modos e codec",
            "LDAC reinicia a conexão; os outros modos são aplicados na hora.",
            column![
                toggle_row(
                    "LDAC",
                    "Áudio de alta resolução. Alternar reinicia a conexão BLE.",
                    app.state.ldac,
                    app.busy.is_none(),
                    Message::ToggleLdac,
                ),
                toggle_row(
                    "Modo jogo",
                    "Reduz a latência para jogos e vídeos.",
                    app.state.game_mode,
                    app.busy.is_none(),
                    Message::ToggleGame,
                ),
                toggle_row(
                    "Modo sono",
                    "Perfil de áudio suave para dormir.",
                    app.state.sleep_mode,
                    app.busy.is_none(),
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
                primary_button("Aplicar balanço", Message::ApplyBalance, app.busy.is_none()),
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

fn touch_tab(app: &App) -> Element<'_, Message> {
    let column_for = |title: &'static str, keys: [TouchKey; 4]| -> Element<'_, Message> {
        let mut rows = column![].spacing(9);
        for (index, key) in keys.into_iter().enumerate() {
            let current = app.touch_draft.get(key);
            let selected = TouchOption::ALL
                .iter()
                .copied()
                .find(|option| option.0 == current);
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
                    pick_list(TouchOption::ALL.to_vec(), selected, move |option| {
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
                app.busy.is_none()
            ),
            secondary_button(
                "Descartar alterações",
                Message::ResetTouch,
                app.busy.is_none()
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

fn wear_tab(app: &App) -> Element<'_, Message> {
    column![
        section(
            "Detecção de uso",
            "Pausa a música quando os fones são removidos.",
            column![
                toggle_row(
                    "Ativar detecção",
                    "Usa os sensores ópticos para saber se o fone está no ouvido.",
                    app.wear_enabled,
                    app.busy.is_none(),
                    Message::WearEnabled,
                ),
                toggle_row(
                    "Manter ANC ao remover",
                    "Não desliga o cancelamento quando o fone sai do ouvido.",
                    app.wear_keep_anc,
                    app.busy.is_none(),
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
                primary_button("Aplicar configuração", Message::ApplyWear, app.busy.is_none()),
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

// ---------------------------------------------------------------------------
// Reusable widgets
// ---------------------------------------------------------------------------

fn section<'a>(
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

fn info_row(label: impl Into<String>, value: impl Into<String>) -> Element<'static, Message> {
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

fn labeled_slider<'a>(
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

fn toggle_row(
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
                .on_toggle(message)
                .size(20)
                .style(move |theme, status| toggler_style(value, theme, status)),
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

fn number_field<'a>(
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
            .style(input_style),
    ]
    .spacing(5)
    .into()
}

fn battery_chip<'a>(
    label: &'a str,
    level: Option<melobudsnext::protocol::readings::BatteryLevel>,
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

fn battery_card<'a>(
    label: &'a str,
    level: Option<melobudsnext::protocol::readings::BatteryLevel>,
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

fn gesture_card(
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

fn info_pill(label: &str, value: &str, color: Color) -> Element<'static, Message> {
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
    .style(move |_| pill_style(color))
    .into()
}

fn dot<'a>(color: Color, size: f32) -> Element<'a, Message> {
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

fn chip(label: &'static str, selected: bool, message: Message) -> Element<'static, Message> {
    button(text(label).size(11).font(medium()))
        .padding([8, 12])
        .style(move |theme, status| chip_style(selected, theme, status))
        .on_press(message)
        .into()
}

fn tab_button(tab: Tab, active: bool) -> Element<'static, Message> {
    button(text(tab.label()).size(12).font(medium()))
        .padding([9, 14])
        .style(move |theme, status| tab_style(active, theme, status))
        .on_press(Message::Tab(tab))
        .into()
}

fn primary_button(
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

fn secondary_button(
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

fn danger_button(
    label: impl Into<String>,
    message: Message,
    enabled: bool,
) -> Element<'static, Message> {
    button(text(label.into()).size(11).font(medium()))
        .padding([9, 14])
        .style(danger_button_style)
        .on_press_maybe(enabled.then_some(message))
        .into()
}

// ---------------------------------------------------------------------------
// Human-readable protocol values
// ---------------------------------------------------------------------------

fn scene_label(scene: AncScene) -> &'static str {
    match scene {
        AncScene::Indoor => "Interno",
        AncScene::Daily => "Diário",
        AncScene::Noise => "Ruído",
        AncScene::Wind => "Vento",
        AncScene::Adaptive => "Adaptativo",
    }
}

fn anc_label(mode: AncMode) -> String {
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

fn anc_badge_label(mode: AncMode) -> String {
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

fn anc_color(mode: AncMode) -> Color {
    match mode {
        AncMode::Off | AncMode::Unknown(_) => MUTED,
        AncMode::Anc { .. } => ACCENT,
        AncMode::Transparency { .. } => VIOLET,
    }
}

fn touch_function_label(function: TouchFunction) -> &'static str {
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

fn battery_summary(battery: Option<melobudsnext::protocol::readings::Battery>) -> String {
    match battery {
        Some(battery) => format!("{} / {} / {}", battery.left, battery.right, battery.case),
        None => "-/-".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn theme(_app: &App) -> Theme {
    Theme::custom(
        "MelobudsNext".to_string(),
        iced::theme::Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: ACCENT,
            danger: DANGER,
        },
    )
}

fn root_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(BG.into()),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 22.0,
        },
        ..Default::default()
    }
}

fn illustration_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD_DEEP.into()),
        border: Border {
            color: Color { a: 0.7, ..BORDER },
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}

fn soft_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD.into()),
        border: Border {
            color: Color { a: 0.8, ..BORDER },
            width: 1.0,
            radius: 11.0.into(),
        },
        ..Default::default()
    }
}

fn track_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD_ALT.into()),
        border: Border {
            radius: 2.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn divider_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color { a: 0.7, ..BORDER }.into()),
        ..Default::default()
    }
}

fn logo_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(ACCENT.into()),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn soft_pill_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD.into()),
        border: Border {
            color: Color { a: 0.8, ..BORDER },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn pill_style(color: Color) -> container::Style {
    container::Style {
        background: Some(Color { a: 0.10, ..color }.into()),
        border: Border {
            color: Color { a: 0.35, ..color },
            width: 1.0,
            radius: 20.0.into(),
        },
        text_color: Some(color),
        ..Default::default()
    }
}

fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => ACCENT,
        text_input::Status::Hovered => Color { a: 0.8, ..BORDER },
        _ => BORDER,
    };
    text_input::Style {
        background: CARD_DEEP.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 9.0.into(),
        },
        icon: MUTED,
        placeholder: MUTED,
        value: TEXT,
        selection: Color { a: 0.35, ..ACCENT },
    }
}

fn pick_list_style(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: TEXT,
        placeholder_color: MUTED,
        handle_color: if matches!(status, pick_list::Status::Opened) {
            ACCENT
        } else {
            MUTED
        },
        background: CARD_DEEP.into(),
        border: Border {
            color: if matches!(
                status,
                pick_list::Status::Hovered | pick_list::Status::Opened
            ) {
                ACCENT
            } else {
                BORDER
            },
            width: 1.0,
            radius: 9.0.into(),
        },
    }
}

fn menu_style(_theme: &Theme) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: CARD.into(),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: TEXT,
        selected_text_color: BG,
        selected_background: ACCENT.into(),
    }
}

fn slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (ACCENT.into(), CARD_ALT.into()),
            width: 5.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: ACCENT.into(),
            border_width: 3.0,
            border_color: BG,
        },
    }
}

fn toggler_style(toggled: bool, _theme: &Theme, status: toggler::Status) -> toggler::Style {
    let hovered = matches!(
        status,
        toggler::Status::Hovered { .. } | toggler::Status::Active { .. }
    );
    toggler::Style {
        background: if toggled {
            if hovered {
                ACCENT_HOVER
            } else {
                ACCENT
            }
        } else {
            CARD_ALT
        },
        background_border_width: 1.0,
        background_border_color: if toggled { ACCENT } else { BORDER },
        foreground: if toggled { BG } else { MUTED },
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
    }
}

fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => ACCENT_HOVER,
        button::Status::Pressed => ACCENT_PRESSED,
        button::Status::Disabled => CARD_ALT,
        _ => ACCENT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if matches!(status, button::Status::Disabled) {
            MUTED
        } else {
            BG
        },
        border: Border {
            radius: 9.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
    }
}

fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.55,
            ..CARD_ALT
        },
        button::Status::Pressed => CARD_ALT,
        button::Status::Disabled => Color { a: 0.4, ..CARD_ALT },
        _ => CARD_ALT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if matches!(status, button::Status::Disabled) {
            MUTED
        } else {
            TEXT
        },
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let color = if matches!(status, button::Status::Disabled) {
        Color { a: 0.4, ..DANGER }
    } else if matches!(status, button::Status::Hovered) {
        DANGER
    } else {
        Color { a: 0.85, ..DANGER }
    };
    button::Style {
        background: Some(Color { a: 0.08, ..DANGER }.into()),
        text_color: color,
        border: Border {
            color: Color { a: 0.35, ..DANGER },
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn tab_style(active: bool, _theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color) = if active {
        (ACCENT, BG)
    } else {
        match status {
            button::Status::Hovered => (CARD_ALT, TEXT),
            button::Status::Pressed => (CARD, TEXT),
            _ => (Color::TRANSPARENT, MUTED),
        }
    };

    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: if active { ACCENT } else { Color::TRANSPARENT },
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn chip_style(selected: bool, _theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = if selected {
        (
            Color { a: 0.16, ..ACCENT },
            ACCENT,
            Color { a: 0.6, ..ACCENT },
        )
    } else {
        match status {
            button::Status::Hovered => (CARD_ALT, TEXT, BORDER),
            _ => (CARD, MUTED, BORDER),
        }
    };

    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
    }
}

fn scrollable_style(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> iced::Result {
    iced::application("MelobudsNext - QCY Melobuds Pro", update, view)
        .theme(theme)
        .window(iced::window::Settings {
            size: Size::new(1200.0, 800.0),
            resizable: false,
            ..Default::default()
        })
        .run_with(App::new)
}
