//! Shared BLE session used by the GUI.
//!
//! Every action takes the same asynchronous mutex, so the UI can fire a task
//! per interaction without racing commands on the command characteristic. The
//! orchestration itself lives in [`melobudsnext::actions::Controller`], shared
//! with the CLI; this module only maps GUI messages to controller calls.

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use futures::lock::Mutex;
use melobudsnext::actions::{CommandOptions, ConnectError, Controller};
use melobudsnext::config::BleUuids;
use melobudsnext::protocol::framing::Packet;
use melobudsnext::protocol::touch::TouchMapping;
use melobudsnext::DeviceState;

/// Something the GUI can ask the headphones to do.
#[derive(Debug, Clone)]
pub enum Action {
    Connect {
        address: String,
    },
    Refresh,
    Disconnect,
    Command {
        packet: Packet,
        reconnect: bool,
        refresh_drafts: bool,
        label: &'static str,
    },
    Touch {
        mapping: TouchMapping,
    },
    Rename {
        name: String,
    },
}

/// Result of one action, including the state snapshot the UI should render.
#[derive(Debug, Clone, Default)]
pub struct ActionResult {
    pub state: DeviceState,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub warnings: Vec<String>,
    pub connected: bool,
    pub address: Option<String>,
    /// Whether draft controls (sliders, pick lists, text fields) should be
    /// reloaded from the device state.
    pub refresh_drafts: bool,
}

/// Cloneable handle to the single BLE session owned by the application.
#[derive(Clone)]
pub struct Session {
    inner: Arc<Mutex<Controller>>,
}

impl Session {
    pub fn new(uuids: BleUuids, target: Option<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Controller::new(uuids, target))),
        }
    }

    pub async fn run(&self, action: Action) -> ActionResult {
        let mut controller = self.inner.lock().await;
        let mut result = ActionResult::default();

        if let Err(error) = execute(&mut controller, action, &mut result).await {
            result.error = Some(format!("{error:#}"));
            controller.drop_client_if_link_lost().await;
        }

        result.state = controller.state().clone();
        result.connected = controller.is_connected();
        result.address = controller.address();

        result
    }
}

async fn execute(
    controller: &mut Controller,
    action: Action,
    result: &mut ActionResult,
) -> Result<()> {
    match action {
        Action::Connect { address } => {
            let target = address.trim().to_string();
            controller.set_target(target.clone());
            result.warnings.extend(connect(controller, &target).await?);
            result.notice = Some("Conectado e sincronizado com o fone.".to_string());
            result.refresh_drafts = true;
        }
        Action::Refresh => {
            if controller.is_connected() {
                result.warnings.extend(controller.sync_current().await?);
            } else {
                let target = controller
                    .target()
                    .map(str::trim)
                    .filter(|target| !target.is_empty())
                    .map(str::to_string)
                    .ok_or_else(|| anyhow!("informe o MAC ou o ID do fone"))?;
                result.warnings.extend(connect(controller, &target).await?);
            }
            result.notice = Some("Estado atualizado a partir do fone.".to_string());
            result.refresh_drafts = true;
        }
        Action::Disconnect => {
            controller.disconnect().await?;
            result.notice = Some("Fone desconectado.".to_string());
        }
        Action::Command {
            packet,
            reconnect,
            refresh_drafts,
            label,
        } => {
            require_connected(controller)?;
            let outcome = controller
                .command(&packet, &CommandOptions::gui(reconnect))
                .await?;
            result.warnings.extend(outcome.warnings);
            if reconnect || refresh_drafts {
                result.refresh_drafts = true;
            }
            result.notice = Some(label.to_string());
        }
        Action::Touch { mapping } => {
            require_connected(controller)?;
            controller.write_touch(&mapping).await?;
            result.notice =
                Some("Mapa de toque gravado no fone. Os novos gestos já estão ativos.".to_string());
        }
        Action::Rename { name } => {
            require_connected(controller)?;
            controller.rename(&name).await?;
            result.notice = Some("Nome do dispositivo atualizado.".to_string());
        }
    }

    Ok(())
}

fn require_connected(controller: &Controller) -> Result<()> {
    if controller.is_connected() {
        Ok(())
    } else {
        Err(anyhow!("fone não conectado; clique em Conectar"))
    }
}

async fn connect(controller: &mut Controller, target: &str) -> Result<Vec<String>> {
    match controller.connect_to(target).await {
        Ok(warnings) => Ok(warnings),
        Err(ConnectError::Connect(error)) => {
            Err(error).with_context(|| format!("falha ao conectar em {target}"))
        }
        Err(ConnectError::Sync(error)) => {
            Err(error).context("conectou, mas não foi possível sincronizar o estado")
        }
    }
}
