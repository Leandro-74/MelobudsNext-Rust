//! Shared BLE session used by the GUI.
//!
//! Every action takes the same asynchronous mutex, so the UI can fire a task
//! per interaction without racing commands on the command characteristic. The
//! orchestration itself lives in [`melobudsnext::actions::Controller`], shared
//! with the CLI; this module only maps GUI messages to controller calls.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures::lock::Mutex;
use melobudsnext::actions::{
    CommandObserver, CommandOptions, ConnectError, Controller, ReconnectPolicy,
};
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

/// State carried by an unsolicited notification received in the background.
#[derive(Debug, Clone, Default)]
pub struct LiveUpdate {
    pub state: DeviceState,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub connected: bool,
    /// Whether the notification actually changed a known state field.
    pub changed: bool,
}

/// Cloneable handle to the single BLE session owned by the application.
#[derive(Clone)]
pub struct Session {
    inner: Arc<Mutex<Controller>>,
    reconnect: ReconnectPolicy,
}

impl Session {
    /// Builds a session that uses `reconnect` for commands that restart the
    /// headphones, such as LDAC.
    pub fn with_policy(
        uuids: BleUuids,
        target: Option<String>,
        reconnect: ReconnectPolicy,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Controller::new(uuids, target))),
            reconnect,
        }
    }

    pub async fn run(&self, action: Action, observer: &mut dyn CommandObserver) -> ActionResult {
        let mut controller = self.inner.lock().await;
        let mut result = ActionResult::default();

        if let Err(error) = execute(
            &mut controller,
            action,
            &mut result,
            self.reconnect,
            observer,
        )
        .await
        {
            result.error = Some(format!("{error:#}"));
            controller.drop_client_if_link_lost().await;
        }

        result.state = controller.state().clone();
        result.connected = controller.is_connected();
        result.address = controller.address();

        result
    }

    /// Waits for one unsolicited notification and reports the resulting state.
    ///
    /// The wait is bounded, so the caller can run this in a loop feeding the
    /// UI without blocking an action forever. A closed stream or a dead link is
    /// reported as `error` and the client is discarded when the probe confirms
    /// the loss, mirroring [`Session::run`].
    pub async fn poll(&self, wait: Duration) -> LiveUpdate {
        let mut controller = self.inner.lock().await;
        let mut update = LiveUpdate::default();

        if controller.is_connected() {
            match controller.poll_notifications(wait).await {
                Ok(result) => {
                    update.changed = result.changed;
                    update.warnings = result.warnings;
                }
                Err(error) => {
                    update.error = Some(format!("{error:#}"));
                    controller.drop_client_if_link_lost().await;
                }
            }
        }

        update.state = controller.state().clone();
        update.connected = controller.is_connected();
        update
    }

    /// Re-reads the touch mapping so external changes show up without a click.
    pub async fn refresh_touch(&self) -> LiveUpdate {
        let mut controller = self.inner.lock().await;
        let mut update = LiveUpdate::default();

        if controller.is_connected() {
            match controller.refresh_touch().await {
                Ok(changed) => update.changed = changed,
                Err(error) => {
                    // A direct read can race with another client and fail
                    // transiently while the link is still usable. Only report
                    // the failure when the probe confirms the link is gone;
                    // otherwise the next idle tick retries silently.
                    let message = format!("{error:#}");
                    controller.drop_client_if_link_lost().await;
                    update.error = background_error(message, controller.is_connected());
                }
            }
        }

        update.state = controller.state().clone();
        update.connected = controller.is_connected();
        update
    }
}

/// Keeps a background read failure visible only when the link was lost.
fn background_error(error: String, connected: bool) -> Option<String> {
    (!connected).then_some(error)
}

#[cfg(test)]
mod tests {
    use super::background_error;

    #[test]
    fn transient_background_errors_are_ignored_while_the_link_lives() {
        assert_eq!(background_error("not connected".to_string(), true), None);
    }

    #[test]
    fn background_errors_survive_a_lost_link() {
        assert_eq!(
            background_error("not connected".to_string(), false).as_deref(),
            Some("not connected")
        );
    }
}

async fn execute(
    controller: &mut Controller,
    action: Action,
    result: &mut ActionResult,
    policy: ReconnectPolicy,
    observer: &mut dyn CommandObserver,
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
            let options = CommandOptions::gui(reconnect).with_reconnect_policy(policy);
            let outcome = controller.command_with(&packet, &options, observer).await?;
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
