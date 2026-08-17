use crate::db::Database;
use crate::port_manager::PortManager;
use crate::protocol_detector::ProtocolDetector;
use anyhow::{Context, Result};
use chrono::Utc;
use frp_shared::dns::DnsProvider;
use frp_shared::models::{Allocation, AllocationStatus, AuditLog, Mapping, Node};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct AllocationManager {
    db: Database,
    port_mgr: PortManager,
    protocol_detector: Arc<ProtocolDetector>,
    dns_provider: Arc<dyn DnsProvider>,
    default_domain: Option<String>,
}

impl AllocationManager {
    pub fn new(
        db: Database,
        port_mgr: PortManager,
        protocol_detector: Arc<ProtocolDetector>,
        dns_provider: Arc<dyn DnsProvider>,
        default_domain: Option<String>,
    ) -> Self {
        Self {
            db,
            port_mgr,
            protocol_detector,
            dns_provider,
            default_domain,
        }
    }

    /// Provision or refresh a mapping for an allocation.
    pub async fn provision_mapping(
        &self,
        node: &Node,
        alloc: &Allocation,
        egg_name: Option<&str>,
        docker_image: Option<&str>,
    ) -> Result<Mapping> {
        // If an active mapping already exists, return it
        if let Some(existing) = self.db.get_mapping_by_allocation(&alloc.id)? {
            if existing.is_active {
                return Ok(existing);
            }
        }

        // Get the node's assigned gateway
        let gateway = self
            .db
            .get_gateway(&node.assigned_gateway_id)?
            .context(format!("Assigned gateway {} not found", node.assigned_gateway_id))?;

        if !gateway.is_healthy {
            warn!(gateway = %gateway.id, "Provisioning mapping on gateway marked unhealthy");
        }

        // Detect protocol
        let protocol = self.protocol_detector.detect(
            alloc.protocol,
            alloc.server_name.as_deref(),
            egg_name,
            docker_image,
        );

        // Allocate gateway port
        let gateway_port = self.port_mgr.allocate_port(&gateway, protocol)?;

        // Generate FQDN if domain is configured
        let fqdn = if let Some(domain) = &self.default_domain {
            let prefix = alloc
                .custom_alias
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    alloc
                        .server_id
                        .as_deref()
                        .map(|id| {
                            if id.starts_with("srv-") {
                                id.to_string()
                            } else {
                                format!("srv-{}", id)
                            }
                        })
                        .unwrap_or_else(|| format!("mc-{}", alloc.pterodactyl_allocation_id))
                });
            Some(format!("{}.{}", prefix, domain))
        } else {
            None
        };

        // Create DNS record if FQDN is enabled
        if let Some(ref host) = fqdn {
            if let Err(e) = self.dns_provider.create_or_update_record(host, &gateway.public_ip).await {
                warn!(fqdn = %host, error = %e, "Failed to register DNS record");
            }
        }

        let mapping = Mapping {
            id: Uuid::new_v4().to_string(),
            allocation_id: alloc.id.clone(),
            gateway_id: gateway.id.clone(),
            protocol,
            gateway_port,
            target_ip: alloc.local_ip.clone(),
            target_port: alloc.local_port,
            fqdn,
            is_active: true,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db.insert_mapping(&mapping)?;

        // Update allocation status to Active
        let mut updated_alloc = alloc.clone();
        updated_alloc.status = AllocationStatus::Active;
        updated_alloc.updated_at = Utc::now();
        self.db.upsert_allocation(&updated_alloc)?;

        self.db.add_audit_log(&AuditLog::new(
            "MAPPING_CREATED",
            &mapping.id,
            format!(
                "Created mapping for server '{:?}' on gateway {} (port {})",
                alloc.server_name, gateway.id, gateway_port
            ),
        ))?;

        info!(
            mapping_id = %mapping.id,
            gateway = %gateway.id,
            gateway_port = gateway_port,
            local_target = %format!("{}:{}", alloc.local_ip, alloc.local_port),
            protocol = %protocol,
            "FRP mapping successfully provisioned"
        );

        Ok(mapping)
    }

    /// Delete a mapping and release associated resources (ports, DNS).
    pub async fn delete_mapping(&self, allocation_id: &str) -> Result<()> {
        if let Some(mapping) = self.db.get_mapping_by_allocation(allocation_id)? {
            // Remove DNS record if managed
            if let Some(ref fqdn) = mapping.fqdn {
                if let Err(e) = self.dns_provider.delete_record(fqdn).await {
                    warn!(fqdn = %fqdn, error = %e, "Failed to delete DNS record during mapping teardown");
                }
            }

            self.db.delete_mapping(&mapping.id)?;

            self.db.add_audit_log(&AuditLog::new(
                "MAPPING_DELETED",
                &mapping.id,
                format!(
                    "Deleted mapping on gateway {} (port {})",
                    mapping.gateway_id, mapping.gateway_port
                ),
            ))?;

            info!(
                mapping_id = %mapping.id,
                gateway = %mapping.gateway_id,
                port = mapping.gateway_port,
                "FRP mapping and port released"
            );
        }

        Ok(())
    }
}
