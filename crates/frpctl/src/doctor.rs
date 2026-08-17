use crate::http_client::ControllerCliClient;
use frp_shared::api_types::{DoctorCheckItem, DoctorReport, HealthResponse};
use std::env;
use std::path::Path;

pub struct DoctorEngine {
    client: ControllerCliClient,
    #[allow(dead_code)]
    config_path: Option<String>,
}

impl DoctorEngine {
    pub fn new(client: ControllerCliClient, config_path: Option<String>) -> Self {
        Self { client, config_path }
    }

    pub async fn run_checks(&self) -> DoctorReport {
        let mut checks = Vec::new();

        // 1. OS & Architecture Check
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        checks.push(DoctorCheckItem {
            name: "Operating System & Architecture".to_string(),
            passed: true,
            details: format!("OS: {}, Arch: {}", os, arch),
            fix_hint: None,
        });

        // 2. FRP Binary Check
        let frp_paths = [
            "/opt/frp/frpc",
            "/opt/frp/frps",
            "/usr/local/bin/frpc",
            "/usr/local/bin/frps",
            "frpc.exe",
            "frps.exe",
        ];
        let mut found_frp = false;
        let mut found_path = String::new();
        for p in &frp_paths {
            if Path::new(p).exists() {
                found_frp = true;
                found_path = p.to_string();
                break;
            }
        }
        checks.push(DoctorCheckItem {
            name: "FRP Binary Check".to_string(),
            passed: found_frp,
            details: if found_frp {
                format!("Found FRP executable at {}", found_path)
            } else {
                "FRP binary (frpc/frps) not found in standard paths (/opt/frp/)".to_string()
            },
            fix_hint: if found_frp {
                None
            } else {
                Some("Run 'frpctl install gateway' or 'frpctl install agent' to install FRP".to_string())
            },
        });

        // 3. Controller API Connectivity Check
        match self.client.get::<HealthResponse>("/health").await {
            Ok(health) => {
                checks.push(DoctorCheckItem {
                    name: "Controller API Connectivity".to_string(),
                    passed: true,
                    details: format!(
                        "Connected to Controller v{} (Uptime: {}s, DB: OK)",
                        health.version, health.uptime_secs
                    ),
                    fix_hint: None,
                });

                // Gateways status
                checks.push(DoctorCheckItem {
                    name: "Gateway Cluster Status".to_string(),
                    passed: health.healthy_gateways > 0,
                    details: format!(
                        "Gateways online: {}/{} healthy",
                        health.healthy_gateways, health.total_gateways
                    ),
                    fix_hint: if health.healthy_gateways == 0 {
                        Some("Register and start at least one FRP gateway via 'frpctl gateway create'".to_string())
                    } else {
                        None
                    },
                });

                // Nodes status
                checks.push(DoctorCheckItem {
                    name: "Pterodactyl Nodes Status".to_string(),
                    passed: true,
                    details: format!(
                        "Nodes connected: {}/{} healthy (Active mappings: {})",
                        health.healthy_nodes, health.total_nodes, health.active_mappings
                    ),
                    fix_hint: None,
                });
            }
            Err(e) => {
                checks.push(DoctorCheckItem {
                    name: "Controller API Connectivity".to_string(),
                    passed: false,
                    details: format!("Could not reach Controller API: {}", e),
                    fix_hint: Some("Verify controller service is running and CONTROLLER_URL / CONTROLLER_API_KEY are configured".to_string()),
                });
            }
        }

        let overall_status = checks.iter().all(|c| c.passed);
        DoctorReport {
            overall_status,
            checks,
        }
    }
}
