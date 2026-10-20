use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;

/// One battery reading. Bit 7 is the charging flag and bits 0..=6 hold the
/// percentage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLevel {
    pub raw: u8,
}

impl BatteryLevel {
    pub const fn new(raw: u8) -> Self {
        Self { raw }
    }

    pub const fn percent(self) -> u8 {
        self.raw & 0x7F
    }

    pub const fn charging(self) -> bool {
        self.raw & 0x80 != 0
    }

    pub const fn is_unknown(self) -> bool {
        self.percent() == 0
    }
}

impl fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unknown() {
            f.write_str("-/-")
        } else {
            write!(f, "{}%", self.percent())?;
            if self.charging() {
                f.write_str(" (charging)")?;
            }
            Ok(())
        }
    }
}

/// Battery payload read from characteristic `00000008`: `[left, right, case]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battery {
    pub left: BatteryLevel,
    pub right: BatteryLevel,
    pub case: BatteryLevel,
}

impl Battery {
    /// Parses `[left, right, case]`.
    ///
    /// The reference implementation accepts payloads without the case byte, so
    /// only the two earbuds are required; a missing case is reported as unknown
    /// (`-/-`).
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        match bytes {
            [left, right, case, ..] => Ok(Self {
                left: BatteryLevel::new(*left),
                right: BatteryLevel::new(*right),
                case: BatteryLevel::new(*case),
            }),
            [left, right] => Ok(Self {
                left: BatteryLevel::new(*left),
                right: BatteryLevel::new(*right),
                case: BatteryLevel::new(0),
            }),
            _ => Err(ProtocolError::InvalidBatteryLength(bytes.len())),
        }
    }
}

impl fmt::Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "left: {}, right: {}, case: {}",
            self.left, self.right, self.case
        )
    }
}

/// Firmware payload read from characteristic `00000007`: `[major, minor, patch]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl FirmwareVersion {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if let [major, minor, patch, ..] = bytes {
            Ok(Self {
                major: *major,
                minor: *minor,
                patch: *patch,
            })
        } else {
            Err(ProtocolError::InvalidFirmwareLength(bytes.len()))
        }
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_battery_flags_and_levels() {
        let battery = Battery::parse(&[0xD0, 0x46, 0x00]).unwrap();
        assert_eq!(battery.left.percent(), 80);
        assert!(battery.left.charging());
        assert_eq!(battery.right.percent(), 70);
        assert!(!battery.right.charging());
        assert!(battery.case.is_unknown());
    }

    #[test]
    fn accepts_battery_payloads_without_the_case() {
        let battery = Battery::parse(&[0x50, 0x46]).unwrap();
        assert_eq!(battery.left.percent(), 80);
        assert_eq!(battery.right.percent(), 70);
        assert!(battery.case.is_unknown());
    }

    #[test]
    fn rejects_short_battery_payloads() {
        assert_eq!(
            Battery::parse(&[0x50]).unwrap_err(),
            ProtocolError::InvalidBatteryLength(1)
        );
        assert_eq!(
            Battery::parse(&[]).unwrap_err(),
            ProtocolError::InvalidBatteryLength(0)
        );
    }

    #[test]
    fn parses_and_formats_firmware() {
        let version = FirmwareVersion::parse(&[0x02, 0x00, 0x06]).unwrap();
        assert_eq!(version.to_string(), "2.0.6");
        assert_eq!(
            FirmwareVersion::parse(&[0x02]).unwrap_err(),
            ProtocolError::InvalidFirmwareLength(1)
        );
    }
}
