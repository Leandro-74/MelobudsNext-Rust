//! Shared BLE session used by the GUI.
//!
//! Every action takes the same asynchronous mutex, so the UI can fire a task
//! per interaction without racing commands on the command characteristic. The
//! session itself stays open between actions; only `Connect` and `Disconnect`
//! change the link.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures::lock::Mutex;
use melobudsnext::ble::BleSession;
use melobudsnext::config::BleUuids;
use melobudsnext::protocol::commands;
use melobudsnext::protocol::framing::Packet;
use melobudsnext::protocol::touch::TouchMapping;
use melobudsnext::DeviceState;

const SYNC_WINDOW: Duration = Duration::from_millis(1500);
const COMMAND_WINDOW: Duration = Duration::from_millis(450);
const SCAN_TIMEOUT: Duration = Duration::from_secs(8);
const RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_DELAY: Duration = Duration::from_secs(3);

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
    inner: Arc<Mutex<SessionInner>>,
}

struct SessionInner {
    client: Option<BleSession>,
    target: Option<String>,
    uuids: BleUuids,
    state: DeviceState,
}

impl Session {
    pub fn new(uuids: BleUuids, target: Option<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                client: None,
                target,
                uuids,
                state: DeviceState::default(),
            })),
        }
    }

    pub async fn run(&self, action: Action) -> ActionResult {
        let mut inner = self.inner.lock().await;
        let mut result = ActionResult::default();

        if let Err(error) = inner.execute(action, &mut result).await {
            result.error = Some(format!("{error:#}"));
        }

        result.state = inner.state.clone();
        result.connected = inner.client.is_some();
        result.address = inner
            .client
            .as_ref()
            .map(BleSession::address)
            .or_else(|| inner.target.clone());

        result
    }
}

impl SessionInner {
    async fn execute(&mut self, action: Action, result: &mut ActionResult) -> Result<()> {
        match action {
            Action::Connect { address } => {
                self.target = Some(address.trim().to_string());
                let warnings = self.connect_and_sync().await?;
                result.warnings.extend(warnings);
                result.notice = Some("Conectado e sincronizado com o fone.".to_string());
                result.refresh_drafts = true;
            }
            Action::Refresh => {
                if self.client.is_none() {
                    let warnings = self.connect_and_sync().await?;
                    result.warnings.extend(warnings);
                } else {
                    let warnings = self.sync_current().await?;
                    result.warnings.extend(warnings);
                }
                result.notice = Some("Estado atualizado a partir do fone.".to_string());
                result.refresh_drafts = true;
            }
            Action::Disconnect => {
                if let Some(client) = self.client.take() {
                    client.disconnect().await?;
                }
                result.notice = Some("Fone desconectado.".to_string());
            }
            Action::Command {
                packet,
                reconnect,
                refresh_drafts,
                label,
            } => {
                let values = {
                    let client = self.client_ref()?;
                    client.exchange(&packet, COMMAND_WINDOW).await?
                };

                for value in values {
                    if let Err(error) = self.state.apply_bytes(&value) {
                        result
                            .warnings
                            .push(format!("notificação inválida: {error}"));
                    }
                }

                if reconnect {
                    let report = {
                        let client = self.client_mut()?;
                        client
                            .reconnect(RECONNECT_ATTEMPTS, RECONNECT_DELAY)
                            .await?;
                        client.sync(SYNC_WINDOW).await?
                    };
                    self.state = report.state;
                    result.warnings.extend(report.warnings);
                    result.refresh_drafts = true;
                }

                if refresh_drafts {
                    result.refresh_drafts = true;
                }

                result.notice = Some(label.to_string());
            }
            Action::Touch { mapping } => {
                let client = self.client_ref()?;
                client.write_touch(&mapping).await?;
                self.state.touch = Some(mapping);
                result.notice = Some(
                    "Mapa de toque gravado no fone. Os novos gestos já estão ativos.".to_string(),
                );
            }
            Action::Rename { name } => {
                let packet = commands::rename(&name).map_err(anyhow::Error::new)?;
                let values = {
                    let client = self.client_ref()?;
                    client.exchange(&packet, COMMAND_WINDOW).await?
                };

                for value in values {
                    if let Err(error) = self.state.apply_bytes(&value) {
                        result
                            .warnings
                            .push(format!("notificação inválida: {error}"));
                    }
                }

                // Some firmware revisions do not echo the rename; keep the
                // requested name in that case.
                if self.state.name.as_deref() != Some(name.as_str()) {
                    self.state.name = Some(name);
                }
                result.notice = Some("Nome do dispositivo atualizado.".to_string());
            }
        }

        Ok(())
    }

    fn client_ref(&self) -> Result<&BleSession> {
        self.client
            .as_ref()
            .ok_or_else(|| anyhow!("fone não conectado; clique em Conectar"))
    }

    fn client_mut(&mut self) -> Result<&mut BleSession> {
        self.client
            .as_mut()
            .ok_or_else(|| anyhow!("fone não conectado; clique em Conectar"))
    }

    async fn connect_and_sync(&mut self) -> Result<Vec<String>> {
        if let Some(client) = self.client.take() {
            let _ = client.disconnect().await;
        }

        let target = self
            .target
            .clone()
            .filter(|target| !target.trim().is_empty())
            .ok_or_else(|| anyhow!("informe o MAC ou o ID do fone"))?;

        let client = BleSession::connect(&target, &self.uuids, SCAN_TIMEOUT)
            .await
            .with_context(|| format!("falha ao conectar em {target}"))?;

        let report = match client.sync(SYNC_WINDOW).await {
            Ok(report) => report,
            Err(error) => {
                let _ = client.disconnect().await;
                return Err(error).context("conectou, mas não foi possível sincronizar o estado");
            }
        };

        self.state = report.state;
        self.target = Some(client.address());
        self.client = Some(client);

        Ok(report.warnings)
    }

    async fn sync_current(&mut self) -> Result<Vec<String>> {
        let report = {
            let client = self.client_ref()?;
            client.sync(SYNC_WINDOW).await?
        };
        self.state = report.state;
        Ok(report.warnings)
    }
}
