//! Tests for the interface command line and configuration loading.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use melobudsnext::config::Config;

use crate::cli::{load_configuration, offline_flag, Cli};

fn temporary_directory() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "melobudsnext-gui-test-{}-{unique}",
        std::process::id()
    ))
}

#[test]
fn parses_the_config_flag() {
    let cli = Cli::try_parse_from(["melobudsnext-gui", "--config", "/tmp/perfil.json"]).unwrap();

    assert_eq!(
        cli.config.as_deref(),
        Some(std::path::Path::new("/tmp/perfil.json"))
    );
    assert!(!cli.offline);
}

#[test]
fn loads_the_configuration_from_the_requested_path() {
    let directory = temporary_directory();
    let path = directory.join("config.json");
    Config::default()
        .with_mac("AA:BB:CC:DD:EE:FF")
        .save_to(&path)
        .unwrap();

    let cli =
        Cli::try_parse_from(["melobudsnext-gui", "--config", path.to_str().unwrap()]).unwrap();
    let (config, error) = load_configuration(&cli);

    assert!(error.is_none());
    assert_eq!(config.mac_address.as_deref(), Some("AA:BB:CC:DD:EE:FF"));

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reports_configuration_errors_when_using_the_flag() {
    let directory = temporary_directory();
    std::fs::create_dir_all(&directory).unwrap();

    // A directory cannot be read as a file, so loading must report an error.
    let cli =
        Cli::try_parse_from(["melobudsnext-gui", "--config", directory.to_str().unwrap()]).unwrap();
    let (config, error) = load_configuration(&cli);

    assert_eq!(config, Config::default());
    assert!(error.unwrap().contains("configuração"));

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn parses_the_offline_flag_with_clap() {
    let cli = Cli::try_parse_from(["melobudsnext-gui", "--offline"]).unwrap();
    assert!(cli.offline);

    let cli = Cli::try_parse_from(["melobudsnext-gui"]).unwrap();
    assert!(!cli.offline);
}

#[test]
fn rejects_positional_arguments_instead_of_ignoring_them() {
    assert!(Cli::try_parse_from(["melobudsnext-gui", "unexpected"]).is_err());
}

#[test]
fn exposes_help_and_version() {
    use clap::CommandFactory;

    assert_eq!(
        Cli::try_parse_from(["melobudsnext-gui", "--help"])
            .unwrap_err()
            .kind(),
        clap::error::ErrorKind::DisplayHelp
    );

    let mut command = Cli::command();
    let help = command.render_help().to_string();
    assert!(help.contains("--offline"), "help was: {help}");
    assert_eq!(command.get_version(), Some(env!("CARGO_PKG_VERSION")));
}

#[test]
fn treats_absent_or_false_values_as_online() {
    for value in [
        None,
        Some(""),
        Some("   "),
        Some("0"),
        Some("false"),
        Some("NO"),
        Some("off"),
    ] {
        assert!(
            !offline_flag(value),
            "value {value:?} should keep the GUI online"
        );
    }
}

#[test]
fn treats_explicit_truthy_values_as_offline() {
    for value in [Some("1"), Some("true"), Some("YES"), Some(" on ")] {
        assert!(
            offline_flag(value),
            "value {value:?} should enable offline mode"
        );
    }
}

#[test]
fn cor_10_help_and_version_cover_the_gui_flags() {
    use clap::CommandFactory;

    let mut command = Cli::command();
    let help = command.render_help().to_string();

    for flag in ["--offline", "--config"] {
        assert!(help.contains(flag), "help was: {help}");
    }

    assert_eq!(
        Cli::try_parse_from(["melobudsnext-gui", "--version"])
            .unwrap_err()
            .kind(),
        clap::error::ErrorKind::DisplayVersion
    );
}
