//! Shared action layer used by the CLI and the GUI.
//!
//! The action layer talks to the headphones through [`Transport`], a small
//! object-safe trait implemented by [`BleSession`]. Keeping the orchestration
//! behind this seam makes it possible to test connection, synchronisation and
//! command flows with a transport double instead of real hardware.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;

use crate::ble::{BleSession, SyncReport};
use crate::protocol::framing::Packet;
use crate::protocol::touch::TouchMapping;

/// A boxed future returned by [`Transport`] methods.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Operations the action layer needs from a BLE link.
///
/// The trait is intentionally object-safe (`Box<dyn Transport>`) so a session
/// can own the real [`BleSession`] or a test double interchangeably.
pub trait Transport: Send {
    /// Address reported by the connected peripheral.
    fn address(&self) -> String;

    /// Whether the underlying link is still up.
    fn is_connected<'a>(&'a self) -> BoxFuture<'a, Result<bool>>;

    /// Reads the direct characteristics and queries the remaining state.
    fn sync<'a>(&'a self, listen: Duration) -> BoxFuture<'a, Result<SyncReport>>;

    /// Writes a packet and collects the notifications it triggers.
    fn exchange<'a>(
        &'a self,
        packet: &'a Packet,
        listen: Duration,
    ) -> BoxFuture<'a, Result<Vec<Vec<u8>>>>;

    /// Reconnects after a link drop, rediscovering services.
    fn reconnect<'a>(&'a mut self, attempts: u32, delay: Duration) -> BoxFuture<'a, Result<()>>;

    /// Writes a complete 20-byte touch mapping.
    fn write_touch<'a>(&'a self, mapping: &'a TouchMapping) -> BoxFuture<'a, Result<()>>;

    /// Closes the BLE link.
    fn disconnect<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
}

impl Transport for BleSession {
    fn address(&self) -> String {
        BleSession::address(self)
    }

    fn is_connected<'a>(&'a self) -> BoxFuture<'a, Result<bool>> {
        Box::pin(BleSession::is_connected(self))
    }

    fn sync<'a>(&'a self, listen: Duration) -> BoxFuture<'a, Result<SyncReport>> {
        Box::pin(BleSession::sync(self, listen))
    }

    fn exchange<'a>(
        &'a self,
        packet: &'a Packet,
        listen: Duration,
    ) -> BoxFuture<'a, Result<Vec<Vec<u8>>>> {
        Box::pin(BleSession::exchange(self, packet, listen))
    }

    fn reconnect<'a>(&'a mut self, attempts: u32, delay: Duration) -> BoxFuture<'a, Result<()>> {
        Box::pin(BleSession::reconnect(self, attempts, delay))
    }

    fn write_touch<'a>(&'a self, mapping: &'a TouchMapping) -> BoxFuture<'a, Result<()>> {
        Box::pin(BleSession::write_touch(self, mapping))
    }

    fn disconnect<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(BleSession::disconnect(self))
    }
}
