use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Return the name of the DNS provider
    fn provider_name(&self) -> &'static str;

    /// Create or update an A/CNAME record pointing to the gateway IP.
    async fn create_or_update_record(&self, fqdn: &str, target_ip: &str) -> Result<()>;

    /// Delete an existing DNS record.
    async fn delete_record(&self, fqdn: &str) -> Result<()>;
}

/// Manual DNS provider (default mode).
/// Logs and returns success without making external API calls.
#[derive(Debug, Default, Clone)]
pub struct ManualProvider;

#[async_trait]
impl DnsProvider for ManualProvider {
    fn provider_name(&self) -> &'static str {
        "manual"
    }

    async fn create_or_update_record(&self, fqdn: &str, target_ip: &str) -> Result<()> {
        info!(fqdn = %fqdn, target_ip = %target_ip, "Manual DNS: Record mapping registered (no external API call)");
        Ok(())
    }

    async fn delete_record(&self, fqdn: &str) -> Result<()> {
        info!(fqdn = %fqdn, "Manual DNS: Record mapping removed (no external API call)");
        Ok(())
    }
}

/// Cloudflare DNS provider for automated A-record provisioning.
#[derive(Clone)]
pub struct CloudflareProvider {
    api_token: String,
    zone_id: String,
    proxied: bool,
    ttl: u32,
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(api_token: String, zone_id: String, proxied: bool, ttl: u32) -> Self {
        Self {
            api_token,
            zone_id,
            proxied,
            ttl,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", self.api_token);
        let mut auth_header = HeaderValue::from_str(&auth_val)?;
        auth_header.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }
}

#[derive(Deserialize)]
struct CfListResponse {
    success: bool,
    result: Vec<CfRecord>,
    errors: Option<Vec<CfError>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CfRecord {
    id: String,
    name: String,
    content: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CfError {
    code: i64,
    message: String,
}

#[derive(Deserialize)]
struct CfMutationResponse {
    success: bool,
    errors: Option<Vec<CfError>>,
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    fn provider_name(&self) -> &'static str {
        "cloudflare"
    }

    async fn create_or_update_record(&self, fqdn: &str, target_ip: &str) -> Result<()> {
        let headers = self.headers()?;
        let list_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type=A",
            self.zone_id, fqdn
        );

        let resp = self
            .client
            .get(&list_url)
            .headers(headers.clone())
            .send()
            .await
            .context("Failed to send Cloudflare list records request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Cloudflare API returned status {}: {}", status, body);
        }

        let data: CfListResponse = resp.json().await.context("Failed to parse Cloudflare list response")?;
        if !data.success {
            bail!("Cloudflare API error: {:?}", data.errors);
        }

        if let Some(existing) = data.result.into_iter().next() {
            if existing.content == target_ip {
                info!(fqdn = %fqdn, target_ip = %target_ip, "Cloudflare DNS record already up to date");
                return Ok(());
            }

            let update_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, existing.id
            );
            let payload = json!({
                "type": "A",
                "name": fqdn,
                "content": target_ip,
                "ttl": self.ttl,
                "proxied": self.proxied,
            });

            let update_resp = self
                .client
                .put(&update_url)
                .headers(headers)
                .json(&payload)
                .send()
                .await
                .context("Failed to update Cloudflare DNS record")?;

            let mut_data: CfMutationResponse = update_resp.json().await.context("Failed to parse Cloudflare update response")?;
            if !mut_data.success {
                bail!("Failed to update Cloudflare DNS record: {:?}", mut_data.errors);
            }
            info!(fqdn = %fqdn, target_ip = %target_ip, "Updated Cloudflare DNS A record");
        } else {
            let create_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                self.zone_id
            );
            let payload = json!({
                "type": "A",
                "name": fqdn,
                "content": target_ip,
                "ttl": self.ttl,
                "proxied": self.proxied,
            });

            let create_resp = self
                .client
                .post(&create_url)
                .headers(headers)
                .json(&payload)
                .send()
                .await
                .context("Failed to create Cloudflare DNS record")?;

            let mut_data: CfMutationResponse = create_resp.json().await.context("Failed to parse Cloudflare create response")?;
            if !mut_data.success {
                bail!("Failed to create Cloudflare DNS record: {:?}", mut_data.errors);
            }
            info!(fqdn = %fqdn, target_ip = %target_ip, "Created Cloudflare DNS A record");
        }

        Ok(())
    }

    async fn delete_record(&self, fqdn: &str) -> Result<()> {
        let headers = self.headers()?;
        let list_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type=A",
            self.zone_id, fqdn
        );

        let resp = self
            .client
            .get(&list_url)
            .headers(headers.clone())
            .send()
            .await
            .context("Failed to send Cloudflare list records request")?;

        let data: CfListResponse = resp.json().await.context("Failed to parse Cloudflare list response")?;
        for record in data.result {
            let del_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, record.id
            );
            let del_resp = self.client.delete(&del_url).headers(headers.clone()).send().await?;
            let mut_data: CfMutationResponse = del_resp.json().await?;
            if !mut_data.success {
                warn!(fqdn = %fqdn, "Failed to delete Cloudflare DNS record: {:?}", mut_data.errors);
            } else {
                info!(fqdn = %fqdn, record_id = %record.id, "Deleted Cloudflare DNS record");
            }
        }
        Ok(())
    }
}

/// Mock DNS provider for deterministic tests.
#[derive(Clone, Default)]
pub struct MockDnsProvider {
    pub records: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

#[async_trait]
impl DnsProvider for MockDnsProvider {
    fn provider_name(&self) -> &'static str {
        "mock"
    }

    async fn create_or_update_record(&self, fqdn: &str, target_ip: &str) -> Result<()> {
        let mut map = self.records.lock().unwrap();
        map.insert(fqdn.to_string(), target_ip.to_string());
        Ok(())
    }

    async fn delete_record(&self, fqdn: &str) -> Result<()> {
        let mut map = self.records.lock().unwrap();
        map.remove(fqdn);
        Ok(())
    }
}
