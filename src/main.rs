use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use melobudsnext::config::Config;
use melobudsnext::hex;
use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};
use melobudsnext::protocol::commands::{self, NotificationVolume, WearDetection};
use melobudsnext::protocol::framing::{self, Packet};
use melobudsnext::protocol::opcode::Opcode;
use melobudsnext::protocol::readings::{Battery, FirmwareVersion};
use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};
use melobudsnext::DeviceState;

#[derive(Debug, Parser)]
#[command(
    name = "melobudsnext",
    version,
    about = "QCY Melobuds Pro BLE protocol testing CLI"
)]
struct Cli {
    /// Path to the configuration file (defaults to ~/.melobudsnext/config.json)
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build a command packet and print its hexadecimal bytes.
    Build {
        #[command(subcommand)]
        command: BuildCommand,
    },

    /// Decode a notification or response and print the resulting state.
    Decode {
        /// Hexadecimal bytes (spaces, 0x prefixes and other separators are accepted).
        #[arg(required = true, num_args = 1.., value_name = "HEX")]
        data: Vec<String>,

        /// Print the decoded state as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Encode or decode the 20-byte touch mapping block.
    Touch {
        #[command(subcommand)]
        command: TouchCommand,
    },

    /// Decode the three-byte battery characteristic.
    Battery {
        #[arg(required = true, num_args = 1.., value_name = "HEX")]
        data: Vec<String>,
    },

    /// Decode the three-byte firmware characteristic.
    Firmware {
        #[arg(required = true, num_args = 1.., value_name = "HEX")]
        data: Vec<String>,
    },

    /// Inspect or update the persistent configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BuildCommand {
    /// Game Mode (opcode 0x09).
    Game {
        #[arg(value_enum)]
        state: OnOff,
    },

    /// Sleep Mode (opcode 0x10).
    Sleep {
        #[arg(value_enum)]
        state: OnOff,
    },

    /// Channel balance (opcode 0x16).
    Balance {
        /// Balance from 0 (left) to 100 (right); 50 is centered.
        value: u8,
    },

    /// ANC or Transparency (opcode 0x17).
    Anc(AncArgs),

    /// Rename the device (opcode 0x18, maximum 32 UTF-8 bytes).
    Rename { name: String },

    /// Notification volume (opcode 0x1D).
    NotificationVolume {
        #[arg(value_enum)]
        level: VolumeArg,
    },

    /// Wear detection (opcode 0x2C).
    WearDetection(WearArgs),

    /// LDAC (opcode 0x23).
    Ldac {
        #[arg(value_enum)]
        state: OnOff,
    },

    /// Query the current state of an opcode (opcode 0xFE).
    Query {
        /// Opcode name (for example `anc`) or hexadecimal value (for example `0x17`).
        opcode: String,
    },
}

#[derive(Debug, Args)]
struct AncArgs {
    /// ANC mode to encode.
    #[arg(value_enum)]
    mode: AncModeArg,

    /// ANC scene; required when mode is `anc`.
    #[arg(long, value_enum)]
    scene: Option<SceneArg>,

    /// ANC level (1..=3); required for indoor, daily and noise scenes.
    #[arg(long)]
    level: Option<u8>,

    /// Transparency intensity (1..=6).
    #[arg(long)]
    intensity: Option<u8>,

    /// Enable vocal enhancement (transparency mode only).
    #[arg(long)]
    voice: bool,
}

#[derive(Debug, Args)]
struct WearArgs {
    #[arg(value_enum)]
    state: OnOff,

    /// Warning music index (defaults to 1).
    #[arg(long, default_value_t = 1)]
    music_index: u8,

    /// Keep ANC enabled when the earbuds are removed.
    #[arg(long)]
    keep_anc: bool,

    /// Tone value (defaults to 0).
    #[arg(long, default_value_t = 0)]
    tone: u8,
}

#[derive(Debug, Subcommand)]
enum TouchCommand {
    /// Build a 20-byte touch mapping block.
    Encode {
        /// Binding in the form KEY=FUNCTION, for example `left-single=play-pause`.
        #[arg(long = "bind", value_name = "KEY=FUNCTION")]
        bindings: Vec<String>,
    },

    /// Decode a 20-byte touch mapping block.
    Decode {
        #[arg(required = true, num_args = 1.., value_name = "HEX")]
        data: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Print the configuration file path.
    Path,

    /// Print the current configuration as JSON.
    Show,

    /// Create a configuration file with the default UUIDs.
    Init {
        /// Optional device MAC address.
        #[arg(long)]
        mac: Option<String>,
    },

    /// Set the device MAC address.
    SetMac {
        /// Device MAC address, for example AA:BB:CC:DD:EE:FF.
        mac: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OnOff {
    On,
    Off,
}

impl OnOff {
    fn enabled(self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum VolumeArg {
    Low,
    Medium,
    High,
    Max,
}

impl From<VolumeArg> for NotificationVolume {
    fn from(value: VolumeArg) -> Self {
        match value {
            VolumeArg::Low => Self::Low,
            VolumeArg::Medium => Self::Medium,
            VolumeArg::High => Self::High,
            VolumeArg::Max => Self::Max,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AncModeArg {
    Off,
    Anc,
    Transparency,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SceneArg {
    Indoor,
    Daily,
    Noise,
    Wind,
    Adaptive,
}

impl From<SceneArg> for AncScene {
    fn from(value: SceneArg) -> Self {
        match value {
            SceneArg::Indoor => Self::Indoor,
            SceneArg::Daily => Self::Daily,
            SceneArg::Noise => Self::Noise,
            SceneArg::Wind => Self::Wind,
            SceneArg::Adaptive => Self::Adaptive,
        }
    }
}

fn main() -> Result<()> {
    Cli::parse().run()
}

impl Cli {
    fn run(self) -> Result<()> {
        let config_path = self.config.clone().unwrap_or_else(Config::default_path);

        match self.command {
            Command::Build { command } => print_packet(build_packet(command)?),
            Command::Decode { data, json } => decode_notification(&join_hex(&data)?, json)?,
            Command::Touch { command } => run_touch(command)?,
            Command::Battery { data } => decode_battery(&join_hex(&data)?)?,
            Command::Firmware { data } => decode_firmware(&join_hex(&data)?)?,
            Command::Config { command } => run_config(command, &config_path)?,
        }

        Ok(())
    }
}

fn build_packet(command: BuildCommand) -> Result<Packet> {
    let packet = match command {
        BuildCommand::Game { state } => commands::game_mode(state.enabled()),
        BuildCommand::Sleep { state } => commands::sleep_mode(state.enabled()),
        BuildCommand::Balance { value } => commands::channel_balance(value),
        BuildCommand::Anc(args) => commands::anc(build_anc_mode(args)?),
        BuildCommand::Rename { name } => commands::rename(&name)?,
        BuildCommand::NotificationVolume { level } => commands::notification_volume(level.into()),
        BuildCommand::WearDetection(args) => commands::wear_detection(WearDetection {
            enabled: args.state.enabled(),
            music_index: args.music_index,
            keep_anc: args.keep_anc,
            tone: args.tone,
        }),
        BuildCommand::Ldac { state } => commands::ldac(state.enabled()),
        BuildCommand::Query { opcode } => {
            let opcode = opcode
                .parse::<Opcode>()
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("invalid query opcode {opcode:?}"))?;
            commands::query(opcode)
        }
    };

    Ok(packet)
}

fn build_anc_mode(args: AncArgs) -> Result<AncMode> {
    match args.mode {
        AncModeArg::Off => {
            if args.scene.is_some()
                || args.level.is_some()
                || args.intensity.is_some()
                || args.voice
            {
                bail!("--scene, --level, --intensity and --voice are not valid in off mode");
            }
            Ok(AncMode::Off)
        }
        AncModeArg::Anc => {
            if args.intensity.is_some() || args.voice {
                bail!("--intensity and --voice are only valid in transparency mode");
            }
            let scene = args
                .scene
                .ok_or_else(|| anyhow::anyhow!("--scene is required in anc mode"))?;
            Ok(AncMode::anc(scene.into(), args.level)?)
        }
        AncModeArg::Transparency => {
            if args.scene.is_some() || args.level.is_some() {
                bail!("--scene and --level are only valid in anc mode");
            }

            let mode = match (args.voice, args.intensity) {
                (true, None) => TransparencyMode::VoiceEnhancement,
                (false, Some(intensity)) => TransparencyMode::Intensity(intensity),
                (false, None) => {
                    bail!("transparency mode requires --voice or --intensity 1..=6")
                }
                (true, Some(_)) => {
                    bail!("--voice and --intensity cannot be used together")
                }
            };

            Ok(AncMode::transparency(mode)?)
        }
    }
}

fn print_packet(packet: Packet) {
    println!("{}", hex::format(&packet.encode()));
}

fn decode_notification(bytes: &[u8], json: bool) -> Result<()> {
    let packets = framing::parse(bytes)?;

    if packets.is_empty() {
        println!("No complete packets found.");
        return Ok(());
    }

    for packet in &packets {
        let opcode = Opcode::from_u8(packet.opcode)
            .map_or_else(|| "unknown".to_string(), |opcode| opcode.to_string());
        let params = if packet.params.is_empty() {
            "-".to_string()
        } else {
            hex::format(&packet.params)
        };
        println!("0x{:02X}  {:<22} params: {}", packet.opcode, opcode, params);
    }

    let mut state = DeviceState::default();
    for packet in &packets {
        state.apply(packet);
    }

    if json {
        println!("\n{}", serde_json::to_string_pretty(&state)?);
    } else {
        println!("\n{state}");
    }

    Ok(())
}

fn run_touch(command: TouchCommand) -> Result<()> {
    match command {
        TouchCommand::Encode { bindings } => {
            let mut mapping = TouchMapping::new();

            for binding in bindings {
                let (key, function) = binding.split_once('=').ok_or_else(|| {
                    anyhow::anyhow!("invalid binding {binding:?}; expected KEY=FUNCTION")
                })?;
                let key = key
                    .parse::<TouchKey>()
                    .map_err(anyhow::Error::msg)
                    .with_context(|| format!("invalid touch key {key:?}"))?;
                let function = function
                    .parse::<TouchFunction>()
                    .map_err(anyhow::Error::msg)
                    .with_context(|| format!("invalid touch function {function:?}"))?;
                mapping.set(key, function);
            }

            println!("{}", hex::format(&mapping.encode()));
        }
        TouchCommand::Decode { data } => {
            let bytes = join_hex(&data)?;
            let mapping = TouchMapping::parse(&bytes)?;
            println!("Touch mapping:");
            print!("{mapping}");
        }
    }

    Ok(())
}

fn decode_battery(bytes: &[u8]) -> Result<()> {
    let battery = Battery::parse(bytes)?;
    println!("Left:  {}", battery.left);
    println!("Right: {}", battery.right);
    println!("Case:  {}", battery.case);
    Ok(())
}

fn decode_firmware(bytes: &[u8]) -> Result<()> {
    let firmware = FirmwareVersion::parse(bytes)?;
    println!("{firmware}");
    Ok(())
}

fn run_config(command: ConfigCommand, path: &PathBuf) -> Result<()> {
    match command {
        ConfigCommand::Path => println!("{}", path.display()),
        ConfigCommand::Show => {
            let config = Config::load_from(path)?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Init { mac } => {
            let config = Config {
                mac_address: mac,
                ..Config::default()
            };
            config.save_to(path)?;
            println!("Wrote {}", path.display());
        }
        ConfigCommand::SetMac { mac } => {
            let config = Config::load_from(path)?.with_mac(mac);
            config.save_to(path)?;
            println!("Updated {}", path.display());
        }
    }

    Ok(())
}

fn join_hex(parts: &[String]) -> Result<Vec<u8>> {
    let joined = parts.join(" ");
    hex::parse(&joined).with_context(|| format!("invalid hexadecimal input {joined:?}"))
}
