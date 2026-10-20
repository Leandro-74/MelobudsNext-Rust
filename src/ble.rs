//! BLE transport built on top of `btleplug`.
//!
//! The transport keeps the protocol crate independent from a specific
//! Bluetooth backend: it connects to the headphones, resolves the
//! characteristics documented in the protocol, and exposes reads, writes and
//! notifications as ordinary async methods.

use std::collections::BTreeSet;
use std::pin::Pin;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter,
    ValueNotification, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::Stream;
use futures::StreamExt;
use tokio::time::Instant;
use uuid::Uuid;

use crate::config::BleUuids;
use crate::hex;
use crate::protocol::commands;
use crate::protocol::framing::Packet;
use crate::protocol::opcode::Opcode;
use crate::protocol::readings::{Battery, FirmwareVersion};
use crate::protocol::touch::TouchMapping;
use crate::DeviceState;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const SCAN_POLL_INTERVAL: Duration = Duration::from_millis(250);
const QUERY_INTERVAL: Duration = Duration::from_millis(50);
const NOTIFICATION_SETTLE: Duration = Duration::from_millis(150);

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
            }),
            Err(error) => {
                let _ = peripheral.disconnect().await;
                Err(error)
            }
        }
    }

    /// Reconnects after a link drop (for example when LDAC restarts the
    /// headphones). Services and characteristics are rediscovered.
    pub async fn reconnect(&mut self, attempts: u32, delay: Duration) -> Result<()> {
        let _ = self.peripheral.disconnect().await;
        let mut last_error = None;

        for attempt in 1..=attempts.max(1) {
            tokio::time::sleep(delay).await;

            match self.peripheral.connect_with_timeout(CONNECT_TIMEOUT).await {
                Ok(()) => match self.refresh().await {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        let _ = self.peripheral.disconnect().await;
                        last_error = Some(error.context(format!("reconnect attempt {attempt}")));
                    }
                },
                Err(error) => {
                    last_error = Some(
                        anyhow::Error::new(error).context(format!("reconnect attempt {attempt}")),
                    );
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("failed to reconnect")))
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
        Ok(())
    }

    pub fn address(&self) -> String {
        self.peripheral.address().to_string()
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
        self.write_raw(&packet.encode()).await
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

    /// Subscribes to command notifications and returns the notification stream.
    pub async fn subscribe_commands(&self) -> Result<NotificationStream> {
        self.peripheral
            .subscribe(&self.notification)
            .await
            .context("failed to subscribe to command notifications")?;
        let notifications = self
            .peripheral
            .notifications()
            .await
            .context("failed to open notification stream")?;
        tokio::time::sleep(NOTIFICATION_SETTLE).await;
        Ok(notifications)
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

        let mut notifications = self.subscribe_commands().await?;
        let deadline = Instant::now() + listen;
        let mut targets = QUERY_TARGETS.into_iter();
        let mut next_query = Instant::now();

        while Instant::now() < deadline {
            let wake = next_query.min(deadline);

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
                    if let Err(error) = report.state.apply_bytes(&notification.value) {
                        report.warnings.push(format!(
                            "notification {}: {error}",
                            hex::format(&notification.value)
                        ));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Sends the initial `0xFE` queries in the documented order.
    pub async fn query_all(&self) -> Result<()> {
        for target in QUERY_TARGETS {
            self.write_command(&commands::query(target)).await?;
            tokio::time::sleep(QUERY_INTERVAL).await;
        }

        Ok(())
    }

    /// Writes a packet and collects the notifications it triggers.
    pub async fn exchange(&self, packet: &Packet, listen: Duration) -> Result<Vec<Vec<u8>>> {
        let mut notifications = self.subscribe_commands().await?;
        self.write_command(packet).await?;
        Ok(self.receive_values(&mut notifications, listen).await)
    }

    /// Collects command notifications for a bounded amount of time.
    pub async fn receive(&self, stream: &mut NotificationStream, listen: Duration) -> Vec<Vec<u8>> {
        self.receive_values(stream, listen).await
    }

    async fn receive_values(
        &self,
        stream: &mut NotificationStream,
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
                    if notification.uuid == self.notification.uuid {
                        values.push(notification.value);
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }

        values
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.peripheral
            .disconnect()
            .await
            .context("failed to disconnect")
    }
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
    use btleplug::api::CentralState;

    match adapter
        .adapter_state()
        .await
        .context("failed to query adapter state")?
    {
        CentralState::PoweredOn => Ok(()),
        CentralState::PoweredOff => bail!("the Bluetooth adapter is powered off"),
        CentralState::Unknown => bail!("the Bluetooth adapter state is unknown"),
    }
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
