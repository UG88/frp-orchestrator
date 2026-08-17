use anyhow::Result;
use frp_shared::config::GatewayConfig;
use tracing::info;

pub struct FirewallManager;

impl FirewallManager {
    /// Generate UFW firewall commands for the gateway.
    pub fn generate_ufw_commands(config: &GatewayConfig) -> Vec<String> {
        vec![
            format!("ufw allow {}/tcp comment 'FRP Control Port'", config.control_port),
            format!(
                "ufw allow {}:{}/tcp comment 'Minecraft FRP TCP Ports'",
                config.tcp_port_range.start, config.tcp_port_range.end
            ),
            format!(
                "ufw allow {}:{}/udp comment 'Minecraft FRP UDP Ports'",
                config.udp_port_range.start, config.udp_port_range.end
            ),
        ]
    }

    /// Generate nftables rules for the gateway.
    pub fn generate_nftables_rules(config: &GatewayConfig) -> String {
        format!(
            r#"# nftables configuration for FRP Gateway
table inet frp_filter {{
    chain input {{
        type filter hook input priority 0; policy accept;
        
        # FRP Control Traffic
        tcp dport {control_port} accept comment "FRP Control Port"
        
        # Minecraft Port Ranges
        tcp dport {tcp_start}-{tcp_end} accept comment "Minecraft TCP"
        udp dport {udp_start}-{udp_end} accept comment "Minecraft UDP"
    }}
}}
"#,
            control_port = config.control_port,
            tcp_start = config.tcp_port_range.start,
            tcp_end = config.tcp_port_range.end,
            udp_start = config.udp_port_range.start,
            udp_end = config.udp_port_range.end,
        )
    }

    /// Apply firewall configuration on Linux systems if UFW is available.
    pub async fn apply_ufw(config: &GatewayConfig) -> Result<()> {
        let commands = Self::generate_ufw_commands(config);
        for cmd_str in commands {
            info!(command = %cmd_str, "Executing firewall configuration");
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if let Some((prog, args)) = parts.split_first() {
                let _ = tokio::process::Command::new(prog)
                    .args(args)
                    .status()
                    .await;
            }
        }
        Ok(())
    }
}
