//! Command-line options and configuration loading for the interface.

use std::path::PathBuf;

use clap::Parser;
use melobudsnext::config::Config;

/// Options accepted by the graphical interface.
#[derive(Debug, Parser)]
#[command(
    name = "melobudsnext-gui",
    version,
    about = "Painel gráfico para o QCY Melobuds Pro"
)]
pub(crate) struct Cli {
    /// Caminho do arquivo de configuração (padrão: ~/.melobudsnext/config.json).
    #[arg(long, value_name = "PATH")]
    pub(crate) config: Option<PathBuf>,

    /// Abre a interface sem acessar o Bluetooth (demonstração e capturas).
    #[arg(long)]
    pub(crate) offline: bool,
}

/// Loads the configuration from `--config PATH`, or from the default path when
/// the flag is absent. Errors are reported in the UI and fall back to defaults,
/// matching the behavior of the previous `Config::load` call.
pub(crate) fn load_configuration(cli: &Cli) -> (Config, Option<String>) {
    let result = match &cli.config {
        Some(path) => Config::load_from(path),
        None => Config::load(),
    };

    match result {
        Ok(config) => (config, None),
        Err(error) => (
            Config::default(),
            Some(format!("não foi possível ler a configuração: {error}")),
        ),
    }
}

/// Interprets `MELOBUDNEXT_GUI_OFFLINE` as a boolean instead of a presence
/// flag, so `0`, `false`, `no`, `off` and an empty value keep the GUI online.
/// Unrecognized non-empty values are treated as enabled for backwards
/// compatibility with `MELOBUDNEXT_GUI_OFFLINE=anything`.
pub(crate) fn offline_flag(value: Option<&str>) -> bool {
    match value {
        None => false,
        Some(value) => !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "" | "0" | "false" | "no" | "off"
        ),
    }
}
