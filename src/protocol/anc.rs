use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;

/// ANC scenes supported by opcode `0x17` when `mode = 0x01`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncScene {
    Indoor,
    Daily,
    Noise,
    Wind,
    Adaptive,
}

impl AncScene {
    pub const ALL: [Self; 5] = [
        Self::Indoor,
        Self::Daily,
        Self::Noise,
        Self::Wind,
        Self::Adaptive,
    ];

    pub const fn sub(self) -> u8 {
        match self {
            Self::Indoor => 0x01,
            Self::Daily => 0x02,
            Self::Noise => 0x03,
            Self::Wind => 0x04,
            Self::Adaptive => 0x05,
        }
    }

    pub const fn supports_levels(self) -> bool {
        matches!(self, Self::Indoor | Self::Daily | Self::Noise)
    }

    pub const fn from_sub(sub: u8) -> Option<Self> {
        match sub {
            0x01 => Some(Self::Indoor),
            0x02 => Some(Self::Daily),
            0x03 => Some(Self::Noise),
            0x04 => Some(Self::Wind),
            0x05 => Some(Self::Adaptive),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Indoor => "indoor",
            Self::Daily => "daily",
            Self::Noise => "noise",
            Self::Wind => "wind",
            Self::Adaptive => "adaptive",
        }
    }
}

impl fmt::Display for AncScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for AncScene {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "indoor" => Ok(Self::Indoor),
            "daily" => Ok(Self::Daily),
            "noise" => Ok(Self::Noise),
            "wind" => Ok(Self::Wind),
            "adaptive" => Ok(Self::Adaptive),
            other => Err(format!("unknown ANC scene {other:?}")),
        }
    }
}

/// Transparency behavior for `mode = 0x03`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransparencyMode {
    VoiceEnhancement,
    Intensity(u8),
}

/// Complete ANC/transparency state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncMode {
    #[default]
    Off,
    Anc {
        scene: AncScene,
        level: Option<u8>,
    },
    Transparency {
        mode: TransparencyMode,
    },
    /// Triples reported by the firmware that this implementation does not
    /// understand. They are preserved so live state is never silently dropped.
    Unknown([u8; 3]),
}

impl AncMode {
    /// Builds an ANC scene, validating whether the scene accepts a level.
    pub fn anc(scene: AncScene, level: Option<u8>) -> Result<Self, ProtocolError> {
        if scene.supports_levels() {
            let level = level.ok_or(ProtocolError::AncLevelRequired)?;
            if !(1..=3).contains(&level) {
                return Err(ProtocolError::InvalidAncLevel(level));
            }
            Ok(Self::Anc {
                scene,
                level: Some(level),
            })
        } else {
            if level.is_some() {
                return Err(ProtocolError::AncLevelNotSupported);
            }
            Ok(Self::Anc { scene, level: None })
        }
    }

    /// Builds a transparency mode, validating the intensity range.
    pub fn transparency(mode: TransparencyMode) -> Result<Self, ProtocolError> {
        if let TransparencyMode::Intensity(intensity) = mode {
            if !(1..=6).contains(&intensity) {
                return Err(ProtocolError::InvalidTransparencyIntensity(intensity));
            }
        }
        Ok(Self::Transparency { mode })
    }

    /// Encodes the three parameters `[mode, sub, noise]`.
    pub fn params(self) -> [u8; 3] {
        match self {
            Self::Off => [0x00, 0x00, 0x00],
            Self::Anc { scene, level } => [
                0x01,
                scene.sub(),
                level.map(|value| value - 1).unwrap_or(0x00),
            ],
            Self::Transparency { mode } => [
                0x03,
                0x01,
                match mode {
                    TransparencyMode::VoiceEnhancement => 0x00,
                    TransparencyMode::Intensity(intensity) => intensity,
                },
            ],
            Self::Unknown(payload) => payload,
        }
    }

    /// Decodes `[mode, sub, noise]`, accepting `02 00 00` as the reported
    /// "off" state used by firmware 2.0.6.
    ///
    /// Decoding is lenient: combinations that are not documented are kept in
    /// [`AncMode::Unknown`] instead of failing, mirroring the reference
    /// implementation. Validation for outgoing commands lives in [`Self::anc`]
    /// and [`Self::transparency`].
    pub fn from_params(params: &[u8]) -> Result<Self, ProtocolError> {
        if params.len() < 3 {
            return Err(ProtocolError::InvalidAncLength(params.len()));
        }

        let payload = [params[0], params[1], params[2]];

        if payload[0] == 0x00 || payload == [0x02, 0x00, 0x00] {
            return Ok(Self::Off);
        }

        if let [0x01, sub, noise] = payload {
            if let Some(scene) = AncScene::from_sub(sub) {
                if scene.supports_levels() && noise <= 2 {
                    return Ok(Self::Anc {
                        scene,
                        level: Some(noise + 1),
                    });
                }
                if !scene.supports_levels() && noise == 0 {
                    return Ok(Self::Anc { scene, level: None });
                }
            }
        }

        if let [0x03, 0x01, noise] = payload {
            match noise {
                0x00 => {
                    return Ok(Self::Transparency {
                        mode: TransparencyMode::VoiceEnhancement,
                    })
                }
                0x01..=0x06 => {
                    return Ok(Self::Transparency {
                        mode: TransparencyMode::Intensity(noise),
                    })
                }
                _ => {}
            }
        }

        Ok(Self::Unknown(payload))
    }
}

impl fmt::Display for AncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => f.write_str("off"),
            Self::Anc {
                scene,
                level: Some(level),
            } => write!(f, "anc/{scene}/level-{level}"),
            Self::Anc { scene, level: None } => write!(f, "anc/{scene}"),
            Self::Transparency { mode } => match mode {
                TransparencyMode::VoiceEnhancement => f.write_str("transparency/voice"),
                TransparencyMode::Intensity(intensity) => {
                    write!(f, "transparency/intensity-{intensity}")
                }
            },
            Self::Unknown(payload) => write!(
                f,
                "unknown({:02X} {:02X} {:02X})",
                payload[0], payload[1], payload[2]
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_documented_anc_packets() {
        assert_eq!(AncMode::Off.params(), [0x00, 0x00, 0x00]);
        assert_eq!(
            AncMode::anc(AncScene::Indoor, Some(2)).unwrap().params(),
            [0x01, 0x01, 0x01]
        );
        assert_eq!(
            AncMode::transparency(TransparencyMode::Intensity(4))
                .unwrap()
                .params(),
            [0x03, 0x01, 0x04]
        );
        assert_eq!(
            AncMode::transparency(TransparencyMode::VoiceEnhancement)
                .unwrap()
                .params(),
            [0x03, 0x01, 0x00]
        );
    }

    #[test]
    fn validates_levels_and_intensities() {
        assert_eq!(
            AncMode::anc(AncScene::Indoor, None).unwrap_err(),
            ProtocolError::AncLevelRequired
        );
        assert_eq!(
            AncMode::anc(AncScene::Indoor, Some(4)).unwrap_err(),
            ProtocolError::InvalidAncLevel(4)
        );
        assert_eq!(
            AncMode::anc(AncScene::Wind, Some(1)).unwrap_err(),
            ProtocolError::AncLevelNotSupported
        );
        assert_eq!(
            AncMode::transparency(TransparencyMode::Intensity(0)).unwrap_err(),
            ProtocolError::InvalidTransparencyIntensity(0)
        );
    }

    #[test]
    fn parses_off_reports() {
        assert_eq!(
            AncMode::from_params(&[0x00, 0x00, 0x00]).unwrap(),
            AncMode::Off
        );
        assert_eq!(
            AncMode::from_params(&[0x02, 0x00, 0x00]).unwrap(),
            AncMode::Off
        );
    }

    #[test]
    fn round_trips_all_modes() {
        let modes = [
            AncMode::Off,
            AncMode::anc(AncScene::Indoor, Some(1)).unwrap(),
            AncMode::anc(AncScene::Daily, Some(3)).unwrap(),
            AncMode::anc(AncScene::Noise, Some(2)).unwrap(),
            AncMode::anc(AncScene::Wind, None).unwrap(),
            AncMode::anc(AncScene::Adaptive, None).unwrap(),
            AncMode::transparency(TransparencyMode::VoiceEnhancement).unwrap(),
            AncMode::transparency(TransparencyMode::Intensity(6)).unwrap(),
            AncMode::Unknown([0x01, 0x09, 0x00]),
            AncMode::Unknown([0x03, 0x01, 0x07]),
        ];

        for mode in modes {
            assert_eq!(AncMode::from_params(&mode.params()), Ok(mode));
        }
    }

    #[test]
    fn preserves_unknown_payloads_instead_of_failing() {
        for payload in [
            [0x01, 0x09, 0x00],
            [0x01, 0x01, 0x03],
            [0x02, 0x01, 0x00],
            [0x03, 0x02, 0x00],
            [0x03, 0x01, 0x07],
            [0x04, 0x00, 0x00],
        ] {
            assert_eq!(
                AncMode::from_params(&payload).unwrap(),
                AncMode::Unknown(payload)
            );
        }
    }

    #[test]
    fn rejects_truncated_payloads() {
        assert_eq!(
            AncMode::from_params(&[0x00]).unwrap_err(),
            ProtocolError::InvalidAncLength(1)
        );
        assert_eq!(
            AncMode::from_params(&[0x01, 0x02]).unwrap_err(),
            ProtocolError::InvalidAncLength(2)
        );
    }
}
