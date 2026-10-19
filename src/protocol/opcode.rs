use std::fmt;
use std::str::FromStr;

/// Opcodes defined by the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    GameMode = 0x09,
    SleepMode = 0x10,
    ChannelBalance = 0x16,
    Anc = 0x17,
    Rename = 0x18,
    NotificationVolume = 0x1D,
    Ldac = 0x23,
    AncWear = 0x28,
    WearDetection = 0x2C,
    Battery = 0x2F,
    Firmware = 0x30,
    Query = 0xFE,
}

impl Opcode {
    pub const ALL: [Self; 12] = [
        Self::GameMode,
        Self::SleepMode,
        Self::ChannelBalance,
        Self::Anc,
        Self::Rename,
        Self::NotificationVolume,
        Self::Ldac,
        Self::AncWear,
        Self::WearDetection,
        Self::Battery,
        Self::Firmware,
        Self::Query,
    ];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x09 => Some(Self::GameMode),
            0x10 => Some(Self::SleepMode),
            0x16 => Some(Self::ChannelBalance),
            0x17 => Some(Self::Anc),
            0x18 => Some(Self::Rename),
            0x1D => Some(Self::NotificationVolume),
            0x23 => Some(Self::Ldac),
            0x28 => Some(Self::AncWear),
            0x2C => Some(Self::WearDetection),
            0x2F => Some(Self::Battery),
            0x30 => Some(Self::Firmware),
            0xFE => Some(Self::Query),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::GameMode => "game-mode",
            Self::SleepMode => "sleep-mode",
            Self::ChannelBalance => "channel-balance",
            Self::Anc => "anc",
            Self::Rename => "rename",
            Self::NotificationVolume => "notification-volume",
            Self::Ldac => "ldac",
            Self::AncWear => "anc-wear",
            Self::WearDetection => "wear-detection",
            Self::Battery => "battery",
            Self::Firmware => "firmware",
            Self::Query => "query",
        }
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        opcode.as_u8()
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Opcode {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();

        if let Some(hex) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
            .or_else(|| (trimmed.len() == 2).then_some(trimmed))
        {
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                if let Some(opcode) = Self::from_u8(value) {
                    return Ok(opcode);
                }
            }
        }

        let normalized = trimmed.to_ascii_lowercase().replace(['-', '_', ' '], "");

        let opcode = match normalized.as_str() {
            "gamemode" | "game" => Self::GameMode,
            "sleepmode" | "sleep" => Self::SleepMode,
            "channelbalance" | "balance" => Self::ChannelBalance,
            "anc" => Self::Anc,
            "rename" => Self::Rename,
            "notificationvolume" | "notification" => Self::NotificationVolume,
            "ldac" => Self::Ldac,
            "ancwear" => Self::AncWear,
            "weardetection" | "wear" => Self::WearDetection,
            "battery" => Self::Battery,
            "firmware" | "version" => Self::Firmware,
            "query" => Self::Query,
            _ => {
                return Err(format!(
                    "unknown opcode {input:?}; use a number such as 0x17 or a known name"
                ))
            }
        };

        Ok(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_values_in_both_directions() {
        for opcode in Opcode::ALL {
            assert_eq!(Opcode::from_u8(opcode.as_u8()), Some(opcode));
        }
        assert_eq!(Opcode::from_u8(0x0C), None);
    }

    #[test]
    fn parses_names_and_numbers() {
        assert_eq!("0x17".parse::<Opcode>().unwrap(), Opcode::Anc);
        assert_eq!("17".parse::<Opcode>().unwrap(), Opcode::Anc);
        assert_eq!("game-mode".parse::<Opcode>().unwrap(), Opcode::GameMode);
        assert_eq!(
            "wear_detection".parse::<Opcode>().unwrap(),
            Opcode::WearDetection
        );
        assert!("0x0C".parse::<Opcode>().is_err());
    }
}
