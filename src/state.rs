use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;
use crate::protocol::anc::AncMode;
use crate::protocol::commands::{NotificationVolume, WearDetection};
use crate::protocol::framing::{self, Packet};
use crate::protocol::opcode::Opcode;
use crate::protocol::readings::{Battery, FirmwareVersion};
use crate::protocol::touch::TouchMapping;

/// In-memory state of the headphones.
///
/// The state is updated from command echoes, query responses and direct
/// characteristic reads. It is deliberately not persisted between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceState {
    pub game_mode: bool,
    pub sleep_mode: bool,
    pub channel_balance: u8,
    pub anc: AncMode,
    pub name: Option<String>,
    pub notification_volume: Option<u8>,
    pub ldac: bool,
    pub wear_detection: Option<WearDetection>,
    pub battery: Option<Battery>,
    pub firmware: Option<FirmwareVersion>,
    pub touch: Option<TouchMapping>,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            game_mode: false,
            sleep_mode: false,
            channel_balance: 50,
            anc: AncMode::Off,
            name: None,
            notification_volume: None,
            ldac: false,
            wear_detection: None,
            battery: None,
            firmware: None,
            touch: None,
        }
    }
}

impl DeviceState {
    /// Applies a single command-channel block. Returns `true` when the opcode
    /// was recognized and the state was updated.
    pub fn apply(&mut self, packet: &Packet) -> bool {
        let Some(opcode) = Opcode::from_u8(packet.opcode) else {
            return false;
        };

        match opcode {
            Opcode::GameMode => {
                let Some(value) = packet.params.first() else {
                    return false;
                };
                self.game_mode = *value == 0x01;
                true
            }
            Opcode::SleepMode => {
                let Some(value) = packet.params.first() else {
                    return false;
                };
                self.sleep_mode = *value == 0x01;
                true
            }
            Opcode::ChannelBalance => {
                let Some(value) = packet.params.first() else {
                    return false;
                };
                self.channel_balance = (*value).min(100);
                true
            }
            Opcode::Anc => match AncMode::from_params(&packet.params) {
                Ok(mode) => {
                    self.anc = mode;
                    true
                }
                Err(_) => false,
            },
            Opcode::Rename => {
                let name = String::from_utf8_lossy(&packet.params)
                    .trim_end_matches('\0')
                    .to_string();
                self.name = Some(name);
                true
            }
            Opcode::NotificationVolume => {
                let Some(value) = packet.params.first() else {
                    return false;
                };
                self.notification_volume = Some(*value);
                true
            }
            Opcode::Ldac => {
                let Some(value) = packet.params.first() else {
                    return false;
                };
                self.ldac = *value == 0x01;
                true
            }
            Opcode::WearDetection => match WearDetection::from_params(&packet.params) {
                Some(configuration) => {
                    self.wear_detection = Some(configuration);
                    true
                }
                None => false,
            },
            Opcode::AncWear | Opcode::Battery | Opcode::Firmware | Opcode::Query => false,
        }
    }

    /// Parses a notification and applies every valid block to the state.
    /// Returns the number of recognized blocks.
    pub fn apply_bytes(&mut self, bytes: &[u8]) -> Result<usize, ProtocolError> {
        let packets = framing::parse(bytes)?;
        Ok(packets.iter().filter(|packet| self.apply(packet)).count())
    }

    pub fn set_battery(&mut self, battery: Battery) {
        self.battery = Some(battery);
    }

    pub fn set_firmware(&mut self, firmware: FirmwareVersion) {
        self.firmware = Some(firmware);
    }

    pub fn set_touch(&mut self, touch: TouchMapping) {
        self.touch = Some(touch);
    }
}

fn option<T: fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

impl fmt::Display for DeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game mode: {}", on_off(self.game_mode))?;
        writeln!(f, "Sleep mode: {}", on_off(self.sleep_mode))?;
        writeln!(f, "Channel balance: {}", self.channel_balance)?;
        writeln!(f, "ANC: {}", self.anc)?;
        writeln!(f, "Name: {}", option(self.name.as_deref()))?;
        match self.notification_volume {
            Some(value) => writeln!(
                f,
                "Notification volume: {}",
                describe_notification_volume(value)
            )?,
            None => writeln!(f, "Notification volume: -")?,
        }
        writeln!(f, "LDAC: {}", on_off(self.ldac))?;

        match self.wear_detection {
            Some(configuration) => writeln!(
                f,
                "Wear detection: {}, music {}, keep ANC {}, tone {}",
                on_off(configuration.enabled),
                configuration.music_index,
                on_off(configuration.keep_anc),
                configuration.tone
            )?,
            None => writeln!(f, "Wear detection: -")?,
        }

        writeln!(f, "Battery: {}", option(self.battery))?;
        writeln!(f, "Firmware: {}", option(self.firmware))?;

        match &self.touch {
            Some(mapping) => {
                writeln!(f, "Touch mapping:")?;
                write!(f, "{mapping}")?;
            }
            None => writeln!(f, "Touch mapping: -")?,
        }

        Ok(())
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}

/// Formats the raw notification-volume byte together with its category.
pub fn describe_notification_volume(value: u8) -> String {
    match NotificationVolume::from_byte(value) {
        Some(level) => format!("0x{value:02X} ({level})"),
        None => format!("0x{value:02X}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::anc::{AncScene, TransparencyMode};
    use crate::protocol::readings::BatteryLevel;
    use crate::protocol::touch::TouchKey;
    use crate::protocol::TouchFunction;

    #[test]
    fn applies_a_composite_notification() {
        let bytes = [0xFF, 0x06, 0x09, 0x01, 0x01, 0x17, 0x03, 0x00, 0x00, 0x00];
        let mut state = DeviceState::default();
        assert_eq!(state.apply_bytes(&bytes).unwrap(), 2);
        assert!(state.game_mode);
        assert_eq!(state.anc, AncMode::Off);
    }

    #[test]
    fn applies_individual_commands() {
        let mut state = DeviceState::default();

        assert!(state.apply(&Packet::new(0x16, [0x3C])));
        assert_eq!(state.channel_balance, 60);

        assert!(state.apply(&Packet::new(0x17, [0x01, 0x02, 0x02])));
        assert_eq!(state.anc, AncMode::anc(AncScene::Daily, Some(3)).unwrap());

        assert!(state.apply(&Packet::new(0x18, b"Fone\0\0".to_vec())));
        assert_eq!(state.name.as_deref(), Some("Fone"));

        assert!(state.apply(&Packet::new(0x1D, [0x06, 0x06])));
        assert_eq!(state.notification_volume, Some(0x06));

        assert!(state.apply(&Packet::new(0x2C, [0x01, 0x01, 0x01])));
        assert_eq!(
            state.wear_detection,
            Some(WearDetection {
                enabled: true,
                music_index: 1,
                keep_anc: true,
                tone: 0,
            })
        );

        assert!(state.apply(&Packet::new(0x23, [0x01])));
        assert!(state.ldac);
    }

    #[test]
    fn ignores_unknown_or_uninterpreted_packets() {
        let mut state = DeviceState::default();
        assert!(!state.apply(&Packet::new(0x42, [0x01])));
        assert!(!state.apply(&Packet::new(0x28, [0x01, 0x02])));
    }

    #[test]
    fn accepts_three_byte_wear_detection_reports() {
        let mut state = DeviceState::default();
        assert!(state.apply(&Packet::new(0x2C, [0x01, 0x02, 0x00])));
        assert_eq!(state.wear_detection.unwrap().tone, 0);
    }

    #[test]
    fn stores_direct_readings_and_touch_mapping() {
        let mut state = DeviceState::default();
        state.set_battery(Battery::parse(&[0xD0, 0x46, 0x00]).unwrap());
        state.set_firmware(FirmwareVersion::parse(&[0x02, 0x00, 0x06]).unwrap());

        let mut touch = TouchMapping::new();
        touch.set(TouchKey::LeftSingle, TouchFunction::PlayPause);
        state.set_touch(touch);

        assert_eq!(state.battery.unwrap().left, BatteryLevel::new(0xD0));
        assert_eq!(state.firmware.unwrap().to_string(), "2.0.6");
        assert_eq!(
            state.touch.unwrap().get(TouchKey::LeftSingle),
            TouchFunction::PlayPause
        );
    }

    #[test]
    fn accepts_reported_anc_off_state() {
        let mut state = DeviceState::default();
        assert!(state.apply(&Packet::new(0x17, [0x02, 0x00, 0x00])));
        assert_eq!(state.anc, AncMode::Off);
    }

    #[test]
    fn decodes_transparency_state() {
        let mut state = DeviceState::default();
        assert!(state.apply(&Packet::new(0x17, [0x03, 0x01, 0x04])));
        assert_eq!(
            state.anc,
            AncMode::transparency(TransparencyMode::Intensity(4)).unwrap()
        );
    }

    #[test]
    fn describes_notification_volume() {
        assert_eq!(describe_notification_volume(0x06), "0x06 (medium)");
        assert_eq!(describe_notification_volume(0x05), "0x05");
    }
}
