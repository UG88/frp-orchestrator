use anyhow::Result;
use frp_shared::config::{AgentConfig, ControllerConfig, GatewayConfig, PortRangeConfig, PterodactylConfig};
use std::io::{self, Write};
use std::path::Path;

pub struct ConfigWizard;

impl ConfigWizard {
    fn prompt(question: &str, default: &str) -> String {
        print!("{} [{}]: ", question, default);
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                default.to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            default.to_string()
        }
    }

    fn prompt_secret(question: &str, default: &str) -> String {
        print!("{} [{}]: ", question, default);
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                default.to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            default.to_string()
        }
    }

    /// Interactive setup for a Gateway Server
    pub fn setup_gateway(target_path: impl AsRef<Path>) -> Result<()> {
        println!("============================================================");
        println!("     FRP Gateway Interactive Configuration Wizard           ");
        println!("============================================================");

        let id = Self::prompt("Enter Gateway ID", "gw-sg-01");
        let region = Self::prompt("Enter Gateway Region (e.g. singapore, europe, us)", "singapore");
        let public_ip = Self::prompt("Enter Gateway Public IP", "198.51.100.10");
        let control_port: u16 = Self::prompt("Enter FRP Control Port", "7000").parse().unwrap_or(7000);
        let tcp_start: u16 = Self::prompt("Enter Minecraft TCP Start Port", "30000").parse().unwrap_or(30000);
        let tcp_end: u16 = Self::prompt("Enter Minecraft TCP End Port", "40000").parse().unwrap_or(40000);
        let udp_start: u16 = Self::prompt("Enter Minecraft UDP Start Port", "30000").parse().unwrap_or(30000);
        let udp_end: u16 = Self::prompt("Enter Minecraft UDP End Port", "40000").parse().unwrap_or(40000);
        let token = Self::prompt_secret("Enter FRP Gateway Secret Token", "generate-a-strong-token-here");

        let config = GatewayConfig {
            id,
            region,
            public_ip,
            control_port,
            tcp_port_range: PortRangeConfig {
                start: tcp_start,
                end: tcp_end,
            },
            udp_port_range: PortRangeConfig {
                start: udp_start,
                end: udp_end,
            },
            reserved_ports: vec![30000, 30001],
            token,
            tls_enable: true,
            dashboard_port: Some(7500),
        };

        let toml_str = toml::to_string_pretty(&config)?;
        std::fs::write(target_path.as_ref(), toml_str)?;

        println!("------------------------------------------------------------");
        println!(" [✓] Gateway configuration saved to: {}", target_path.as_ref().display());
        println!("============================================================");
        Ok(())
    }

    /// Interactive setup for a Pterodactyl Node Agent
    pub fn setup_agent(target_path: impl AsRef<Path>) -> Result<()> {
        println!("============================================================");
        println!("     FRP Node Agent Interactive Configuration Wizard        ");
        println!("============================================================");

        let agent_id = Self::prompt("Enter Agent ID", "agent-sg-node-01");
        let controller_url = Self::prompt("Enter FRP Controller URL", "https://controller.internal.example.com");
        let agent_token = Self::prompt_secret("Enter Agent Token", "ENV:FRP_AGENT_TOKEN");
        let ptero_node_id: u64 = Self::prompt("Enter Pterodactyl Node ID (from Panel)", "1").parse().unwrap_or(1);
        let admin_pass = Self::prompt_secret("Enter local FRP client admin password", "admin");

        let config = AgentConfig {
            agent_id,
            controller_url,
            agent_token,
            pterodactyl_node_id: ptero_node_id,
            frpc_binary_path: "/opt/frp/frpc".to_string(),
            frpc_config_dir: "/opt/frp/conf.d".to_string(),
            frpc_main_config: "/opt/frp/frpc.toml".to_string(),
            frpc_admin_addr: "127.0.0.1:7400".to_string(),
            frpc_admin_user: "admin".to_string(),
            frpc_admin_password: admin_pass,
            heartbeat_interval_secs: 15,
        };

        let toml_str = toml::to_string_pretty(&config)?;
        std::fs::write(target_path.as_ref(), toml_str)?;

        println!("------------------------------------------------------------");
        println!(" [✓] Agent configuration saved to: {}", target_path.as_ref().display());
        println!("============================================================");
        Ok(())
    }

    /// Interactive setup for the Central Controller
    pub fn setup_controller(target_path: impl AsRef<Path>) -> Result<()> {
        println!("============================================================");
        println!("     FRP Controller Interactive Configuration Wizard        ");
        println!("============================================================");

        let listen_addr = Self::prompt("Enter Controller Listen Address", "0.0.0.0:8080");
        let api_key = Self::prompt_secret("Enter Controller Master API Key", "ENV:CONTROLLER_API_KEY");
        let ptero_url = Self::prompt("Enter Pterodactyl Panel URL", "https://panel.example.com");
        let ptero_api_key = Self::prompt_secret("Enter Pterodactyl Application API Key (ptla_...)", "ENV:PTERODACTYL_API_KEY");
        let domain = Self::prompt("Enter Default Player Connection Domain (e.g. mc.example.com)", "mc.example.com");

        let config = ControllerConfig {
            listen_addr,
            api_key,
            database_path: "/var/lib/frp-orchestrator/controller.db".to_string(),
            reconciliation_interval_secs: 30,
            pterodactyl: PterodactylConfig {
                url: ptero_url,
                api_key: ptero_api_key,
                sync_interval_secs: 30,
                auto_expose_default: true,
                egg_protocol_overrides: std::collections::HashMap::new(),
            },
            dns: Some(frp_shared::config::DnsConfig {
                default_domain: Some(domain),
                cloudflare: Some(frp_shared::config::CloudflareConfig {
                    enabled: false,
                    api_token: "ENV:CLOUDFLARE_API_TOKEN".to_string(),
                    zone_id: "your_zone_id_here".to_string(),
                    proxied: false,
                    ttl: 1,
                }),
                route53: None,
            }),
            gateways: vec![GatewayConfig {
                id: "gw-sg-01".to_string(),
                region: "singapore".to_string(),
                public_ip: "3.108.50.20".to_string(),
                control_port: 7000,
                tcp_port_range: PortRangeConfig {
                    start: 30000,
                    end: 40000,
                },
                udp_port_range: PortRangeConfig {
                    start: 30000,
                    end: 40000,
                },
                reserved_ports: vec![30000, 30001],
                token: "ENV:FRP_GATEWAY_TOKEN".to_string(),
                tls_enable: true,
                dashboard_port: Some(7500),
            }],
        };

        let toml_str = toml::to_string_pretty(&config)?;
        std::fs::write(target_path.as_ref(), toml_str)?;

        println!("------------------------------------------------------------");
        println!(" [✓] Controller configuration saved to: {}", target_path.as_ref().display());
        println!("============================================================");
        Ok(())
    }
}
