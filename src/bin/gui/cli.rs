//! Command-line options and configuration loading for the interface.

use std::path::PathBuf;

use clap::Parser;
use melobudsnext::actions::RECONNECT_ATTEMPTS;
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

    /// Tentativas de reconexão após o reinício do LDAC (padrão: 10).
    #[arg(long, default_value_t = RECONNECT_ATTEMPTS, value_parser = parse_reconnect_attempts)]
    pub(crate) reconnect_attempts: u32,

    /// Segundos entre as tentativas de reconexão do LDAC (padrão: 3).
    #[arg(long, default_value_t = 3.0, value_parser = parse_reconnect_delay)]
    pub(crate) reconnect_delay: f64,
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

fn parse_reconnect_attempts(raw: &str) -> Result<u32, String> {
    let attempts = raw
        .parse::<u32>()
        .map_err(|_| "as tentativas de reconexão devem ser um número inteiro".to_string())?;
    melobudsnext::cli::reconnect_attempts(attempts).map_err(|error| error.to_string())
}

fn parse_reconnect_delay(raw: &str) -> Result<f64, String> {
    let seconds = raw
        .parse::<f64>()
        .map_err(|_| "o intervalo de reconexão deve ser um número de segundos".to_string())?;
    melobudsnext::cli::reconnect_delay(seconds)
        .map(|_| seconds)
        .map_err(|error| error.to_string())
}
