use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SERVICE_UUID: &str = "0000a001-0000-1000-8000-00805f9b34fb";
pub const COMMAND_WRITE_UUID: &str = "00001001-0000-1000-8000-00805f9b34fb";
pub const NOTIFICATION_UUID: &str = "00001002-0000-1000-8000-00805f9b34fb";
pub const BATTERY_UUID: &str = "00000008-0000-1000-8000-00805f9b34fb";
pub const FIRMWARE_UUID: &str = "00000007-0000-1000-8000-00805f9b34fb";
pub const TOUCH_UUID: &str = "0000000d-0000-1000-8000-00805f9b34fb";

/// BLE UUIDs used by the headphones. These are persisted with the MAC address
/// so a future transport layer can resolve them without hard-coding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BleUuids {
    pub service: String,
    pub command_write: String,
    pub notification: String,
    pub battery: String,
    pub firmware: String,
    pub touch: String,
}

impl Default for BleUuids {
    fn default() -> Self {
        Self {
            service: SERVICE_UUID.to_string(),
            command_write: COMMAND_WRITE_UUID.to_string(),
            notification: NOTIFICATION_UUID.to_string(),
            battery: BATTERY_UUID.to_string(),
            firmware: FIRMWARE_UUID.to_string(),
            touch: TOUCH_UUID.to_string(),
        }
    }
}

/// Persistent configuration stored in `~/.melobudsnext/config.json`.
///
/// Deserialization also accepts the flat format written by the Python
/// implementation (`address`, `uuid_service`, `uuid_write`, `uuid_notify`), so
/// an existing installation keeps working without manual conversion.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RawConfig")]
pub struct Config {
    pub mac_address: Option<String>,
    pub uuids: BleUuids,
}

/// Intermediate representation covering both the current and the legacy
/// configuration layouts.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    mac_address: Option<String>,
    uuids: Option<BleUuids>,
    address: Option<String>,
    uuid_service: Option<String>,
    uuid_write: Option<String>,
    uuid_notify: Option<String>,
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        let RawConfig {
            mac_address,
            uuids,
            address,
            uuid_service,
            uuid_write,
            uuid_notify,
        } = raw;

        // When the current nested layout is present it wins as a whole; the
        // legacy flat keys are only consulted for old configuration files.
        let uuids = uuids.unwrap_or_else(|| {
            let mut uuids = BleUuids::default();
            if let Some(value) = uuid_service {
                uuids.service = value;
            }
            if let Some(value) = uuid_write {
                uuids.command_write = value;
            }
            if let Some(value) = uuid_notify {
                uuids.notification = value;
            }
            uuids
        });

        Self {
            mac_address: mac_address.or(address),
            uuids,
        }
    }
}

impl Config {
    /// Returns the default configuration path for the current user.
    ///
    /// Resolving the path fails when neither `HOME` nor `USERPROFILE` is set,
    /// instead of silently falling back to the current directory. Callers can
    /// still pass an explicit path with `--config`.
    pub fn default_path() -> io::Result<PathBuf> {
        default_path_from(home_directory())
    }

    /// Loads the default configuration, returning defaults when the file does
    /// not exist.
    pub fn load() -> io::Result<Self> {
        Self::load_from(Self::default_path()?)
    }

    pub fn load_from(path: impl AsRef<Path>) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to(Self::default_path()?)
    }

    /// Writes the configuration atomically.
    ///
    /// The JSON is written to a temporary file in the same directory and then
    /// renamed over the destination, so an interrupted write cannot leave a
    /// truncated configuration behind.
    pub fn save_to(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = temporary_path(path);
        fs::write(&temporary, contents)?;

        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }

        Ok(())
    }

    pub fn with_mac(mut self, mac: impl Into<String>) -> Self {
        self.mac_address = Some(mac.into());
        self
    }
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

fn default_path_from(home: Option<PathBuf>) -> io::Result<PathBuf> {
    let home = home.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cannot determine the home directory; set HOME or USERPROFILE, or pass --config PATH",
        )
    })?;

    Ok(home.join(".melobudsnext").join("config.json"))
}

/// Returns the sibling path used to stage an atomic write.
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("melobudsnext-test-{}-{unique}", std::process::id()))
    }

    #[test]
    fn default_path_uses_the_home_directory() {
        let path = default_path_from(Some(PathBuf::from("/tmp/home"))).unwrap();

        assert_eq!(
            path,
            PathBuf::from("/tmp/home")
                .join(".melobudsnext")
                .join("config.json")
        );
    }

    #[test]
    fn default_path_fails_without_a_home_directory() {
        let error = default_path_from(None).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.to_string().contains("--config"));
    }

    #[test]
    fn missing_file_returns_defaults() {
        let path = temporary_directory().join("missing.json");
        let config = Config::load_from(path).unwrap();
        assert_eq!(config, Config::default());
        assert_eq!(config.uuids.service, SERVICE_UUID);
    }

    #[test]
    fn saves_and_loads_configuration() {
        let directory = temporary_directory();
        let path = directory.join("config.json");
        let config = Config::default().with_mac("AA:BB:CC:DD:EE:FF");

        config.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded, config);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn saves_atomically_without_leaving_a_temporary_file() {
        let directory = temporary_directory();
        let path = directory.join("config.json");
        let config = Config::default().with_mac("AA:BB:CC:DD:EE:FF");

        config.save_to(&path).unwrap();

        assert!(path.exists());
        assert!(!temporary_path(&path).exists());
        assert_eq!(Config::load_from(&path).unwrap(), config);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn failed_save_preserves_the_previous_file() {
        let directory = temporary_directory();
        let path = directory.join("config.json");
        let original = Config::default().with_mac("AA:BB:CC:DD:EE:FF");
        original.save_to(&path).unwrap();

        // Occupy the staging path so the write fails before the rename.
        fs::create_dir_all(temporary_path(&path)).unwrap();

        let replacement = Config::default().with_mac("11:22:33:44:55:66");
        assert!(replacement.save_to(&path).is_err());
        assert_eq!(Config::load_from(&path).unwrap(), original);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn fills_missing_fields_with_defaults() {
        let directory = temporary_directory();
        let path = directory.join("config.json");
        fs::create_dir_all(&directory).unwrap();
        fs::write(&path, r#"{"mac_address":"11:22:33:44:55:66"}"#).unwrap();

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.mac_address.as_deref(), Some("11:22:33:44:55:66"));
        assert_eq!(config.uuids, BleUuids::default());

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn loads_the_legacy_python_layout() {
        let contents = r#"{
            "address": "11:22:33:44:55:66",
            "uuid_service": "0000a001-0000-1000-8000-00805f9b34fb",
            "uuid_write": "00001001-0000-1000-8000-00805f9b34fb",
            "uuid_notify": "00001002-0000-1000-8000-00805f9b34fb"
        }"#;

        let config: Config = serde_json::from_str(contents).unwrap();

        assert_eq!(config.mac_address.as_deref(), Some("11:22:33:44:55:66"));
        assert_eq!(config.uuids.service, SERVICE_UUID);
        assert_eq!(config.uuids.command_write, COMMAND_WRITE_UUID);
        assert_eq!(config.uuids.notification, NOTIFICATION_UUID);
        assert_eq!(config.uuids.battery, BATTERY_UUID);
    }

    #[test]
    fn prefers_the_current_layout_over_legacy_keys() {
        let contents = r#"{
            "mac_address": "AA:BB:CC:DD:EE:FF",
            "address": "11:22:33:44:55:66",
            "uuids": {"service": "11111111-0000-1000-8000-00805f9b34fb"},
            "uuid_service": "22222222-0000-1000-8000-00805f9b34fb"
        }"#;

        let config: Config = serde_json::from_str(contents).unwrap();

        assert_eq!(config.mac_address.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
        assert_eq!(config.uuids.service, "11111111-0000-1000-8000-00805f9b34fb");
    }
}
