//! Wire protocol implementation.
//!
//! Commands use the `[0xFF, body_len, opcode, param_len, params...]` framing,
//! while touch mapping and battery/firmware readings use standalone formats.

pub mod anc;
pub mod commands;
pub mod framing;
pub mod opcode;
pub mod readings;
pub mod touch;

pub use anc::{AncMode, AncScene, TransparencyMode};
pub use commands::{NotificationVolume, WearDetection};
pub use framing::Packet;
pub use opcode::Opcode;
pub use readings::{Battery, BatteryLevel, FirmwareVersion};
pub use touch::{TouchFunction, TouchKey, TouchMapping};

/// Size of the touch mapping characteristic payload.
pub const TOUCH_MAPPING_LEN: usize = 20;

/// Maximum length of the device name accepted by opcode `0x18`.
pub const MAX_DEVICE_NAME_LEN: usize = 32;
