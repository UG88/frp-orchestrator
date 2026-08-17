use anyhow::{bail, Context, Result};
use frp_shared::api_types::{
    AgentDesiredStateResponse, AgentReportStateRequest, HeartbeatNodeRequest, RegisterNodeRequest,
    RegisterNodeResponse, RunningProxyReport,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::time::Duration;
use tracing::debug;

#[derive(Clone)]
pub struct ControllerClient {
    base_url: String,
    agent_id: String,
    agent_token: String,
    client: reqwest::Client,
}

impl ControllerClient {
    pub fn new(base_url: String, agent_id: String, agent_token: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            agent_id,
            agent_token,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", self.agent_token);
        let mut auth_header = HeaderValue::from_str(&auth_val)?;
        auth_header.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    pub async fn register_node(
        &self,
        name: &str,
        pterodactyl_node_id: u64,
        assigned_gateway_id: &str,
        local_ip: &str,
    ) -> Result<RegisterNodeResponse> {
        let url = format!("{}/api/v1/nodes/register", self.base_url);
        let headers = self.headers()?;

        let payload = RegisterNodeRequest {
            node_id: self.agent_id.clone(),
            name: name.to_string(),
            pterodactyl_node_id,
            assigned_gateway_id: assigned_gateway_id.to_string(),
            local_ip: local_ip.to_string(),
            agent_token: self.agent_token.clone(),
        };

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to connect to controller register endpoint")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Node registration failed (status {}): {}", status, body);
        }

        let data: RegisterNodeResponse = resp.json().await?;
        Ok(data)
    }

    pub async fn get_desired_state(&self) -> Result<AgentDesiredStateResponse> {
        let url = format!("{}/api/v1/agent/{}/desired-state", self.base_url, self.agent_id);
        let headers = self.headers()?;

        debug!(url = %url, "Fetching desired state from controller");
        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to fetch desired state from controller")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to get desired state (status {}): {}", status, body);
        }

        let state: AgentDesiredStateResponse = resp.json().await?;
        Ok(state)
    }

    pub async fn send_heartbeat(&self, is_healthy: bool, running_proxies_count: usize) -> Result<()> {
        let url = format!("{}/api/v1/nodes/{}/heartbeat", self.base_url, self.agent_id);
        let headers = self.headers()?;

        let payload = HeartbeatNodeRequest {
            node_id: self.agent_id.clone(),
            is_healthy,
            running_proxies_count,
            error: None,
        };

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to send heartbeat to controller")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Heartbeat rejected (status {}): {}", status, body);
        }

        Ok(())
    }

    pub async fn report_state(&self, frpc_running: bool, running_proxies: Vec<RunningProxyReport>) -> Result<()> {
        let url = format!("{}/api/v1/agent/{}/report-state", self.base_url, self.agent_id);
        let headers = self.headers()?;

        let payload = AgentReportStateRequest {
            node_id: self.agent_id.clone(),
            frpc_running,
            running_proxies,
        };

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to report state to controller")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("State report rejected (status {}): {}", status, body);
        }

        Ok(())
    }
}
