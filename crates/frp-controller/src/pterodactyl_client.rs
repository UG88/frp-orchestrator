use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone)]
pub struct PterodactylClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PteroAllocationItem {
    pub id: u64,
    pub ip: String,
    pub alias: Option<String>,
    pub port: u16,
    pub assigned: bool,
    pub server_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PteroServerItem {
    pub id: u64,
    pub uuid: String,
    pub name: String,
    pub node: u64,
    pub allocation: u64,
    pub egg: u64,
    pub docker_image: Option<String>,
}

#[derive(Deserialize)]
struct PteroListResponse<T> {
    data: Vec<PteroDataItem<T>>,
}

#[derive(Deserialize)]
struct PteroDataItem<T> {
    attributes: T,
}

#[derive(Deserialize)]
struct PteroAllocationAttributes {
    id: u64,
    ip: String,
    alias: Option<String>,
    port: u16,
    assigned: bool,
    #[serde(default)]
    server: Option<u64>,
}

#[derive(Deserialize)]
struct PteroServerAttributes {
    id: u64,
    uuid: String,
    name: String,
    node: u64,
    allocation: u64,
    egg: u64,
    #[serde(default)]
    docker_image: Option<String>,
}

impl PterodactylClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", self.api_key);
        let mut auth_header = HeaderValue::from_str(&auth_val)?;
        auth_header.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// Fetch all allocations for a given Pterodactyl node.
    pub async fn get_node_allocations(&self, ptero_node_id: u64) -> Result<Vec<PteroAllocationItem>> {
        let url = format!("{}/api/application/nodes/{}/allocations", self.base_url, ptero_node_id);
        let headers = self.headers()?;

        debug!(url = %url, "Fetching allocations from Pterodactyl API");
        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to connect to Pterodactyl API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Pterodactyl API error (status {}): {}", status, body);
        }

        let body: PteroListResponse<PteroAllocationAttributes> = resp
            .json()
            .await
            .context("Failed to deserialize Pterodactyl allocations response")?;

        let items = body
            .data
            .into_iter()
            .map(|d| PteroAllocationItem {
                id: d.attributes.id,
                ip: d.attributes.ip,
                alias: d.attributes.alias,
                port: d.attributes.port,
                assigned: d.attributes.assigned,
                server_id: d.attributes.server,
            })
            .collect();

        Ok(items)
    }

    /// Fetch all servers to resolve egg/metadata info.
    pub async fn get_servers(&self) -> Result<Vec<PteroServerItem>> {
        let url = format!("{}/api/application/servers", self.base_url);
        let headers = self.headers()?;

        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to query Pterodactyl servers")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Pterodactyl API servers error (status {}): {}", status, body);
        }

        let body: PteroListResponse<PteroServerAttributes> = resp
            .json()
            .await
            .context("Failed to deserialize Pterodactyl servers response")?;

        let items = body
            .data
            .into_iter()
            .map(|d| PteroServerItem {
                id: d.attributes.id,
                uuid: d.attributes.uuid,
                name: d.attributes.name,
                node: d.attributes.node,
                allocation: d.attributes.allocation,
                egg: d.attributes.egg,
                docker_image: d.attributes.docker_image,
            })
            .collect();

        Ok(items)
    }
}
