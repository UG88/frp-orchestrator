pub mod firewall;
pub mod frps_config;
pub mod health;

#[cfg(test)]
mod tests {
    use super::*;
    use frp_shared::config::{GatewayConfig, PortRangeConfig};

    #[test]
    fn test_frps_config_rendering() {
        let gw = GatewayConfig {
            id: "gw-sg-01".to_string(),
            region: "singapore".to_string(),
            public_ip: "198.51.100.10".to_string(),
            control_port: 7000,
            tcp_port_range: PortRangeConfig {
                start: 30000,
                end: 40000,
            },
            udp_port_range: PortRangeConfig {
                start: 30000,
                end: 40000,
            },
            reserved_ports: vec![30000],
            token: "test-token-xyz".to_string(),
            tls_enable: true,
            dashboard_port: Some(7500),
        };

        let toml = frps_config::FrpsConfigGenerator::render_toml(&gw);
        assert!(toml.contains("bindPort = 7000"));
        assert!(toml.contains("auth.token = \"test-token-xyz\""));
        assert!(toml.contains("start = 30000, end = 40000"));
        assert!(toml.contains("webServer.port = 7500"));
    }

    #[test]
    fn test_firewall_command_generation() {
        let gw = GatewayConfig {
            id: "gw-in-01".to_string(),
            region: "india".to_string(),
            public_ip: "203.0.113.5".to_string(),
            control_port: 7000,
            tcp_port_range: PortRangeConfig {
                start: 31000,
                end: 32000,
            },
            udp_port_range: PortRangeConfig {
                start: 31000,
                end: 32000,
            },
            reserved_ports: vec![],
            token: "tok".to_string(),
            tls_enable: true,
            dashboard_port: None,
        };

        let ufw = firewall::FirewallManager::generate_ufw_commands(&gw);
        assert_eq!(ufw.len(), 3);
        assert_eq!(ufw[0], "ufw allow 7000/tcp comment 'FRP Control Port'");
        assert_eq!(ufw[1], "ufw allow 31000:32000/tcp comment 'Minecraft FRP TCP Ports'");
        assert_eq!(ufw[2], "ufw allow 31000:32000/udp comment 'Minecraft FRP UDP Ports'");
    }
}
