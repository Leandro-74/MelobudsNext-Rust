//! BLE transport built on top of `btleplug`.
//!
//! The transport keeps the protocol crate independent from a specific
//! Bluetooth backend: it connects to the headphones, resolves the
//! characteristics documented in the protocol, and exposes reads, writes and
//! notifications as ordinary async methods.

use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use btleplug::api::{
    Central, CentralState, CharPropFlags, Characteristic, Manager as _, Peripheral as _,
    ScanFilter, ValueNotification, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::Stream;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::time::Instant;
use uuid::Uuid;

use crate::config::BleUuids;
use crate::hex;
use crate::protocol::commands;
use crate::protocol::framing::{self, Packet};
use crate::protocol::opcode::Opcode;
use crate::protocol::readings::{Battery, FirmwareVersion};
use crate::protocol::touch::TouchMapping;
use crate::DeviceState;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const SCAN_POLL_INTERVAL: Duration = Duration::from_millis(250);
const QUERY_INTERVAL: Duration = Duration::from_millis(50);
const NOTIFICATION_SETTLE: Duration = Duration::from_millis(150);
/// Extra time kept after the last expected query response, so a trailing
/// notification from the same burst is not lost.
const SYNC_GRACE: Duration = Duration::from_millis(100);
/// Polls made while the adapter reports `Unknown` right after initialisation.
const ADAPTER_STATE_ATTEMPTS: u32 = 10;
/// Delay between adapter state polls.
const ADAPTER_STATE_DELAY: Duration = Duration::from_millis(100);

/// Opcodes queried after connecting, in the order used by the reference
/// implementation.
const QUERY_TARGETS: [Opcode; 8] = [
    Opcode::Anc,
    Opcode::GameMode,
    Opcode::SleepMode,
    Opcode::Rename,
    Opcode::ChannelBalance,
    Opcode::NotificationVolume,
    Opcode::Ldac,
    Opcode::WearDetection,
];

/// Notifications produced by the command characteristic.
pub type NotificationStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

/// Result of a state synchronisation.
///
/// Optional direct reads (battery, firmware, touch) no longer abort the whole
/// sync; their failures are reported as warnings instead.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub state: DeviceState,
    pub warnings: Vec<String>,
}

/// Tracks which query responses already arrived during a sync.
///
/// `sync` ends as soon as every target answered plus a short grace period,
/// instead of always waiting for the full listen window.
struct QueryTracker {
    pending: BTreeSet<u8>,
    grace: Duration,
    done_at: Option<Instant>,
}

impl QueryTracker {
    fn new(targets: impl IntoIterator<Item = Opcode>, grace: Duration) -> Self {
        Self {
            pending: targets.into_iter().map(Opcode::as_u8).collect(),
            grace,
            done_at: None,
        }
    }

    /// Records the opcodes carried by one notification.
    fn observe(&mut self, bytes: &[u8], now: Instant) {
        let Ok(packets) = framing::parse(bytes) else {
            return;
        };

        for packet in packets {
            self.pending.remove(&packet.opcode);
        }

        if self.pending.is_empty() && self.done_at.is_none() {
            self.done_at = Some(now + self.grace);
        }
    }

    /// Instant at which the wait may end, capped by the full window deadline.
    fn stop_at(&self, deadline: Instant) -> Instant {
        self.done_at.map_or(deadline, |done| done.min(deadline))
    }
}

/// A device found during a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub address: String,
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
    pub services: Vec<String>,
}

/// A resolved GATT characteristic, exposed for troubleshooting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GattCharacteristicInfo {
    pub uuid: Uuid,
    pub service_uuid: Uuid,
    pub properties: String,
}

/// Scans for nearby BLE devices and returns everything the adapter discovered.
pub async fn scan(timeout: Duration) -> Result<Vec<DiscoveredDevice>> {
    let adapter = first_adapter().await?;
    ensure_powered(&adapter).await?;

    adapter
        .start_scan(ScanFilter::default())
        .await
        .context("failed to start BLE scan")?;
    tokio::time::sleep(timeout).await;

    let peripherals = adapter.peripherals().await;
    let _ = adapter.stop_scan().await;
    let peripherals = peripherals.context("failed to list discovered devices")?;

    let mut devices = Vec::with_capacity(peripherals.len());
    for peripheral in peripherals {
        let properties = peripheral.properties().await.ok().flatten();
        let name = properties.as_ref().and_then(|properties| {
            properties
                .local_name
                .clone()
                .or_else(|| properties.advertisement_name.clone())
        });
        let rssi = properties.as_ref().and_then(|properties| properties.rssi);
        let services = properties
            .as_ref()
            .map(|properties| {
                properties
                    .services
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        devices.push(DiscoveredDevice {
            address: peripheral.address().to_string(),
            id: peripheral.id().to_string(),
            name,
            rssi,
            services,
        });
    }

    devices.sort_by(|left, right| left.address.cmp(&right.address));
    Ok(devices)
}

/// An open BLE session bound to the Melobuds command channel.
pub struct BleSession {
    peripheral: Peripheral,
    uuids: UuidSet,
    command: Characteristic,
    notification: Characteristic,
    battery: Option<Characteristic>,
    firmware: Option<Characteristic>,
    touch: Option<Characteristic>,
    /// Notification stream kept open for the whole session.
    ///
    /// `btleplug` requires the stream to be polled mutably, so it lives behind
    /// an async mutex shared by `sync` and `exchange`. The subscription is
    /// dropped and reopened only when the link is rebuilt.
    notifications: Mutex<Option<NotificationStream>>,
}

/// Characteristics resolved for a connected device.
struct Resolved {
    command: Characteristic,
    notification: Characteristic,
    battery: Option<Characteristic>,
    firmware: Option<Characteristic>,
    touch: Option<Characteristic>,
}

impl BleSession {
    /// Finds a device by address (or platform id), connects, discovers services
    /// and resolves every characteristic from the supplied UUID set.
    pub async fn connect(address: &str, uuids: &BleUuids, scan_timeout: Duration) -> Result<Self> {
        let adapter = first_adapter().await?;
        ensure_powered(&adapter).await?;

        let peripheral = find_peripheral(&adapter, address, scan_timeout).await?;
        Self::open(peripheral, UuidSet::from_config(uuids)?).await
    }

    async fn open(peripheral: Peripheral, uuids: UuidSet) -> Result<Self> {
        let resolved = async {
            if !peripheral
                .is_connected()
                .await
                .context("failed to query connection state")?
            {
                peripheral
                    .connect_with_timeout(CONNECT_TIMEOUT)
                    .await
                    .with_context(|| format!("failed to connect to {}", peripheral.address()))?;
            }

            peripheral
                .discover_services()
                .await
                .context("failed to discover GATT services")?;

            resolve(&peripheral, &uuids)
        }
        .await;

        match resolved {
            Ok(resolved) => Ok(Self {
                peripheral,
                uuids,
                command: resolved.command,
                notification: resolved.notification,
                battery: resolved.battery,
                firmware: resolved.firmware,
                touch: resolved.touch,
                notifications: Mutex::new(None),
            }),
            Err(error) => {
                let _ = peripheral.disconnect().await;
                Err(error)
            }
        }
    }

    /// Performs one reconnection attempt after a link drop (for example when
    /// LDAC restarts the headphones). Services and characteristics are
    /// rediscovered. The caller drives the retry loop so it can report
    /// progress between attempts.
    pub async fn reconnect_once(&mut self, delay: Duration) -> Result<()> {
        let _ = self.peripheral.disconnect().await;
        tokio::time::sleep(delay).await;

        self.peripheral
            .connect_with_timeout(CONNECT_TIMEOUT)
            .await
            .context("failed to reconnect")?;
        self.refresh().await
    }

    async fn refresh(&mut self) -> Result<()> {
        self.peripheral
            .discover_services()
            .await
            .context("failed to rediscover GATT services")?;
        let resolved = resolve(&self.peripheral, &self.uuids)?;
        self.command = resolved.command;
        self.notification = resolved.notification;
        self.battery = resolved.battery;
        self.firmware = resolved.firmware;
        self.touch = resolved.touch;
        // The old subscription belongs to the previous link; reopen it lazily
        // on the next notification-consuming operation.
        *self.notifications.get_mut() = None;
        Ok(())
    }

    pub fn address(&self) -> String {
        self.peripheral.address().to_string()
    }

    /// Reports whether the underlying BLE link is still up.
    ///
    /// Actions call this after writing so a headset that was switched off mid
    /// session is reported as an error instead of leaving the caller with a
    /// stale "connected" state.
    pub async fn is_connected(&self) -> Result<bool> {
        self.peripheral
            .is_connected()
            .await
            .context("failed to query connection state")
    }

    /// Fails when the peripheral no longer reports an active link.
    async fn ensure_link(&self) -> Result<()> {
        if self.is_connected().await? {
            Ok(())
        } else {
            Err(anyhow!("BLE link lost; connect again"))
        }
    }

    pub fn notification_uuid(&self) -> Uuid {
        self.notification.uuid
    }

    pub fn services(&self) -> Vec<Uuid> {
        self.peripheral
            .services()
            .iter()
            .map(|service| service.uuid)
            .collect()
    }

    pub fn characteristics(&self) -> Vec<GattCharacteristicInfo> {
        self.peripheral
            .characteristics()
            .into_iter()
            .map(|characteristic| GattCharacteristicInfo {
                uuid: characteristic.uuid,
                service_uuid: characteristic.service_uuid,
                properties: format_properties(characteristic.properties),
            })
            .collect()
    }

    pub async fn read_battery(&self) -> Result<Battery> {
        let characteristic = self
            .battery
            .as_ref()
            .ok_or_else(|| anyhow!("battery characteristic not found"))?;
        let bytes = self
            .peripheral
            .read(characteristic)
            .await
            .context("failed to read battery characteristic")?;
        Battery::parse(&bytes).context("invalid battery payload")
    }

    pub async fn read_firmware(&self) -> Result<FirmwareVersion> {
        let characteristic = self
            .firmware
            .as_ref()
            .ok_or_else(|| anyhow!("firmware characteristic not found"))?;
        let bytes = self
            .peripheral
            .read(characteristic)
            .await
            .context("failed to read firmware characteristic")?;
        FirmwareVersion::parse(&bytes).context("invalid firmware payload")
    }

    pub async fn read_touch(&self) -> Result<TouchMapping> {
        let characteristic = self
            .touch
            .as_ref()
            .ok_or_else(|| anyhow!("touch characteristic not found"))?;
        let bytes = self
            .peripheral
            .read(characteristic)
            .await
            .context("failed to read touch mapping")?;
        TouchMapping::parse(&bytes).context("invalid touch mapping payload")
    }

    /// Writes a raw command-channel payload (normally an encoded packet).
    pub async fn write_raw(&self, bytes: &[u8]) -> Result<()> {
        self.write_raw_with(bytes, false).await
    }

    /// Writes raw bytes choosing between write-with and write-without response.
    pub async fn write_raw_with(&self, bytes: &[u8], with_response: bool) -> Result<()> {
        let write_type = if with_response {
            WriteType::WithResponse
        } else {
            WriteType::WithoutResponse
        };

        self.peripheral
            .write(&self.command, bytes, write_type)
            .await
            .context("failed to write command characteristic")
    }

    /// Writes an encoded protocol packet.
    pub async fn write_command(&self, packet: &Packet) -> Result<()> {
        self.write_raw(&packet.try_encode()?).await
    }

    /// Writes a complete 20-byte touch mapping.
    pub async fn write_touch(&self, mapping: &TouchMapping) -> Result<()> {
        let characteristic = self
            .touch
            .as_ref()
            .ok_or_else(|| anyhow!("touch characteristic not found"))?;
        self.peripheral
            .write(
                characteristic,
                &mapping.encode(),
                WriteType::WithoutResponse,
            )
            .await
            .context("failed to write touch mapping")
    }

    /// Returns the session-wide notification stream, subscribing on first use.
    ///
    /// The subscription is kept open until [`BleSession::refresh`] rebuilds the
    /// link, so commands and syncs no longer pay the settle delay every time.
    async fn notification_stream(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<NotificationStream>>> {
        let mut slot = self.notifications.lock().await;

        if slot.is_none() {
            self.peripheral
                .subscribe(&self.notification)
                .await
                .context("failed to subscribe to command notifications")?;
            let stream = self
                .peripheral
                .notifications()
                .await
                .context("failed to open notification stream")?;
            tokio::time::sleep(NOTIFICATION_SETTLE).await;
            *slot = Some(stream);
        }

        Ok(slot)
    }

    /// Waits for the next command notification on the persistent stream.
    ///
    /// Returns `None` when the backend closes the stream.
    pub async fn next_notification(&self) -> Result<Option<ValueNotification>> {
        let mut slot = self.notification_stream().await?;
        let stream = slot
            .as_mut()
            .ok_or_else(|| anyhow!("notification stream is not open"))?;
        Ok(stream.next().await)
    }

    /// Waits up to `wait` for one command notification and returns its value.
    ///
    /// Unlike [`BleSession::receive`], this is meant for an idle session: it
    /// returns as soon as one notification arrives, letting the GUI mirror
    /// changes made on the headphones without polling the full state. A stream
    /// closed by the backend is reported as an error so the caller can discard
    /// the dead link; a timeout returns `Ok(None)`.
    pub async fn wait_notification(&self, wait: Duration) -> Result<Option<Vec<u8>>> {
        let mut slot = self.notification_stream().await?;
        let stream = slot
            .as_mut()
            .ok_or_else(|| anyhow!("notification stream is not open"))?;
        next_value(stream, self.notification.uuid, wait).await
    }

    /// Collects command notifications for `listen` using the session stream.
    pub async fn receive(&self, listen: Duration) -> Result<Vec<Vec<u8>>> {
        let mut slot = self.notification_stream().await?;
        let stream = slot
            .as_mut()
            .ok_or_else(|| anyhow!("notification stream is not open"))?;
        Ok(receive_values(stream, self.notification.uuid, listen).await)
    }

    /// Reads the direct characteristics, queries the remaining state and waits
    /// for the responses.
    ///
    /// Optional reads are best-effort: a missing or unreadable characteristic
    /// becomes a warning in [`SyncReport`] instead of aborting the sync. State
    /// queries are interleaved with notification polling so responses are not
    /// lost while the next query is being written.
    pub async fn sync(&self, listen: Duration) -> Result<SyncReport> {
        let mut report = SyncReport::default();

        self.ensure_link().await?;

        if self.battery.is_some() {
            match self.read_battery().await {
                Ok(battery) => report.state.set_battery(battery),
                Err(error) => report.warnings.push(format!("battery: {error:#}")),
            }
        }
        if self.firmware.is_some() {
            match self.read_firmware().await {
                Ok(firmware) => report.state.set_firmware(firmware),
                Err(error) => report.warnings.push(format!("firmware: {error:#}")),
            }
        }
        if self.touch.is_some() {
            match self.read_touch().await {
                Ok(touch) => report.state.set_touch(touch),
                Err(error) => report.warnings.push(format!("touch: {error:#}")),
            }
        }

        let mut slot = self.notification_stream().await?;
        let notifications = slot
            .as_mut()
            .ok_or_else(|| anyhow!("notification stream is not open"))?;
        let deadline = Instant::now() + listen;
        let mut tracker = QueryTracker::new(QUERY_TARGETS, SYNC_GRACE);
        let mut targets = QUERY_TARGETS.into_iter();
        let mut next_query = Instant::now();

        while Instant::now() < deadline {
            let stop_at = tracker.stop_at(deadline);
            if Instant::now() >= stop_at {
                break;
            }

            let wake = next_query.min(stop_at);

            tokio::select! {
                _ = tokio::time::sleep_until(wake) => {
                    if next_query > Instant::now() {
                        continue;
                    }

                    match targets.next() {
                        Some(target) => {
                            if let Err(error) = self.write_command(&commands::query(target)).await {
                                report.warnings.push(format!(
                                    "query 0x{:02X}: {error:#}",
                                    target.as_u8()
                                ));
                                break;
                            }
                            next_query = Instant::now() + QUERY_INTERVAL;
                        }
                        None => next_query = deadline,
                    }
                }
                item = notifications.next() => {
                    let Some(notification) = item else {
                        break;
                    };
                    if notification.uuid != self.notification.uuid {
                        continue;
                    }
                    tracker.observe(&notification.value, Instant::now());
                    if let Err(error) = report.state.apply_bytes(&notification.value) {
                        report.warnings.push(format!(
                            "notification {}: {error}",
                            hex::format(&notification.value)
                        ));
                    }
                }
            }
        }

        self.ensure_link().await?;
        Ok(report)
    }

    /// Writes a packet and collects the notifications it triggers.
    pub async fn exchange(&self, packet: &Packet, listen: Duration) -> Result<Vec<Vec<u8>>> {
        let mut slot = self.notification_stream().await?;
        let notifications = slot
            .as_mut()
            .ok_or_else(|| anyhow!("notification stream is not open"))?;
        self.write_command(packet).await?;
        let values = receive_values(notifications, self.notification.uuid, listen).await;
        drop(slot);
        self.ensure_link().await?;
        Ok(values)
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.peripheral
            .disconnect()
            .await
            .context("failed to disconnect")
    }
}

/// Waits for one notification from `notification_uuid`, up to `wait`.
///
/// Notifications from other characteristics are skipped. A closed stream is an
/// error; reaching the deadline without a matching notification returns
/// `Ok(None)`.
async fn next_value(
    stream: &mut NotificationStream,
    notification_uuid: Uuid,
    wait: Duration,
) -> Result<Option<Vec<u8>>> {
    let deadline = Instant::now() + wait;

    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }

        match tokio::time::timeout(deadline - now, stream.next()).await {
            Ok(Some(notification)) => {
                if notification.uuid == notification_uuid {
                    return Ok(Some(notification.value));
                }
            }
            Ok(None) => return Err(anyhow!("notification stream closed")),
            Err(_) => return Ok(None),
        }
    }
}

/// Collects command notifications for a bounded amount of time.
///
/// Takes the open stream explicitly so both [`BleSession::sync`] and
/// [`BleSession::exchange`] reuse the session subscription.
async fn receive_values(
    stream: &mut NotificationStream,
    notification_uuid: Uuid,
    listen: Duration,
) -> Vec<Vec<u8>> {
    let deadline = Instant::now() + listen;
    let mut values = Vec::new();

    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        match tokio::time::timeout(deadline - now, stream.next()).await {
            Ok(Some(notification)) => {
                if notification.uuid == notification_uuid {
                    values.push(notification.value);
                }
            }
            Ok(None) | Err(_) => break,
        }
    }

    values
}

fn resolve(peripheral: &Peripheral, uuids: &UuidSet) -> Result<Resolved> {
    // The device exposes the command/notification UUIDs in more than one
    // service. Resolve them inside the configured service, like the reference
    // implementation does, otherwise the wrong characteristic may be selected.
    let services = peripheral.services();
    let service = services
        .iter()
        .find(|service| service.uuid == uuids.service)
        .ok_or_else(|| {
            anyhow!(
                "service {} not found on device {}",
                uuids.service,
                peripheral.address()
            )
        })?;

    let characteristics = &service.characteristics;

    Ok(Resolved {
        command: find_characteristic(characteristics, uuids.command_write, "command write")?,
        notification: find_characteristic(characteristics, uuids.notification, "notification")?,
        battery: find_optional_characteristic(characteristics, uuids.battery),
        firmware: find_optional_characteristic(characteristics, uuids.firmware),
        touch: find_optional_characteristic(characteristics, uuids.touch),
    })
}

async fn first_adapter() -> Result<Adapter> {
    let manager = Manager::new()
        .await
        .context("failed to initialize the Bluetooth manager")?;
    let adapters = manager
        .adapters()
        .await
        .context("failed to list Bluetooth adapters")?;
    adapters
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no Bluetooth adapter found"))
}

async fn ensure_powered(adapter: &Adapter) -> Result<()> {
    poll_adapter_state(
        || async {
            adapter
                .adapter_state()
                .await
                .context("failed to query adapter state")
        },
        ADAPTER_STATE_ATTEMPTS,
        ADAPTER_STATE_DELAY,
    )
    .await
}

/// Polls the adapter until it is powered on.
///
/// Some backends report [`CentralState::Unknown`] for a short while after the
/// adapter is initialised. A transient `Unknown` is retried; `PoweredOff` is
/// still an immediate error, and a state that never settles gets a distinct
/// message so the caller knows the difference.
async fn poll_adapter_state<F, Fut>(mut query: F, attempts: u32, delay: Duration) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<CentralState>>,
{
    let attempts = attempts.max(1);

    for attempt in 1..=attempts {
        match query().await? {
            CentralState::PoweredOn => return Ok(()),
            CentralState::PoweredOff => bail!("the Bluetooth adapter is powered off"),
            CentralState::Unknown => {
                if attempt == attempts {
                    bail!("the Bluetooth adapter state remained unknown after {attempts} attempts");
                }
                tokio::time::sleep(delay).await;
            }
        }
    }

    unreachable!("the adapter state loop returns or fails on every attempt")
}

async fn find_peripheral(adapter: &Adapter, target: &str, timeout: Duration) -> Result<Peripheral> {
    if let Some(peripheral) = find_match(adapter, target).await? {
        return Ok(peripheral);
    }

    adapter
        .start_scan(ScanFilter::default())
        .await
        .context("failed to start BLE scan")?;

    let deadline = Instant::now() + timeout;
    let mut found = None;

    loop {
        match find_match(adapter, target).await {
            Ok(Some(peripheral)) => {
                found = Some(peripheral);
                break;
            }
            Ok(None) => {}
            Err(error) => {
                let _ = adapter.stop_scan().await;
                return Err(error);
            }
        }

        if Instant::now() >= deadline {
            break;
        }

        let remaining = deadline - Instant::now();
        tokio::time::sleep(SCAN_POLL_INTERVAL.min(remaining)).await;
    }

    let _ = adapter.stop_scan().await;
    found.ok_or_else(|| {
        anyhow!(
            "device {target:?} not found within {} seconds",
            timeout.as_secs()
        )
    })
}

async fn find_match(adapter: &Adapter, target: &str) -> Result<Option<Peripheral>> {
    let normalized_target = normalize_target(target);
    let peripherals = adapter
        .peripherals()
        .await
        .context("failed to list discovered devices")?;

    for peripheral in peripherals {
        let address = normalize_target(&peripheral.address().to_string());
        let id = normalize_target(&peripheral.id().to_string());

        if address == normalized_target || id == normalized_target {
            return Ok(Some(peripheral));
        }

        if normalized_target.len() >= 3 {
            let matches_name = peripheral
                .properties()
                .await
                .ok()
                .flatten()
                .and_then(|properties| properties.local_name.or(properties.advertisement_name))
                .is_some_and(|name| normalize_target(&name).contains(&normalized_target));

            if matches_name {
                return Ok(Some(peripheral));
            }
        }
    }

    Ok(None)
}

fn normalize_target(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn format_properties(properties: CharPropFlags) -> String {
    const NAMES: [(CharPropFlags, &str); 8] = [
        (CharPropFlags::BROADCAST, "broadcast"),
        (CharPropFlags::READ, "read"),
        (
            CharPropFlags::WRITE_WITHOUT_RESPONSE,
            "write-without-response",
        ),
        (CharPropFlags::WRITE, "write"),
        (CharPropFlags::NOTIFY, "notify"),
        (CharPropFlags::INDICATE, "indicate"),
        (CharPropFlags::AUTHENTICATED_SIGNED_WRITES, "signed-write"),
        (CharPropFlags::EXTENDED_PROPERTIES, "extended"),
    ];

    let values = NAMES
        .iter()
        .filter(|(flag, _)| properties.contains(*flag))
        .map(|(_, name)| *name)
        .collect::<Vec<_>>();

    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn find_characteristic(
    characteristics: &BTreeSet<Characteristic>,
    uuid: Uuid,
    label: &str,
) -> Result<Characteristic> {
    find_optional_characteristic(characteristics, uuid)
        .ok_or_else(|| anyhow!("{label} characteristic {uuid} not found"))
}

fn find_optional_characteristic(
    characteristics: &BTreeSet<Characteristic>,
    uuid: Uuid,
) -> Option<Characteristic> {
    characteristics
        .iter()
        .find(|characteristic| characteristic.uuid == uuid)
        .cloned()
}

struct UuidSet {
    service: Uuid,
    command_write: Uuid,
    notification: Uuid,
    battery: Uuid,
    firmware: Uuid,
    touch: Uuid,
}

impl UuidSet {
    fn from_config(config: &BleUuids) -> Result<Self> {
        Ok(Self {
            service: parse_uuid(&config.service, "service")?,
            command_write: parse_uuid(&config.command_write, "command write")?,
            notification: parse_uuid(&config.notification, "notification")?,
            battery: parse_uuid(&config.battery, "battery")?,
            firmware: parse_uuid(&config.firmware, "firmware")?,
            touch: parse_uuid(&config.touch, "touch")?,
        })
    }
}

fn parse_uuid(value: &str, label: &str) -> Result<Uuid> {
    Uuid::parse_str(value).with_context(|| format!("invalid {label} UUID {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::channel::mpsc;

    const TEST_UUID: &str = "00001002-0000-1000-8000-00805f9b34fb";

    fn notification(uuid: Uuid, value: Vec<u8>) -> ValueNotification {
        ValueNotification {
            uuid,
            service_uuid: Uuid::nil(),
            value,
        }
    }

    #[tokio::test]
    async fn reuses_the_same_notification_stream_across_receives() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (sender, receiver) = mpsc::unbounded();
        let mut stream: NotificationStream = Box::pin(receiver);

        sender
            .unbounded_send(notification(uuid, vec![0xFF, 0x03, 0x09, 0x01, 0x01]))
            .unwrap();
        let first = receive_values(&mut stream, uuid, Duration::from_millis(20)).await;

        sender
            .unbounded_send(notification(uuid, vec![0xFF, 0x03, 0x10, 0x01, 0x01]))
            .unwrap();
        let second = receive_values(&mut stream, uuid, Duration::from_millis(20)).await;

        assert_eq!(first, vec![vec![0xFF, 0x03, 0x09, 0x01, 0x01]]);
        assert_eq!(second, vec![vec![0xFF, 0x03, 0x10, 0x01, 0x01]]);
    }

    #[tokio::test]
    async fn returns_a_live_notification_as_soon_as_it_arrives() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (sender, receiver) = mpsc::unbounded();
        let mut stream: NotificationStream = Box::pin(receiver);

        sender
            .unbounded_send(notification(
                uuid,
                vec![0xFF, 0x05, 0x17, 0x03, 0x01, 0x02, 0x02],
            ))
            .unwrap();

        let value = next_value(&mut stream, uuid, Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(value, Some(vec![0xFF, 0x05, 0x17, 0x03, 0x01, 0x02, 0x02]));
    }

    #[tokio::test]
    async fn live_notification_wait_times_out_without_values() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (_sender, receiver) = mpsc::unbounded();
        let mut stream: NotificationStream = Box::pin(receiver);

        let value = next_value(&mut stream, uuid, Duration::from_millis(20))
            .await
            .unwrap();

        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn live_notification_wait_skips_other_characteristics() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (sender, receiver) = mpsc::unbounded();
        let mut stream: NotificationStream = Box::pin(receiver);

        sender
            .unbounded_send(notification(Uuid::nil(), vec![0x01]))
            .unwrap();
        sender
            .unbounded_send(notification(uuid, vec![0xFF, 0x03, 0x09, 0x01, 0x01]))
            .unwrap();

        let value = next_value(&mut stream, uuid, Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(value, Some(vec![0xFF, 0x03, 0x09, 0x01, 0x01]));
    }

    #[tokio::test]
    async fn live_notification_wait_reports_a_closed_stream() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (sender, receiver) = mpsc::unbounded::<ValueNotification>();
        drop(sender);
        let mut stream: NotificationStream = Box::pin(receiver);

        let error = next_value(&mut stream, uuid, Duration::from_secs(1))
            .await
            .unwrap_err();

        assert!(error.to_string().contains("notification stream closed"));
    }

    #[tokio::test]
    async fn ignores_notifications_from_other_characteristics() {
        let uuid = Uuid::parse_str(TEST_UUID).unwrap();
        let (sender, receiver) = mpsc::unbounded();
        let mut stream: NotificationStream = Box::pin(receiver);

        sender
            .unbounded_send(notification(Uuid::nil(), vec![0x01]))
            .unwrap();
        sender
            .unbounded_send(notification(uuid, vec![0xFF, 0x03, 0x09, 0x01, 0x02]))
            .unwrap();

        let values = receive_values(&mut stream, uuid, Duration::from_millis(20)).await;

        assert_eq!(values, vec![vec![0xFF, 0x03, 0x09, 0x01, 0x02]]);
    }

    /// Builds one composite notification carrying a block for each target.
    fn query_responses(targets: &[Opcode]) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0x00];
        for target in targets {
            bytes.extend([target.as_u8(), 0x01, 0x00]);
        }
        bytes
    }

    #[test]
    fn sync_stops_after_the_last_response_plus_grace() {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(1500);
        let mut tracker = QueryTracker::new(QUERY_TARGETS, SYNC_GRACE);

        tracker.observe(
            &query_responses(&QUERY_TARGETS),
            now + Duration::from_millis(400),
        );

        assert_eq!(
            tracker.stop_at(deadline),
            now + Duration::from_millis(400) + SYNC_GRACE
        );
        assert!(tracker.stop_at(deadline) < deadline);
    }

    #[test]
    fn sync_waits_for_the_full_window_when_a_response_is_missing() {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(1500);
        let mut tracker = QueryTracker::new(QUERY_TARGETS, SYNC_GRACE);

        // Every target except the last one answers.
        tracker.observe(
            &query_responses(&QUERY_TARGETS[..QUERY_TARGETS.len() - 1]),
            now + Duration::from_millis(400),
        );

        assert_eq!(tracker.stop_at(deadline), deadline);
    }

    #[test]
    fn sync_starts_the_grace_period_only_after_all_targets_answer() {
        let now = Instant::now();
        let deadline = now + Duration::from_millis(1500);
        let mut tracker = QueryTracker::new(QUERY_TARGETS, SYNC_GRACE);

        tracker.observe(&query_responses(&QUERY_TARGETS[..1]), now);
        assert_eq!(tracker.stop_at(deadline), deadline);

        tracker.observe(
            &query_responses(&QUERY_TARGETS[1..]),
            now + Duration::from_millis(350),
        );

        assert_eq!(
            tracker.stop_at(deadline),
            now + Duration::from_millis(350) + SYNC_GRACE
        );
    }

    #[tokio::test]
    async fn retries_while_the_adapter_state_is_unknown() {
        use std::collections::VecDeque;
        use std::sync::{Arc, Mutex as StdMutex};

        let states = Arc::new(StdMutex::new(VecDeque::from([
            CentralState::Unknown,
            CentralState::Unknown,
            CentralState::PoweredOn,
        ])));
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let result = poll_adapter_state(
            || {
                let states = states.clone();
                let calls = calls.clone();
                async move {
                    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(states.lock().unwrap().pop_front().expect("scripted state"))
                }
            },
            5,
            Duration::ZERO,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn powered_off_adapter_fails_without_retrying() {
        use std::collections::VecDeque;
        use std::sync::{Arc, Mutex as StdMutex};

        let states = Arc::new(StdMutex::new(VecDeque::from([
            CentralState::PoweredOff,
            CentralState::PoweredOn,
        ])));

        let result = poll_adapter_state(
            || {
                let states = states.clone();
                async move { Ok(states.lock().unwrap().pop_front().expect("scripted state")) }
            },
            5,
            Duration::ZERO,
        )
        .await;

        let error = result.unwrap_err();
        assert!(error.to_string().contains("powered off"));
        assert_eq!(states.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn unknown_adapter_state_reports_a_distinct_error() {
        let result =
            poll_adapter_state(|| async { Ok(CentralState::Unknown) }, 3, Duration::ZERO).await;

        let error = result.unwrap_err();
        assert!(error.to_string().contains("remained unknown"));
    }

    #[test]
    fn normalizes_addresses_for_matching() {
        assert_eq!(normalize_target("AA:BB:CC:DD:EE:FF"), "AABBCCDDEEFF");
        assert_eq!(normalize_target("aa-bb-cc-dd-ee-ff"), "AABBCCDDEEFF");
        assert_eq!(normalize_target("aabbccddeeff"), "AABBCCDDEEFF");
    }

    #[test]
    fn formats_characteristic_properties() {
        let properties = CharPropFlags::READ | CharPropFlags::NOTIFY;
        assert_eq!(format_properties(properties), "read, notify");
        assert_eq!(format_properties(CharPropFlags::empty()), "none");
    }

    #[test]
    fn parses_default_uuids() {
        let uuids = UuidSet::from_config(&BleUuids::default()).unwrap();
        assert_eq!(uuids.service.to_string(), crate::config::SERVICE_UUID);
        assert_eq!(uuids.touch.to_string(), crate::config::TOUCH_UUID);
    }

    #[test]
    fn rejects_invalid_uuids() {
        let config = BleUuids {
            touch: "not-a-uuid".to_string(),
            ..BleUuids::default()
        };
        assert!(UuidSet::from_config(&config).is_err());
    }
}
