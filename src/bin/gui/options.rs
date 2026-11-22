//! Option types shown by the interface controls.

use std::fmt;

use melobudsnext::protocol::anc::AncScene;
use melobudsnext::protocol::commands::NotificationVolume;
use melobudsnext::protocol::touch::TouchFunction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
    Status,
    Anc,
    Audio,
    Touch,
    Wear,
}

impl Tab {
    pub(crate) const ALL: [Self; 5] = [
        Self::Status,
        Self::Anc,
        Self::Audio,
        Self::Touch,
        Self::Wear,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Anc => "ANC",
            Self::Audio => "Áudio",
            Self::Touch => "Toque",
            Self::Wear => "Uso",
        }
    }

    pub(crate) fn subtitle(self) -> &'static str {
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
pub(crate) enum AncChoice {
    Off,
    Scene(AncScene),
    Transparency,
    /// Triple reported by the firmware that the interface cannot represent.
    /// Kept as its own choice so it is never silently converted into "off".
    Unknown([u8; 3]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VolumeOption(pub(crate) NotificationVolume);

impl VolumeOption {
    pub(crate) const ALL: [Self; 4] = [
        Self(NotificationVolume::Low),
        Self(NotificationVolume::Medium),
        Self(NotificationVolume::High),
        Self(NotificationVolume::Max),
    ];

    pub(crate) fn label(self) -> &'static str {
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
pub(crate) struct TouchOption(pub(crate) TouchFunction);

impl TouchOption {
    pub(crate) const ALL: [Self; 12] = [
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

    /// Options shown by the selector for the value currently stored.
    ///
    /// A function reported by the firmware that this interface does not know is
    /// appended to the list so the stored value is visible instead of leaving
    /// the selector blank. Choosing it writes the same byte back.
    pub(crate) fn options(current: TouchFunction) -> Vec<Self> {
        let mut options = Self::ALL.to_vec();
        if let TouchFunction::Unknown(value) = current {
            options.push(Self(TouchFunction::Unknown(value)));
        }
        options
    }
}

impl fmt::Display for TouchOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TouchFunction::None => formatter.write_str("Nenhuma"),
            TouchFunction::PlayPause => formatter.write_str("Reproduzir / pausar"),
            TouchFunction::Previous => formatter.write_str("Faixa anterior"),
            TouchFunction::Next => formatter.write_str("Próxima faixa"),
            TouchFunction::VoiceAssistant => formatter.write_str("Assistente de voz"),
            TouchFunction::VolumeUp => formatter.write_str("Aumentar volume"),
            TouchFunction::VolumeDown => formatter.write_str("Diminuir volume"),
            TouchFunction::ToggleGameMode => formatter.write_str("Alternar modo jogo"),
            TouchFunction::AnswerCall => formatter.write_str("Atender chamada"),
            TouchFunction::RejectCall => formatter.write_str("Rejeitar chamada"),
            TouchFunction::HoldCall => formatter.write_str("Segurar chamada"),
            TouchFunction::AncMode => formatter.write_str("Alternar ANC"),
            TouchFunction::Unknown(value) => write!(formatter, "Desconhecida (0x{value:02X})"),
        }
    }
}
