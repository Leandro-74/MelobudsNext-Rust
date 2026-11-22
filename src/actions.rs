//! Shared action layer used by the CLI and the GUI.
//!
//! The action layer talks to the headphones through [`Transport`], a small
//! object-safe trait implemented by [`BleSession`]. Keeping the orchestration
//! behind this seam makes it possible to test connection, synchronisation and
//! command flows with a transport double instead of real hardware.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::ble::{BleSession, SyncReport};
use crate::config::BleUuids;
use crate::hex;
use crate::protocol::commands;
use crate::protocol::framing::Packet;
use crate::protocol::touch::TouchMapping;
use crate::state::DeviceState;

/// Window used to synchronise the full device state.
pub const SYNC_WINDOW: Duration = Duration::from_millis(1500);
/// Window used by the GUI to wait for command echoes.
pub const COMMAND_WINDOW: Duration = Duration::from_millis(450);
/// Scan window used when a session connects.
pub const SCAN_TIMEOUT: Duration = Duration::from_secs(8);
/// Attempts made when the headphones restart (for example after LDAC).
pub const RECONNECT_ATTEMPTS: u32 = 10;
/// Delay between reconnection attempts.
pub const RECONNECT_DELAY: Duration = Duration::from_secs(3);

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

/// Creates the transport used by a [`Controller`].
///
/// The connector is injectable so tests can supply a transport double without
/// touching btleplug or the Bluetooth adapter.
pub trait Connector: Send + Sync {
    /// Connects to `address` and returns an open transport.
    fn connect<'a>(
        &'a self,
        address: &'a str,
        uuids: &'a BleUuids,
        scan_timeout: Duration,
    ) -> BoxFuture<'a, Result<Box<dyn Transport>>>;
}

/// Connector backed by the real [`BleSession`] transport.
#[derive(Debug, Default)]
pub struct BleConnector;

impl Connector for BleConnector {
    fn connect<'a>(
        &'a self,
        address: &'a str,
        uuids: &'a BleUuids,
        scan_timeout: Duration,
    ) -> BoxFuture<'a, Result<Box<dyn Transport>>> {
        Box::pin(async move {
            let session = BleSession::connect(address, uuids, scan_timeout).await?;
            Ok(Box::new(session) as Box<dyn Transport>)
        })
    }
}

/// Failure while opening a session.
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("failed to connect")]
    Connect(#[source] anyhow::Error),

    #[error("connected but failed to synchronize the state")]
    Sync(#[source] anyhow::Error),
}

/// How a command that is not echoed by the firmware updates the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoPolicy {
    /// Only notifications update the state (CLI behaviour).
    EchoesOnly,
    /// Apply the requested packet when the firmware stays silent (GUI).
    ApplyRequestedWhenSilent,
}

/// Options for one command exchange.
#[derive(Debug, Clone, Copy)]
pub struct CommandOptions {
    /// Time spent listening for the command echoes.
    pub listen: Duration,
    /// Time spent synchronising the state after a reconnection.
    pub sync_window: Duration,
    /// Whether the command restarts the headphones and requires a reconnection.
    pub reconnect: bool,
    /// How to handle a command that is not echoed.
    pub echo_policy: EchoPolicy,
}

impl CommandOptions {
    /// Options used by the GUI, which keeps the requested value without an echo.
    pub fn gui(reconnect: bool) -> Self {
        Self {
            listen: COMMAND_WINDOW,
            sync_window: SYNC_WINDOW,
            reconnect,
            echo_policy: EchoPolicy::ApplyRequestedWhenSilent,
        }
    }

    /// Options used by the CLI, which only applies the echoes it receives.
    pub fn cli(listen: Duration, reconnect: bool) -> Self {
        Self {
            listen,
            sync_window: SYNC_WINDOW,
            reconnect,
            echo_policy: EchoPolicy::EchoesOnly,
        }
    }
}

/// Result of one command exchange.
#[derive(Debug, Default)]
pub struct CommandOutcome {
    /// Notifications collected while waiting for the echoes.
    pub notifications: Vec<Vec<u8>>,
    /// Non-fatal problems found while applying the command.
    pub warnings: Vec<String>,
    /// Whether the command triggered a reconnection.
    pub reconnected: bool,
}

/// Receives progress notifications while a command is being executed.
///
/// The CLI uses it to print the echoes and the reconnection message in the same
/// order as before; the GUI uses the no-op implementation.
pub trait CommandObserver: Send {
    fn notification(&mut self, _value: &[u8]) {}

    fn reconnecting(&mut self) {}
}

/// Observer used when the caller only needs the final result.
pub struct NoopObserver;

impl CommandObserver for NoopObserver {}

/// Whether the session must discard its client after a failed action.
///
/// The client is dropped unless the peripheral explicitly confirms that the
/// link is still up; an inconclusive probe is treated as a lost link so the UI
/// never keeps showing "Conectado" for a session that cannot be used.
pub fn discards_client(link_alive: Option<bool>) -> bool {
    link_alive != Some(true)
}

/// Applies command echoes to the state, keeping the CLI warning format.
pub fn apply_echoes(state: &mut DeviceState, values: &[Vec<u8>]) -> Vec<String> {
    let mut warnings = Vec::new();

    for value in values {
        if let Err(error) = state.apply_bytes(value) {
            warnings.push(format!("notification {}: {error}", hex::format(value)));
        }
    }

    warnings
}

/// Applies the notifications produced by a command to the state.
///
/// Echoes are authoritative, but the firmware can stay silent when the value
/// did not change. When no notification is recognized, the requested packet is
/// applied instead, so the GUI keeps the value the user chose instead of
/// reverting the control after a successful command.
///
/// Returns one warning per notification that could not be decoded.
pub fn apply_command_result(
    state: &mut DeviceState,
    packet: &Packet,
    values: &[Vec<u8>],
) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut recognized = 0usize;

    for value in values {
        match state.apply_bytes(value) {
            Ok(count) => recognized += count,
            Err(error) => warnings.push(format!("notificação inválida: {error}")),
        }
    }

    if recognized == 0 {
        state.apply(packet);
    }

    warnings
}

/// Owns one BLE session and the shared action orchestration.
///
/// The controller keeps the link open between actions (as the GUI needs) and
/// exposes the same `sync → send → apply → reconnect` flow used by the CLI.
pub struct Controller {
    connector: Arc<dyn Connector>,
    target: Option<String>,
    uuids: BleUuids,
    scan_timeout: Duration,
    state: DeviceState,
    client: Option<Box<dyn Transport>>,
}

impl Controller {
    /// Builds a controller backed by the real BLE transport.
    pub fn new(uuids: BleUuids, target: Option<String>) -> Self {
        Self::with_connector(Arc::new(BleConnector), uuids, target)
    }

    /// Builds a controller with an injected connector, used by tests.
    pub fn with_connector(
        connector: Arc<dyn Connector>,
        uuids: BleUuids,
        target: Option<String>,
    ) -> Self {
        Self {
            connector,
            target,
            uuids,
            scan_timeout: SCAN_TIMEOUT,
            state: DeviceState::default(),
            client: None,
        }
    }

    pub fn set_target(&mut self, target: impl Into<String>) {
        self.target = Some(target.into());
    }

    /// Overrides the scan window used by the next connection.
    pub fn set_scan_timeout(&mut self, scan_timeout: Duration) {
        self.scan_timeout = scan_timeout;
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn state(&self) -> &DeviceState {
        &self.state
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Address of the open link, or the last target when disconnected.
    pub fn address(&self) -> Option<String> {
        self.client
            .as_ref()
            .map(|client| client.address())
            .or_else(|| self.target.clone())
    }

    fn client_ref(&self) -> Result<&dyn Transport> {
        self.client
            .as_deref()
            .ok_or_else(|| anyhow!("device not connected"))
    }

    /// Connects to `address`, keeps it as the target and synchronises the state.
    pub async fn connect_to(&mut self, address: &str) -> Result<Vec<String>, ConnectError> {
        self.target = Some(address.to_string());

        if let Some(client) = self.client.take() {
            let _ = client.disconnect().await;
        }

        let client = self
            .connector
            .connect(address, &self.uuids, self.scan_timeout)
            .await
            .map_err(ConnectError::Connect)?;

        let report = match client.sync(SYNC_WINDOW).await {
            Ok(report) => report,
            Err(error) => {
                let _ = client.disconnect().await;
                return Err(ConnectError::Sync(error));
            }
        };

        self.state = report.state;
        self.target = Some(client.address());
        self.client = Some(client);

        Ok(report.warnings)
    }

    /// Re-reads the state of the current link.
    pub async fn sync_current(&mut self) -> Result<Vec<String>> {
        let report = {
            let client = self.client_ref()?;
            client.sync(SYNC_WINDOW).await?
        };
        self.state = report.state;
        Ok(report.warnings)
    }

    /// Closes the link, if any.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(client) = self.client.take() {
            client.disconnect().await?;
        }
        Ok(())
    }

    /// Sends a command and applies the result to the state.
    pub async fn command(
        &mut self,
        packet: &Packet,
        options: &CommandOptions,
    ) -> Result<CommandOutcome> {
        self.command_with(packet, options, &mut NoopObserver).await
    }

    /// Sends a command, reporting progress to `observer`.
    pub async fn command_with(
        &mut self,
        packet: &Packet,
        options: &CommandOptions,
        observer: &mut dyn CommandObserver,
    ) -> Result<CommandOutcome> {
        let values = {
            let client = self.client_ref()?;
            client.exchange(packet, options.listen).await?
        };

        for value in &values {
            observer.notification(value);
        }

        let mut warnings = Vec::new();

        // The CLI keeps the pre-reconnection state when the link is rebuilt;
        // the GUI applies the result after the reconnection sync below.
        if options.echo_policy == EchoPolicy::EchoesOnly {
            warnings.extend(apply_echoes(&mut self.state, &values));
        }

        if options.reconnect {
            observer.reconnecting();

            let report = {
                let client = self
                    .client
                    .as_mut()
                    .ok_or_else(|| anyhow!("device not connected"))?;
                client
                    .reconnect(RECONNECT_ATTEMPTS, RECONNECT_DELAY)
                    .await?;
                client.sync(options.sync_window.max(options.listen)).await?
            };

            self.state = report.state;
            if options.echo_policy == EchoPolicy::EchoesOnly {
                // The CLI historically reports only the post-reconnection
                // warnings; keep that output unchanged.
                warnings = report.warnings;
            } else {
                warnings.extend(report.warnings);
            }
        }

        if options.echo_policy == EchoPolicy::ApplyRequestedWhenSilent {
            warnings.extend(apply_command_result(&mut self.state, packet, &values));
        }

        Ok(CommandOutcome {
            notifications: values,
            warnings,
            reconnected: options.reconnect,
        })
    }

    /// Renames the device, keeping the requested name when it is not echoed.
    pub async fn rename(&mut self, name: &str) -> Result<CommandOutcome> {
        let packet = commands::rename(name).map_err(anyhow::Error::new)?;
        let values = {
            let client = self.client_ref()?;
            client.exchange(&packet, COMMAND_WINDOW).await?
        };

        let warnings = apply_command_result(&mut self.state, &packet, &values);

        // Some firmware revisions do not echo the rename; keep the requested
        // name in that case.
        if self.state.name.as_deref() != Some(name) {
            self.state.name = Some(name.to_string());
        }

        Ok(CommandOutcome {
            notifications: values,
            warnings,
            reconnected: false,
        })
    }

    /// Writes a complete touch mapping and updates the in-memory state.
    pub async fn write_touch(&mut self, mapping: &TouchMapping) -> Result<()> {
        let client = self.client_ref()?;
        client.write_touch(mapping).await?;
        self.state.touch = Some(mapping.clone());
        Ok(())
    }

    /// Discards the client when a failed action also lost the BLE link.
    pub async fn drop_client_if_link_lost(&mut self) {
        let Some(client) = self.client.as_ref() else {
            return;
        };

        let link_alive = client.is_connected().await.ok();

        if discards_client(link_alive) {
            self.client = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_command_result, discards_client};
    use crate::protocol::commands;
    use crate::state::DeviceState;

    #[test]
    fn discards_the_client_when_the_link_is_gone_or_unknown() {
        assert!(discards_client(Some(false)));
        assert!(discards_client(None));
    }

    #[test]
    fn keeps_the_client_when_the_link_is_alive() {
        assert!(!discards_client(Some(true)));
    }

    #[test]
    fn keeps_the_requested_value_when_the_command_is_not_echoed() {
        let mut state = DeviceState::default();

        let warnings = apply_command_result(&mut state, &commands::game_mode(true), &[]);

        assert!(warnings.is_empty());
        assert!(state.game_mode);
    }

    #[test]
    fn prefers_the_echo_when_the_firmware_answers() {
        let mut state = DeviceState::default();
        let echo = vec![0xFF, 0x03, 0x09, 0x01, 0x02];

        apply_command_result(&mut state, &commands::game_mode(true), &[echo]);

        assert!(!state.game_mode);
    }

    #[test]
    fn keeps_the_requested_value_when_only_invalid_notifications_arrive() {
        let mut state = DeviceState::default();

        let warnings = apply_command_result(&mut state, &commands::ldac(true), &[vec![0x00]]);

        assert_eq!(warnings.len(), 1);
        assert!(state.ldac);
    }
}
