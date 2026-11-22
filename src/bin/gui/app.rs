//! Application state and message handling.

use iced::Task;
use melobudsnext::config::Config;
use melobudsnext::protocol::anc::{AncMode, TransparencyMode};
use melobudsnext::protocol::commands::{self, NotificationVolume, WearDetection};
use melobudsnext::protocol::touch::{TouchKey, TouchMapping};
use melobudsnext::DeviceState;

use crate::ble_task::{Action, ActionResult, Session};
use crate::cli::{load_configuration, offline_flag, Cli};
use crate::options::{AncChoice, Tab, TouchOption, VolumeOption};

pub(crate) const OFFLINE_NOTICE: &str =
    "Modo offline: as ações do fone estão desativadas para demonstração.";

#[derive(Debug, Clone)]
pub(crate) enum Message {
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

impl Message {
    /// Whether the message would trigger a BLE write or link change.
    fn is_device_action(&self) -> bool {
        matches!(
            self,
            Self::Connect
                | Self::Refresh
                | Self::Disconnect
                | Self::ApplyAnc
                | Self::ToggleGame(_)
                | Self::ToggleSleep(_)
                | Self::ToggleLdac(_)
                | Self::ApplyBalance
                | Self::VolumeSelected(_)
                | Self::ApplyTouch
                | Self::ApplyWear
                | Self::ApplyRename
        )
    }
}

pub(crate) struct App {
    pub(crate) session: Session,
    pub(crate) connected: bool,
    pub(crate) offline: bool,
    pub(crate) busy: Option<String>,
    pub(crate) address_input: String,
    pub(crate) state: DeviceState,
    pub(crate) tab: Tab,
    pub(crate) notice: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) warnings: Vec<String>,

    pub(crate) anc_choice: AncChoice,
    pub(crate) anc_level: u8,
    pub(crate) transparency_voice: bool,
    pub(crate) transparency_intensity: u8,

    pub(crate) balance: u8,
    pub(crate) volume: Option<VolumeOption>,
    pub(crate) touch_draft: TouchMapping,

    pub(crate) wear_enabled: bool,
    pub(crate) wear_keep_anc: bool,
    pub(crate) wear_music: String,
    pub(crate) wear_tone: String,

    pub(crate) rename_input: String,
}

impl App {
    pub(crate) fn new(cli: Cli) -> (Self, Task<Message>) {
        let (config, config_error) = load_configuration(&cli);

        let offline =
            cli.offline || offline_flag(std::env::var("MELOBUDNEXT_GUI_OFFLINE").ok().as_deref());

        Self::build(config, config_error, offline)
    }

    pub(crate) fn build(
        config: Config,
        config_error: Option<String>,
        offline: bool,
    ) -> (Self, Task<Message>) {
        let target = config.mac_address.clone();
        let address_input = target.clone().unwrap_or_default();

        let mut app = Self {
            session: Session::new(config.uuids.clone(), target.clone()),
            connected: false,
            offline,
            busy: None,
            address_input,
            state: DeviceState::default(),
            tab: Tab::Status,
            notice: if offline {
                Some(OFFLINE_NOTICE.to_string())
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

    pub(crate) fn dispatch(&mut self, label: impl Into<String>, action: Action) -> Task<Message> {
        // Defense in depth: the update handler already rejects device actions
        // in offline mode, so a write can never reach the session here.
        if self.offline || self.busy.is_some() {
            return Task::none();
        }

        self.busy = Some(label.into());
        self.error = None;
        self.notice = None;
        self.warnings.clear();

        let session = self.session.clone();
        Task::perform(async move { session.run(action).await }, Message::Finished)
    }

    /// Controls that write to the headphones are only active while online and
    /// no other action is running.
    pub(crate) fn controls_enabled(&self) -> bool {
        !self.offline && self.busy.is_none()
    }

    pub(crate) fn finish(&mut self, result: ActionResult) -> Task<Message> {
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

    pub(crate) fn load_drafts(&mut self) {
        self.anc_choice = match self.state.anc {
            AncMode::Off => AncChoice::Off,
            AncMode::Anc { scene, .. } => AncChoice::Scene(scene),
            AncMode::Transparency { .. } => AncChoice::Transparency,
            AncMode::Unknown(payload) => AncChoice::Unknown(payload),
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

    pub(crate) fn active_anc_mode(&self) -> Result<AncMode, String> {
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
            AncChoice::Unknown(payload) => Err(format!(
                "O ANC reportado pelo fone é desconhecido ({}); escolha um modo antes de aplicar.",
                crate::widgets::unknown_anc_label(payload)
            )),
        }
    }

    pub(crate) fn battery_left(&self) -> Option<melobudsnext::protocol::readings::BatteryLevel> {
        self.state.battery.map(|battery| battery.left)
    }

    pub(crate) fn battery_right(&self) -> Option<melobudsnext::protocol::readings::BatteryLevel> {
        self.state.battery.map(|battery| battery.right)
    }
}

pub(crate) fn update(app: &mut App, message: Message) -> Task<Message> {
    let message = match message {
        Message::Finished(result) => return app.finish(result),
        message => message,
    };

    if apply_local(app, &message) {
        return Task::none();
    }

    if app.busy.is_some() {
        return Task::none();
    }

    // Offline mode is for layout previews and screenshots: local drafts still
    // react, but every device action is rejected before it can fail with a
    // "not connected" error.
    if app.offline && message.is_device_action() {
        app.error = None;
        app.notice = Some(OFFLINE_NOTICE.to_string());
        return Task::none();
    }

    match message {
        Message::Connect => {
            let address = app.address_input.trim().to_string();
            if address.is_empty() {
                app.error = Some("Informe o MAC ou o ID do fone para conectar.".to_string());
                return Task::none();
            }
            app.dispatch("Conectando ao fone…", Action::Connect { address })
        }
        Message::Refresh => app.dispatch("Sincronizando estado…", Action::Refresh),
        Message::Disconnect => app.dispatch("Desconectando…", Action::Disconnect),
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

        Message::ToggleGame(enabled) => {
            app.state.game_mode = enabled;
            app.dispatch(
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
            )
        }
        Message::ToggleSleep(enabled) => {
            app.state.sleep_mode = enabled;
            app.dispatch(
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
            )
        }
        Message::ToggleLdac(enabled) => {
            app.state.ldac = enabled;
            app.dispatch(
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
            )
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

        Message::ApplyTouch => app.dispatch(
            "Gravando mapa de toque…",
            Action::Touch {
                mapping: app.touch_draft.clone(),
            },
        ),

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

        _ => Task::none(),
    }
}

fn apply_local(app: &mut App, message: &Message) -> bool {
    match message {
        Message::Tab(tab) => app.tab = *tab,
        Message::AddressChanged(value) => app.address_input = value.clone(),
        Message::SelectAnc(choice) => app.anc_choice = *choice,
        Message::AncLevelChanged(level) => app.anc_level = *level,
        Message::TransparencyIntensityChanged(intensity) => {
            app.transparency_intensity = *intensity;
        }
        Message::VoiceEnhanced(enabled) => app.transparency_voice = *enabled,
        Message::BalanceChanged(value) => app.balance = *value,
        Message::TouchChanged(key, option) => app.touch_draft.set(*key, option.0),
        Message::ResetTouch => {
            app.touch_draft = app.state.touch.clone().unwrap_or_default();
            app.notice = Some("Alterações do mapa de toque descartadas.".to_string());
        }
        Message::WearEnabled(enabled) => app.wear_enabled = *enabled,
        Message::WearKeepAnc(enabled) => app.wear_keep_anc = *enabled,
        Message::WearMusicChanged(value) => app.wear_music = value.clone(),
        Message::WearToneChanged(value) => app.wear_tone = value.clone(),
        Message::RenameChanged(value) => app.rename_input = value.clone(),
        _ => return false,
    }

    true
}
