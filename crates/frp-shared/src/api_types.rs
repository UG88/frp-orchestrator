use crate::models::{Gateway, Node, Protocol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database_ok: bool,
    pub total_gateways: usize,
    pub healthy_gateways: usize,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub active_mappings: usize,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeRequest {
    pub node_id: String,
    pub name: String,
    pub pterodactyl_node_id: u64,
    pub assigned_gateway_id: String,
    pub local_ip: String,
    pub agent_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeResponse {
    pub node: Node,
    pub assigned_gateway: Gateway,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatNodeRequest {
    pub node_id: String,
    pub is_healthy: bool,
    pub running_proxies_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDesiredProxy {
    pub proxy_name: String,
    pub mapping_id: String,
    pub allocation_id: String,
    pub protocol: Protocol,
    pub local_ip: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub gateway_public_ip: String,
    pub gateway_control_port: u16,
    pub gateway_token: String,
    pub fqdn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDesiredStateResponse {
    pub node_id: String,
    pub gateway: Gateway,
    pub proxies: Vec<AgentDesiredProxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningProxyReport {
    pub proxy_name: String,
    pub status: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReportStateRequest {
    pub node_id: String,
    pub frpc_running: bool,
    pub running_proxies: Vec<RunningProxyReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMappingRequest {
    pub allocation_id: String,
    pub node_id: String,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub local_ip: String,
    pub local_port: u16,
    pub protocol: Protocol,
    pub custom_alias: Option<String>,
    pub gateway_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileResult {
    pub created_mappings: usize,
    pub removed_mappings: usize,
    pub updated_mappings: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheckItem {
    pub name: String,
    pub passed: bool,
    pub details: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub overall_status: bool,
    pub checks: Vec<DoctorCheckItem>,
}
