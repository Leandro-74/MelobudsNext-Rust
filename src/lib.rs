//! Core implementation of the QCY Melobuds Pro BLE protocol.
//!
//! The crate is intentionally split between the wire protocol, the in-memory
//! device state and configuration. A transport layer (BLE) can be added on top
//! without changing the protocol primitives.

pub mod ble;
pub mod config;
pub mod error;
pub mod hex;
pub mod protocol;
pub mod state;

pub use ble::BleSession;
pub use error::ProtocolError;
pub use state::DeviceState;
