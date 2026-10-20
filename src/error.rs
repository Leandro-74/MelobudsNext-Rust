use thiserror::Error;

/// Errors produced while encoding or decoding protocol data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("empty packet")]
    EmptyPacket,

    #[error("truncated packet header: expected 2 bytes")]
    TruncatedHeader,

    #[error("invalid packet header 0x{0:02X}; expected 0xFF")]
    InvalidHeader(u8),

    #[error("name is too long: {0} bytes (maximum is 32)")]
    NameTooLong(usize),

    #[error("invalid ANC payload length: {0} (expected 3)")]
    InvalidAncLength(usize),

    #[error("invalid ANC payload: {0:02X?}")]
    InvalidAnc([u8; 3]),

    #[error("invalid ANC level {0}; expected a value in 1..=3")]
    InvalidAncLevel(u8),

    #[error("ANC scene does not support levels")]
    AncLevelNotSupported,

    #[error("ANC scene requires a level in 1..=3")]
    AncLevelRequired,

    #[error("invalid transparency intensity {0}; expected a value in 1..=6")]
    InvalidTransparencyIntensity(u8),

    #[error("invalid battery payload length: {0} (expected at least 2)")]
    InvalidBatteryLength(usize),

    #[error("invalid firmware payload length: {0} (expected 3)")]
    InvalidFirmwareLength(usize),

    #[error("invalid touch mapping length: {0} (expected an even number of bytes, normally 20)")]
    InvalidTouchMappingLength(usize),
}
