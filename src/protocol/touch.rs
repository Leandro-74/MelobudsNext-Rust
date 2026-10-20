use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;
use crate::protocol::TOUCH_MAPPING_LEN;

/// Touch key identifiers used by the mapping characteristic (`0000000D`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TouchKey {
    LeftSingle = 0x01,
    RightSingle = 0x02,
    LeftDouble = 0x03,
    RightDouble = 0x04,
    LeftTriple = 0x05,
    RightTriple = 0x06,
    LeftQuad = 0x07,
    RightQuad = 0x08,
}

impl TouchKey {
    pub const ALL: [Self; 8] = [
        Self::LeftSingle,
        Self::RightSingle,
        Self::LeftDouble,
        Self::RightDouble,
        Self::LeftTriple,
        Self::RightTriple,
        Self::LeftQuad,
        Self::RightQuad,
    ];

    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn index(self) -> usize {
        self.code() as usize - 1
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::LeftSingle),
            0x02 => Some(Self::RightSingle),
            0x03 => Some(Self::LeftDouble),
            0x04 => Some(Self::RightDouble),
            0x05 => Some(Self::LeftTriple),
            0x06 => Some(Self::RightTriple),
            0x07 => Some(Self::LeftQuad),
            0x08 => Some(Self::RightQuad),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::LeftSingle => "left-single",
            Self::RightSingle => "right-single",
            Self::LeftDouble => "left-double",
            Self::RightDouble => "right-double",
            Self::LeftTriple => "left-triple",
            Self::RightTriple => "right-triple",
            Self::LeftQuad => "left-quad",
            Self::RightQuad => "right-quad",
        }
    }
}

impl fmt::Display for TouchKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for TouchKey {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "left-single" | "left-single-tap" => Ok(Self::LeftSingle),
            "right-single" | "right-single-tap" => Ok(Self::RightSingle),
            "left-double" => Ok(Self::LeftDouble),
            "right-double" => Ok(Self::RightDouble),
            "left-triple" => Ok(Self::LeftTriple),
            "right-triple" => Ok(Self::RightTriple),
            "left-quad" => Ok(Self::LeftQuad),
            "right-quad" => Ok(Self::RightQuad),
            other => Err(format!("unknown touch key {other:?}")),
        }
    }
}

/// Functions assignable to a touch key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchFunction {
    #[default]
    None,
    PlayPause,
    Previous,
    Next,
    VoiceAssistant,
    VolumeUp,
    VolumeDown,
    ToggleGameMode,
    AnswerCall,
    RejectCall,
    HoldCall,
    AncMode,
    Unknown(u8),
}

impl TouchFunction {
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::None => 0x00,
            Self::PlayPause => 0x01,
            Self::Previous => 0x02,
            Self::Next => 0x03,
            Self::VoiceAssistant => 0x04,
            Self::VolumeUp => 0x05,
            Self::VolumeDown => 0x06,
            Self::ToggleGameMode => 0x07,
            Self::AnswerCall => 0x08,
            Self::RejectCall => 0x09,
            Self::HoldCall => 0x0A,
            Self::AncMode => 0x0B,
            Self::Unknown(value) => value,
        }
    }

    /// Unknown values are preserved instead of being silently discarded.
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x01 => Self::PlayPause,
            0x02 => Self::Previous,
            0x03 => Self::Next,
            0x04 => Self::VoiceAssistant,
            0x05 => Self::VolumeUp,
            0x06 => Self::VolumeDown,
            0x07 => Self::ToggleGameMode,
            0x08 => Self::AnswerCall,
            0x09 => Self::RejectCall,
            0x0A => Self::HoldCall,
            0x0B => Self::AncMode,
            other => Self::Unknown(other),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PlayPause => "play-pause",
            Self::Previous => "previous",
            Self::Next => "next",
            Self::VoiceAssistant => "voice-assistant",
            Self::VolumeUp => "volume-up",
            Self::VolumeDown => "volume-down",
            Self::ToggleGameMode => "toggle-game-mode",
            Self::AnswerCall => "answer-call",
            Self::RejectCall => "reject-call",
            Self::HoldCall => "hold-call",
            Self::AncMode => "anc-mode",
            Self::Unknown(_) => "unknown",
        }
    }
}

impl fmt::Display for TouchFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(value) => write!(f, "unknown(0x{value:02X})"),
            other => f.write_str(other.name()),
        }
    }
}

impl FromStr for TouchFunction {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();

        if let Some(hex) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            let value = u8::from_str_radix(hex, 16)
                .map_err(|_| format!("invalid touch function {input:?}"))?;
            return Ok(Self::from_u8(value));
        }

        match trimmed.to_ascii_lowercase().as_str() {
            "none" | "off" => Ok(Self::None),
            "play-pause" => Ok(Self::PlayPause),
            "previous" | "previous-track" => Ok(Self::Previous),
            "next" | "next-track" => Ok(Self::Next),
            "voice-assistant" | "assistant" => Ok(Self::VoiceAssistant),
            "volume-up" => Ok(Self::VolumeUp),
            "volume-down" => Ok(Self::VolumeDown),
            "toggle-game-mode" | "game-mode" => Ok(Self::ToggleGameMode),
            "answer-call" => Ok(Self::AnswerCall),
            "reject-call" => Ok(Self::RejectCall),
            "hold-call" => Ok(Self::HoldCall),
            "anc-mode" | "anc" => Ok(Self::AncMode),
            other => Err(format!("unknown touch function {other:?}")),
        }
    }
}

/// Full 20-byte touch mapping: eight `[key, function]` pairs plus four zero
/// padding bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchMapping {
    functions: [TouchFunction; 8],
}

impl Default for TouchMapping {
    fn default() -> Self {
        Self {
            functions: [TouchFunction::None; 8],
        }
    }
}

impl TouchMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: TouchKey) -> TouchFunction {
        self.functions[key.index()]
    }

    pub fn set(&mut self, key: TouchKey, function: TouchFunction) {
        self.functions[key.index()] = function;
    }

    pub fn functions(&self) -> &[TouchFunction; 8] {
        &self.functions
    }

    /// Encodes the complete block with all eight keys in fixed order.
    pub fn encode(&self) -> [u8; TOUCH_MAPPING_LEN] {
        let mut bytes = [0u8; TOUCH_MAPPING_LEN];

        for (index, key) in TouchKey::ALL.into_iter().enumerate() {
            bytes[index * 2] = key.code();
            bytes[index * 2 + 1] = self.get(key).as_u8();
        }

        bytes
    }

    /// Decodes any even-length block of `[key, function]` pairs. Pairs whose
    /// key byte is `0x00` are ignored and trailing pairs for unknown keys are
    /// preserved only when the key is recognized.
    ///
    /// Firmware normally reports 20 bytes (eight pairs plus padding), but some
    /// versions report only the pairs, so the length is not fixed.
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < 2 || bytes.len() % 2 != 0 {
            return Err(ProtocolError::InvalidTouchMappingLength(bytes.len()));
        }

        let mut mapping = Self::default();

        for pair in bytes.chunks_exact(2) {
            let key = pair[0];
            if key == 0x00 {
                continue;
            }

            if let Some(key) = TouchKey::from_u8(key) {
                mapping.set(key, TouchFunction::from_u8(pair[1]));
            }
        }

        Ok(mapping)
    }
}

impl fmt::Display for TouchMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in TouchKey::ALL {
            writeln!(f, "  {key}: {}", self.get(key))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: [u8; 20] = [
        0x01, 0x01, 0x02, 0x03, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00, 0x07, 0x00, 0x08,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn decodes_the_documented_example() {
        let mapping = TouchMapping::parse(&EXAMPLE).unwrap();
        assert_eq!(mapping.get(TouchKey::LeftSingle), TouchFunction::PlayPause);
        assert_eq!(mapping.get(TouchKey::RightSingle), TouchFunction::Next);
        assert_eq!(mapping.get(TouchKey::LeftDouble), TouchFunction::None);
    }

    #[test]
    fn encodes_the_documented_example() {
        let mut mapping = TouchMapping::new();
        mapping.set(TouchKey::LeftSingle, TouchFunction::PlayPause);
        mapping.set(TouchKey::RightSingle, TouchFunction::Next);
        assert_eq!(mapping.encode(), EXAMPLE);
    }

    #[test]
    fn ignores_zero_key_pairs() {
        let mapping = TouchMapping::parse(&[0u8; 20]).unwrap();
        for key in TouchKey::ALL {
            assert_eq!(mapping.get(key), TouchFunction::None);
        }
    }

    #[test]
    fn preserves_unknown_function_bytes() {
        let mut bytes = EXAMPLE;
        bytes[1] = 0x42;
        let mapping = TouchMapping::parse(&bytes).unwrap();
        assert_eq!(
            mapping.get(TouchKey::LeftSingle),
            TouchFunction::Unknown(0x42)
        );
        assert_eq!(mapping.encode(), bytes);
    }

    #[test]
    fn accepts_firmware_payloads_without_padding() {
        let mapping = TouchMapping::parse(&EXAMPLE[..16]).unwrap();
        assert_eq!(mapping.get(TouchKey::LeftSingle), TouchFunction::PlayPause);
        assert_eq!(mapping.get(TouchKey::RightSingle), TouchFunction::Next);
    }

    #[test]
    fn rejects_invalid_lengths() {
        assert_eq!(
            TouchMapping::parse(&[0x00; 19]).unwrap_err(),
            ProtocolError::InvalidTouchMappingLength(19)
        );
        assert_eq!(
            TouchMapping::parse(&[]).unwrap_err(),
            ProtocolError::InvalidTouchMappingLength(0)
        );
    }
}
