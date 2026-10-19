//! Helpers for parsing and formatting hexadecimal byte strings.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HexError {
    #[error("odd number of hexadecimal digits in {0:?}")]
    OddLength(String),

    #[error("invalid hexadecimal byte {0:?}")]
    InvalidByte(String),

    #[error("no hexadecimal bytes were provided")]
    Empty,
}

/// Parses a hexadecimal string into bytes.
///
/// Whitespace, commas, semicolons, colons, dashes and underscores are accepted
/// as separators. Tokens may be prefixed with `0x` and single-digit tokens are
/// interpreted as one byte (for example `F` becomes `0x0F`).
pub fn parse(input: &str) -> Result<Vec<u8>, HexError> {
    let normalized = input.replace([',', ';', ':', '-', '_'], " ");
    let mut bytes = Vec::new();

    for raw_token in normalized.split_whitespace() {
        let token = raw_token
            .strip_prefix("0x")
            .or_else(|| raw_token.strip_prefix("0X"))
            .unwrap_or(raw_token);

        if token.is_empty() {
            continue;
        }

        if !token.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(HexError::InvalidByte(raw_token.to_string()));
        }

        if token.len() == 1 {
            let byte = u8::from_str_radix(token, 16)
                .map_err(|_| HexError::InvalidByte(raw_token.to_string()))?;
            bytes.push(byte);
            continue;
        }

        if token.len() % 2 != 0 {
            return Err(HexError::OddLength(raw_token.to_string()));
        }

        for chunk in token.as_bytes().chunks(2) {
            let pair = std::str::from_utf8(chunk)
                .map_err(|_| HexError::InvalidByte(raw_token.to_string()))?;
            let byte = u8::from_str_radix(pair, 16)
                .map_err(|_| HexError::InvalidByte(raw_token.to_string()))?;
            bytes.push(byte);
        }
    }

    if bytes.is_empty() {
        return Err(HexError::Empty);
    }

    Ok(bytes)
}

/// Formats bytes as uppercase hexadecimal pairs separated by spaces.
pub fn format(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_several_input_styles() {
        assert_eq!(
            parse("FF 03 09 01 01").unwrap(),
            [0xFF, 0x03, 0x09, 0x01, 0x01]
        );
        assert_eq!(
            parse("ff,03,09,01,01").unwrap(),
            [0xFF, 0x03, 0x09, 0x01, 0x01]
        );
        assert_eq!(parse("0xFF 0x03 0x09").unwrap(), [0xFF, 0x03, 0x09]);
        assert_eq!(parse("FF03090101").unwrap(), [0xFF, 0x03, 0x09, 0x01, 0x01]);
        assert_eq!(parse("F A 0").unwrap(), [0x0F, 0x0A, 0x00]);
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(parse("").unwrap_err(), HexError::Empty);
        assert!(matches!(parse("ABC").unwrap_err(), HexError::OddLength(_)));
        assert!(matches!(parse("GG").unwrap_err(), HexError::InvalidByte(_)));
    }

    #[test]
    fn formats_bytes() {
        assert_eq!(format(&[0xFF, 0x03, 0x09]), "FF 03 09");
    }
}
