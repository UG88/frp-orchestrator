use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Supported network protocols for Minecraft server traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    Both,
    Auto,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
            Protocol::Both => write!(f, "both"),
            Protocol::Auto => write!(f, "auto"),
        }
    }
}

impl Protocol {
    pub fn is_tcp(&self) -> bool {
        matches!(self, Protocol::Tcp | Protocol::Both)
    }

    pub fn is_udp(&self) -> bool {
        matches!(self, Protocol::Udp | Protocol::Both)
    }
}

/// Lifecycle status of an allocation mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllocationStatus {
    Pending,
    Active,
    Error,
    Orphaned,
    Deleted,
}

impl fmt::Display for AllocationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationStatus::Pending => write!(f, "pending"),
            AllocationStatus::Active => write!(f, "active"),
            AllocationStatus::Error => write!(f, "error"),
            AllocationStatus::Orphaned => write!(f, "orphaned"),
            AllocationStatus::Deleted => write!(f, "deleted"),
        }
    }
}

/// Gateway server definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub region: String,
    pub public_ip: String,
    pub control_port: u16,
    pub tcp_port_range_start: u16,
    pub tcp_port_range_end: u16,
    pub udp_port_range_start: u16,
    pub udp_port_range_end: u16,
    pub reserved_ports: Vec<u16>,
    pub is_healthy: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub token: String,
    pub created_at: DateTime<Utc>,
}

/// Pterodactyl node registered with the controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub pterodactyl_node_id: u64,
    pub assigned_gateway_id: String,
    pub local_ip: String,
    pub is_healthy: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub agent_token: String,
    pub created_at: DateTime<Utc>,
}

/// Desired allocation on a Pterodactyl node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: String,
    pub node_id: String,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub pterodactyl_allocation_id: u64,
    pub local_ip: String,
    pub local_port: u16,
    pub protocol: Protocol,
    pub custom_alias: Option<String>,
    pub status: AllocationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Active FRP proxy mapping between Gateway public port and Node local port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub id: String,
    pub allocation_id: String,
    pub gateway_id: String,
    pub protocol: Protocol,
    pub gateway_port: u16,
    pub target_ip: String,
    pub target_port: u16,
    pub fqdn: Option<String>,
    pub is_active: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Managed DNS record pointing to a Gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub mapping_id: String,
    pub fqdn: String,
    pub target_ip: String,
    pub provider: String,
    pub is_synced: bool,
    pub created_at: DateTime<Utc>,
}

/// Audit log record for tracking state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub event_type: String,
    pub resource_id: String,
    pub details: String,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(event_type: impl Into<String>, resource_id: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            resource_id: resource_id.into(),
            details: details.into(),
            created_at: Utc::now(),
        }
    }
}

/// Port pool statistics for a gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPoolStatus {
    pub gateway_id: String,
    pub tcp_total: u32,
    pub tcp_allocated: u32,
    pub tcp_available: u32,
    pub udp_total: u32,
    pub udp_allocated: u32,
    pub udp_available: u32,
}
