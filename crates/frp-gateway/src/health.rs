use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct GatewayTelemetry {
    gateway_id: String,
    controller_url: String,
    controller_token: String,
    dashboard_port: u16,
    client: reqwest::Client,
}

impl GatewayTelemetry {
    pub fn new(
        gateway_id: String,
        controller_url: String,
        controller_token: String,
        dashboard_port: u16,
    ) -> Self {
        Self {
            gateway_id,
            controller_url: controller_url.trim_end_matches('/').to_string(),
            controller_token,
            dashboard_port,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Check if frps dashboard is alive locally.
    pub async fn check_frps_alive(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/api/serverinfo", self.dashboard_port);
        matches!(self.client.get(&url).send().await, Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 401)
    }

    /// Send heartbeat to the controller.
    pub async fn send_heartbeat(&self, is_healthy: bool) -> Result<()> {
        let url = format!("{}/api/v1/gateways/{}/heartbeat", self.controller_url, self.gateway_id);

        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", self.controller_token);
        if let Ok(mut h) = HeaderValue::from_str(&auth_val) {
            h.set_sensitive(true);
            headers.insert(AUTHORIZATION, h);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "is_healthy": is_healthy
        });

        debug!(url = %url, "Sending gateway heartbeat to controller");
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to send gateway heartbeat")?;

        if !resp.status().is_success() {
            warn!(status = %resp.status(), "Gateway heartbeat was rejected by controller");
        }

        Ok(())
    }

    /// Run background telemetry loop.
    pub async fn run(&self, interval_secs: u64) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        info!(gateway = %self.gateway_id, "Starting gateway telemetry loop");

        loop {
            interval.tick().await;
            let alive = self.check_frps_alive().await;
            if let Err(e) = self.send_heartbeat(alive).await {
                debug!(error = %e, "Gateway heartbeat failed");
            }
        }
    }
}
