use crate::controller_client::ControllerClient;
use crate::frpc_manager::FrpcManager;
use anyhow::Result;
use frp_shared::api_types::RunningProxyReport;
use frp_shared::config::AgentConfig;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct AgentRunner {
    config: AgentConfig,
    frpc_mgr: FrpcManager,
    controller_client: ControllerClient,
}

impl AgentRunner {
    pub fn new(config: AgentConfig) -> Self {
        let frpc_mgr = FrpcManager::new(&config);
        let controller_client = ControllerClient::new(
            config.controller_url.clone(),
            config.agent_id.clone(),
            config.resolved_agent_token(),
        );

        Self {
            config,
            frpc_mgr,
            controller_client,
        }
    }

    /// Main synchronization and monitoring loop.
    pub async fn run(&self) -> Result<()> {
        info!(
            agent_id = %self.config.agent_id,
            node_id = self.config.pterodactyl_node_id,
            controller = %self.config.controller_url,
            "FRP Agent runner initialized"
        );

        let mut interval = tokio::time::interval(Duration::from_secs(self.config.heartbeat_interval_secs));

        loop {
            interval.tick().await;

            match self.controller_client.get_desired_state().await {
                Ok(desired_state) => {
                    // Update main config if needed
                    if let Err(e) = self.frpc_mgr.write_main_config(&desired_state.gateway) {
                        error!(error = %e, "Failed to write frpc main configuration");
                    }

                    // Sync proxies and reload
                    match self.frpc_mgr.sync_proxies(&desired_state.proxies).await {
                        Ok(changed) => {
                            if changed {
                                info!(
                                    active_proxies = desired_state.proxies.len(),
                                    "FRP client updated with new proxy mappings"
                                );
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to synchronize proxy configurations or reload frpc");
                        }
                    }

                    let frpc_alive = self.frpc_mgr.is_alive().await;
                    let proxy_reports: Vec<RunningProxyReport> = desired_state
                        .proxies
                        .iter()
                        .map(|p| RunningProxyReport {
                            proxy_name: p.proxy_name.clone(),
                            status: if frpc_alive { "running".to_string() } else { "stopped".to_string() },
                            last_error: None,
                        })
                        .collect();

                    if let Err(e) = self.controller_client.report_state(frpc_alive, proxy_reports).await {
                        warn!(error = %e, "Failed to report state to controller");
                    }

                    if let Err(e) = self.controller_client.send_heartbeat(frpc_alive, desired_state.proxies.len()).await {
                        warn!(error = %e, "Failed to send heartbeat to controller");
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "Could not reach controller for desired state; retaining active tunnels"
                    );
                }
            }
        }
    }
}
