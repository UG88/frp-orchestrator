use crate::models::Protocol;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Resolve an environment variable expression if prefixed with `ENV:`.
pub fn resolve_secret(val: &str) -> String {
    if let Some(var_name) = val.strip_prefix("ENV:") {
        env::var(var_name).unwrap_or_else(|_| val.to_string())
    } else {
        val.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRangeConfig {
    pub start: u16,
    pub end: u16,
}

impl PortRangeConfig {
    pub fn validate(&self, name: &str) -> Result<()> {
        if self.start == 0 {
            bail!("{}: start port cannot be 0", name);
        }
        if self.start > self.end {
            bail!("{}: start port ({}) is greater than end port ({})", name, self.start, self.end);
        }
        Ok(())
    }

    pub fn total_ports(&self) -> u32 {
        (self.end - self.start + 1) as u32
    }

    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub id: String,
    pub region: String,
    pub public_ip: String,
    #[serde(default = "default_control_port")]
    pub control_port: u16,
    pub tcp_port_range: PortRangeConfig,
    pub udp_port_range: PortRangeConfig,
    #[serde(default)]
    pub reserved_ports: Vec<u16>,
    pub token: String,
    #[serde(default = "default_true")]
    pub tls_enable: bool,
    pub dashboard_port: Option<u16>,
}

fn default_control_port() -> u16 {
    7000
}

fn default_true() -> bool {
    true
}

impl GatewayConfig {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            bail!("Gateway id cannot be empty");
        }
        if self.public_ip.trim().is_empty() {
            bail!("Gateway {}: public_ip cannot be empty", self.id);
        }
        self.tcp_port_range.validate(&format!("Gateway {} TCP", self.id))?;
        self.udp_port_range.validate(&format!("Gateway {} UDP", self.id))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PterodactylConfig {
    pub url: String,
    pub api_key: String,
    #[serde(default = "default_ptero_sync_interval")]
    pub sync_interval_secs: u64,
    #[serde(default)]
    pub auto_expose_default: bool,
    #[serde(default)]
    pub egg_protocol_overrides: HashMap<String, Protocol>,
}

fn default_ptero_sync_interval() -> u64 {
    30
}

impl PterodactylConfig {
    pub fn resolved_api_key(&self) -> String {
        resolve_secret(&self.api_key)
    }

    pub fn validate(&self) -> Result<()> {
        if self.url.trim().is_empty() {
            bail!("Pterodactyl URL cannot be empty");
        }
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            bail!("Pterodactyl URL must start with http:// or https://");
        }
        if self.resolved_api_key().trim().is_empty() {
            bail!("Pterodactyl API key cannot be empty");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_token: String,
    #[serde(default)]
    pub zone_id: String,
    #[serde(default)]
    pub proxied: bool,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_ttl() -> u32 {
    1 // Auto TTL for Cloudflare
}

impl CloudflareConfig {
    pub fn resolved_api_token(&self) -> String {
        resolve_secret(&self.api_token)
    }

    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.resolved_api_token().trim().is_empty() {
                bail!("Cloudflare is enabled but api_token is empty");
            }
            if self.zone_id.trim().is_empty() {
                bail!("Cloudflare is enabled but zone_id is empty");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route53Config {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub hosted_zone_id: String,
    #[serde(default)]
    pub region: String,
}

impl Route53Config {
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.hosted_zone_id.trim().is_empty() {
            bail!("Route53 is enabled but hosted_zone_id is empty");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub default_domain: Option<String>,
    #[serde(default)]
    pub cloudflare: Option<CloudflareConfig>,
    #[serde(default)]
    pub route53: Option<Route53Config>,
}

impl DnsConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(cf) = &self.cloudflare {
            cf.validate()?;
        }
        if let Some(r53) = &self.route53 {
            r53.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    #[serde(default = "default_controller_listen")]
    pub listen_addr: String,
    pub api_key: String,
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_reconciliation_interval")]
    pub reconciliation_interval_secs: u64,
    #[serde(default)]
    pub default_proxy_protocol: Option<String>,
    pub pterodactyl: PterodactylConfig,
    #[serde(default)]
    pub dns: Option<DnsConfig>,
    #[serde(default)]
    pub gateways: Vec<GatewayConfig>,
}

fn default_controller_listen() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_database_path() -> String {
    "controller.db".to_string()
}

fn default_reconciliation_interval() -> u64 {
    30
}

impl ControllerConfig {
    pub fn resolved_api_key(&self) -> String {
        resolve_secret(&self.api_key)
    }

    pub fn validate(&self) -> Result<()> {
        if self.resolved_api_key().trim().is_empty() {
            bail!("Controller api_key cannot be empty");
        }
        self.pterodactyl.validate().context("Pterodactyl configuration validation failed")?;
        
        if let Some(dns) = &self.dns {
            dns.validate().context("DNS configuration validation failed")?;
        }

        let mut seen_ids = std::collections::HashSet::new();
        for gw in &self.gateways {
            if !seen_ids.insert(&gw.id) {
                bail!("Duplicate gateway id found: {}", gw.id);
            }
            gw.validate().context(format!("Gateway {} configuration error", gw.id))?;
        }
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ControllerConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub controller_url: String,
    pub agent_token: String,
    pub pterodactyl_node_id: u64,
    #[serde(default = "default_frpc_bin")]
    pub frpc_binary_path: String,
    #[serde(default = "default_frpc_conf_dir")]
    pub frpc_config_dir: String,
    #[serde(default = "default_frpc_main_conf")]
    pub frpc_main_config: String,
    #[serde(default = "default_frpc_admin_addr")]
    pub frpc_admin_addr: String,
    #[serde(default = "default_frpc_admin_user")]
    pub frpc_admin_user: String,
    #[serde(default = "default_frpc_admin_password")]
    pub frpc_admin_password: String,
    #[serde(default = "default_agent_heartbeat")]
    pub heartbeat_interval_secs: u64,
}

fn default_frpc_bin() -> String {
    "/opt/frp/frpc".to_string()
}

fn default_frpc_conf_dir() -> String {
    "/opt/frp/conf.d".to_string()
}

fn default_frpc_main_conf() -> String {
    "/opt/frp/frpc.toml".to_string()
}

fn default_frpc_admin_addr() -> String {
    "127.0.0.1:7400".to_string()
}

fn default_frpc_admin_user() -> String {
    "admin".to_string()
}

fn default_frpc_admin_password() -> String {
    "admin".to_string()
}

fn default_agent_heartbeat() -> u64 {
    15
}

impl AgentConfig {
    pub fn resolved_agent_token(&self) -> String {
        resolve_secret(&self.agent_token)
    }

    pub fn resolved_admin_password(&self) -> String {
        resolve_secret(&self.frpc_admin_password)
    }

    pub fn validate(&self) -> Result<()> {
        if self.agent_id.trim().is_empty() {
            bail!("Agent agent_id cannot be empty");
        }
        if self.controller_url.trim().is_empty() {
            bail!("Agent controller_url cannot be empty");
        }
        if self.resolved_agent_token().trim().is_empty() {
            bail!("Agent agent_token cannot be empty");
        }
        if self.pterodactyl_node_id == 0 {
            bail!("Agent pterodactyl_node_id cannot be 0");
        }
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AgentConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
}
