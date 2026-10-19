use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;
use crate::protocol::anc::AncMode;
use crate::protocol::framing::Packet;
use crate::protocol::opcode::Opcode;
use crate::protocol::MAX_DEVICE_NAME_LEN;

const ENABLE: u8 = 0x01;
const DISABLE: u8 = 0x02;

fn toggle(opcode: Opcode, enabled: bool) -> Packet {
    Packet::new(opcode, [if enabled { ENABLE } else { DISABLE }])
}

/// Game Mode (`0x09`).
pub fn game_mode(enabled: bool) -> Packet {
    toggle(Opcode::GameMode, enabled)
}

/// Sleep Mode (`0x10`).
pub fn sleep_mode(enabled: bool) -> Packet {
    toggle(Opcode::SleepMode, enabled)
}

/// Channel balance (`0x16`). Values outside 0..=100 are clamped.
pub fn channel_balance(value: u8) -> Packet {
    Packet::new(Opcode::ChannelBalance, [value.min(100)])
}

/// ANC / Transparency (`0x17`).
pub fn anc(mode: AncMode) -> Packet {
    Packet::new(Opcode::Anc, mode.params())
}

/// Rename (`0x18`). The device name field is limited to 32 UTF-8 bytes.
pub fn rename(name: &str) -> Result<Packet, ProtocolError> {
    let bytes = name.as_bytes();
    if bytes.len() > MAX_DEVICE_NAME_LEN {
        return Err(ProtocolError::NameTooLong(bytes.len()));
    }
    Ok(Packet::new(Opcode::Rename, bytes))
}

/// LDAC (`0x23`).
pub fn ldac(enabled: bool) -> Packet {
    toggle(Opcode::Ldac, enabled)
}

/// State query (`0xFE`).
pub fn query(target: Opcode) -> Packet {
    Packet::new(Opcode::Query, [target.as_u8()])
}

/// Notification volume categories (`0x1D`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationVolume {
    Low,
    Medium,
    High,
    Max,
}

impl NotificationVolume {
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Max];

    pub const fn byte(self) -> u8 {
        match self {
            Self::Low => 0x04,
            Self::Medium => 0x06,
            Self::High => 0x08,
            Self::Max => 0x0A,
        }
    }

    pub const fn from_byte(value: u8) -> Option<Self> {
        match value {
            0x04 => Some(Self::Low),
            0x06 => Some(Self::Medium),
            0x08 => Some(Self::High),
            0x0A => Some(Self::Max),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }
}

impl fmt::Display for NotificationVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for NotificationVolume {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "low" | "1" => Ok(Self::Low),
            "medium" | "2" => Ok(Self::Medium),
            "high" | "3" => Ok(Self::High),
            "max" | "maximum" | "4" => Ok(Self::Max),
            other => Err(format!("unknown notification volume {other:?}")),
        }
    }
}

/// Notification volume (`0x1D`). The byte is duplicated on the wire.
pub fn notification_volume(level: NotificationVolume) -> Packet {
    let value = level.byte();
    Packet::new(Opcode::NotificationVolume, [value, value])
}

/// Wear detection configuration (`0x2C`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WearDetection {
    pub enabled: bool,
    pub music_index: u8,
    pub keep_anc: bool,
    pub tone: u8,
}

impl Default for WearDetection {
    fn default() -> Self {
        Self {
            enabled: false,
            music_index: 1,
            keep_anc: false,
            tone: 0,
        }
    }
}

impl WearDetection {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    /// Params are `[enable, music_index, anc_enable, tone]`.
    pub fn params(self) -> [u8; 4] {
        [
            u8::from(self.enabled),
            self.music_index,
            u8::from(self.keep_anc),
            self.tone,
        ]
    }

    /// Firmware may report three parameters, in which case `tone` is zero.
    pub fn from_params(params: &[u8]) -> Option<Self> {
        match params {
            [enabled, music_index, keep_anc] => Some(Self {
                enabled: *enabled == ENABLE,
                music_index: *music_index,
                keep_anc: *keep_anc == ENABLE,
                tone: 0,
            }),
            [enabled, music_index, keep_anc, tone] => Some(Self {
                enabled: *enabled == ENABLE,
                music_index: *music_index,
                keep_anc: *keep_anc == ENABLE,
                tone: *tone,
            }),
            _ => None,
        }
    }
}

/// Wear detection (`0x2C`).
pub fn wear_detection(configuration: WearDetection) -> Packet {
    Packet::new(Opcode::WearDetection, configuration.params())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::anc::{AncScene, TransparencyMode};

    #[test]
    fn builds_toggle_commands() {
        assert_eq!(game_mode(true).encode(), [0xFF, 0x03, 0x09, 0x01, 0x01]);
        assert_eq!(game_mode(false).encode(), [0xFF, 0x03, 0x09, 0x01, 0x02]);
        assert_eq!(sleep_mode(true).encode(), [0xFF, 0x03, 0x10, 0x01, 0x01]);
        assert_eq!(ldac(false).encode(), [0xFF, 0x03, 0x23, 0x01, 0x02]);
    }

    #[test]
    fn builds_the_balance_command() {
        assert_eq!(channel_balance(60).encode(), [0xFF, 0x03, 0x16, 0x01, 0x3C]);
        assert_eq!(channel_balance(200).encode()[4], 100);
    }

    #[test]
    fn builds_anc_commands() {
        assert_eq!(
            anc(AncMode::Off).encode(),
            [0xFF, 0x05, 0x17, 0x03, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            anc(AncMode::anc(AncScene::Indoor, Some(2)).unwrap()).encode(),
            [0xFF, 0x05, 0x17, 0x03, 0x01, 0x01, 0x01]
        );
        assert_eq!(
            anc(AncMode::transparency(TransparencyMode::Intensity(4)).unwrap()).encode(),
            [0xFF, 0x05, 0x17, 0x03, 0x03, 0x01, 0x04]
        );
    }

    #[test]
    fn builds_rename_commands() {
        assert_eq!(
            rename("Fone").unwrap().encode(),
            [0xFF, 0x06, 0x18, 0x04, 0x46, 0x6F, 0x6E, 0x65]
        );
        assert_eq!(
            rename(&"x".repeat(33)).unwrap_err(),
            ProtocolError::NameTooLong(33)
        );
    }

    #[test]
    fn builds_notification_volume_commands() {
        assert_eq!(
            notification_volume(NotificationVolume::Medium).encode(),
            [0xFF, 0x04, 0x1D, 0x02, 0x06, 0x06]
        );
        for level in NotificationVolume::ALL {
            assert_eq!(NotificationVolume::from_byte(level.byte()), Some(level));
        }
    }

    #[test]
    fn builds_wear_detection_commands() {
        let configuration = WearDetection {
            enabled: true,
            music_index: 1,
            keep_anc: true,
            tone: 0,
        };
        assert_eq!(
            wear_detection(configuration).encode(),
            [0xFF, 0x06, 0x2C, 0x04, 0x01, 0x01, 0x01, 0x00]
        );
        assert_eq!(
            WearDetection::from_params(&[0x01, 0x01, 0x01]),
            Some(configuration)
        );
    }

    #[test]
    fn builds_query_commands() {
        assert_eq!(query(Opcode::Anc).encode(), [0xFF, 0x03, 0xFE, 0x01, 0x17]);
    }
}
