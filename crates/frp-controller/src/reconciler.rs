use crate::allocation_manager::AllocationManager;
use crate::db::Database;
use crate::pterodactyl_client::PterodactylClient;
use chrono::Utc;
use frp_shared::api_types::ReconcileResult;
use frp_shared::models::{Allocation, AllocationStatus, Protocol};
use std::collections::{HashMap, HashSet};
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct Reconciler {
    db: Database,
    ptero_client: PterodactylClient,
    allocation_mgr: AllocationManager,
}

impl Reconciler {
    pub fn new(
        db: Database,
        ptero_client: PterodactylClient,
        allocation_mgr: AllocationManager,
    ) -> Self {
        Self {
            db,
            ptero_client,
            allocation_mgr,
        }
    }

    /// Perform a full reconciliation cycle across all nodes and allocations.
    pub async fn reconcile_all(&self) -> ReconcileResult {
        let mut result = ReconcileResult {
            created_mappings: 0,
            removed_mappings: 0,
            updated_mappings: 0,
            errors: Vec::new(),
        };

        let nodes = match self.db.list_nodes() {
            Ok(n) => n,
            Err(e) => {
                let err_msg = format!("Failed to list nodes for reconciliation: {}", e);
                error!(%err_msg);
                result.errors.push(err_msg);
                return result;
            }
        };

        // Fetch Pterodactyl servers once to map egg/metadata
        let servers_by_alloc: HashMap<u64, (String, String, Option<String>)> = match self.ptero_client.get_servers().await {
            Ok(servers) => servers
                .into_iter()
                .map(|s| (s.allocation, (s.name, format!("egg-{}", s.egg), s.docker_image)))
                .collect(),
            Err(e) => {
                warn!(error = %e, "Could not fetch servers from Pterodactyl API, proceeding with basic allocation data");
                HashMap::new()
            }
        };

        for node in nodes {
            info!(node_id = %node.id, ptero_node_id = node.pterodactyl_node_id, "Reconciling node allocations");

            let ptero_allocs = match self.ptero_client.get_node_allocations(node.pterodactyl_node_id).await {
                Ok(allocs) => allocs,
                Err(e) => {
                    let err = format!("Failed to query Pterodactyl allocations for node {}: {}", node.id, e);
                    warn!(%err);
                    result.errors.push(err);
                    continue;
                }
            };

            let mut seen_ptero_ids = HashSet::new();

            for p_alloc in ptero_allocs {
                seen_ptero_ids.insert(p_alloc.id);

                // Only process allocations assigned to a server
                if !p_alloc.assigned {
                    continue;
                }

                let server_info = servers_by_alloc.get(&p_alloc.id);
                let server_name = server_info.map(|s| s.0.clone());
                let egg_name = server_info.map(|s| s.1.as_str());
                let docker_image = server_info.and_then(|s| s.2.as_deref());

                // Upsert local allocation in DB
                let alloc_id = format!("{}-{}", node.id, p_alloc.id);
                let alloc = Allocation {
                    id: alloc_id.clone(),
                    node_id: node.id.clone(),
                    server_id: p_alloc.server_id.map(|id| id.to_string()),
                    server_name,
                    pterodactyl_allocation_id: p_alloc.id,
                    local_ip: p_alloc.ip.clone(),
                    local_port: p_alloc.port,
                    protocol: Protocol::Auto,
                    custom_alias: p_alloc.alias,
                    status: AllocationStatus::Pending,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                if let Err(e) = self.db.upsert_allocation(&alloc) {
                    let err = format!("Failed to upsert allocation {}: {}", alloc.id, e);
                    error!(%err);
                    result.errors.push(err);
                    continue;
                }

                // Check if mapping exists
                match self.allocation_mgr.provision_mapping(&node, &alloc, egg_name, docker_image).await {
                    Ok(_) => {
                        result.created_mappings += 1;
                    }
                    Err(e) => {
                        let err = format!("Failed to provision mapping for allocation {}: {}", alloc.id, e);
                        error!(%err);
                        result.errors.push(err);
                    }
                }
            }

            // Clean up mappings for allocations that no longer exist on this node in Pterodactyl
            if let Ok(db_allocs) = self.db.list_allocations() {
                for db_alloc in db_allocs {
                    if db_alloc.node_id == node.id && !seen_ptero_ids.contains(&db_alloc.pterodactyl_allocation_id) {
                        info!(
                            allocation_id = %db_alloc.id,
                            ptero_id = db_alloc.pterodactyl_allocation_id,
                            "Allocation removed from Pterodactyl, tearing down mapping"
                        );

                        if let Err(e) = self.allocation_mgr.delete_mapping(&db_alloc.id).await {
                            let err = format!("Failed to delete mapping for obsolete allocation {}: {}", db_alloc.id, e);
                            error!(%err);
                            result.errors.push(err);
                        } else {
                            result.removed_mappings += 1;
                        }

                        let _ = self.db.delete_allocation(&db_alloc.id);
                    }
                }
            }
        }

        info!(
            created = result.created_mappings,
            removed = result.removed_mappings,
            errors = result.errors.len(),
            "Reconciliation cycle completed"
        );

        result
    }
}
