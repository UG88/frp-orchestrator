pub mod controller_client;
pub mod frpc_manager;
pub mod runner;

#[cfg(test)]
mod tests {
    use super::*;
    use frp_shared::api_types::AgentDesiredProxy;
    use frp_shared::config::AgentConfig;
    use frp_shared::models::{Gateway, Protocol};

    #[tokio::test]
    async fn test_proxy_rendering_and_sync() {
        let temp_dir = std::env::temp_dir().join(format!("frp-agent-test-{}", uuid::Uuid::new_v4()));
        let conf_d = temp_dir.join("conf.d");
        let main_conf = temp_dir.join("frpc.toml");

        let cfg = AgentConfig {
            agent_id: "test-agent".to_string(),
            controller_url: "http://127.0.0.1:8080".to_string(),
            agent_token: "tok".to_string(),
            pterodactyl_node_id: 1,
            frpc_binary_path: "/nonexistent/frpc".to_string(),
            frpc_config_dir: conf_d.display().to_string(),
            frpc_main_config: main_conf.display().to_string(),
            frpc_admin_addr: "127.0.0.1:7400".to_string(),
            frpc_admin_user: "admin".to_string(),
            frpc_admin_password: "admin".to_string(),
            heartbeat_interval_secs: 10,
        };

        let mgr = frpc_manager::FrpcManager::new(&cfg);
        mgr.init_directories().unwrap();

        let gw = Gateway {
            id: "gw-sg-01".to_string(),
            region: "singapore".to_string(),
            public_ip: "198.51.100.10".to_string(),
            control_port: 7000,
            tcp_port_range_start: 30000,
            tcp_port_range_end: 40000,
            udp_port_range_start: 30000,
            udp_port_range_end: 40000,
            reserved_ports: vec![],
            is_healthy: true,
            last_heartbeat: None,
            token: "secret-token".to_string(),
            created_at: chrono::Utc::now(),
        };

        mgr.write_main_config(&gw).unwrap();
        assert!(main_conf.exists());
        let main_content = std::fs::read_to_string(&main_conf).unwrap();
        assert!(main_content.contains("serverAddr = \"198.51.100.10\""));

        let proxies = vec![
            AgentDesiredProxy {
                proxy_name: "mc_server_1".to_string(),
                mapping_id: "m-1".to_string(),
                allocation_id: "a-1".to_string(),
                protocol: Protocol::Tcp,
                local_ip: "10.0.0.2".to_string(),
                local_port: 25565,
                remote_port: 30001,
                gateway_public_ip: "198.51.100.10".to_string(),
                gateway_control_port: 7000,
                gateway_token: "secret".to_string(),
                fqdn: None,
            },
            AgentDesiredProxy {
                proxy_name: "mc_server_2_geyser".to_string(),
                mapping_id: "m-2".to_string(),
                allocation_id: "a-2".to_string(),
                protocol: Protocol::Both,
                local_ip: "10.0.0.3".to_string(),
                local_port: 25566,
                remote_port: 30002,
                gateway_public_ip: "198.51.100.10".to_string(),
                gateway_control_port: 7000,
                gateway_token: "secret".to_string(),
                fqdn: None,
            },
        ];

        // Sync without running frpc binary (catch reload error gracefully)
        let _ = mgr.sync_proxies(&proxies).await;

        let p1_file = conf_d.join("mc_server_1.toml");
        let p2_file = conf_d.join("mc_server_2_geyser.toml");

        assert!(p1_file.exists());
        assert!(p2_file.exists());

        let p1_text = std::fs::read_to_string(p1_file).unwrap();
        assert!(p1_text.contains("type = \"tcp\""));
        assert!(p1_text.contains("localPort = 25565"));
        assert!(p1_text.contains("remotePort = 30001"));

        let p2_text = std::fs::read_to_string(p2_file).unwrap();
        assert!(p2_text.contains("type = \"tcp\""));
        assert!(p2_text.contains("type = \"udp\""));
        assert!(p2_text.contains("remotePort = 30002"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
