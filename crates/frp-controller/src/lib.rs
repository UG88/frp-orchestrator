pub mod allocation_manager;
pub mod api;
pub mod db;
pub mod port_manager;
pub mod protocol_detector;
pub mod pterodactyl_client;
pub mod reconciler;

#[cfg(test)]
mod tests {
    use super::*;
    use db::Database;
    use frp_shared::dns::MockDnsProvider;
    use frp_shared::models::{Allocation, AllocationStatus, Gateway, Node, Protocol};
    use port_manager::PortManager;
    use protocol_detector::ProtocolDetector;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_port_manager_and_allocation_lifecycle() {
        let db = Database::open_in_memory().unwrap();
        let port_mgr = PortManager::new(db.clone());

        let gw = Gateway {
            id: "gw-test".to_string(),
            region: "test-region".to_string(),
            public_ip: "198.51.100.1".to_string(),
            control_port: 7000,
            tcp_port_range_start: 30000,
            tcp_port_range_end: 30005,
            udp_port_range_start: 30000,
            udp_port_range_end: 30005,
            reserved_ports: vec![30000],
            is_healthy: true,
            last_heartbeat: None,
            token: "test-token".to_string(),
            created_at: chrono::Utc::now(),
        };
        db.upsert_gateway(&gw).unwrap();

        // 30000 is reserved, so lowest available is 30001
        let port1 = port_mgr.allocate_port(&gw, Protocol::Tcp).unwrap();
        assert_eq!(port1, 30001);

        let node = Node {
            id: "node-1".to_string(),
            name: "Node 1".to_string(),
            pterodactyl_node_id: 1,
            assigned_gateway_id: "gw-test".to_string(),
            local_ip: "10.0.0.5".to_string(),
            is_healthy: true,
            last_heartbeat: None,
            agent_token: "agent-tok".to_string(),
            created_at: chrono::Utc::now(),
        };
        db.upsert_node(&node).unwrap();

        let alloc = Allocation {
            id: "alloc-1".to_string(),
            node_id: "node-1".to_string(),
            server_id: Some("srv-123".to_string()),
            server_name: Some("Minecraft Survival".to_string()),
            pterodactyl_allocation_id: 101,
            local_ip: "10.0.0.5".to_string(),
            local_port: 25565,
            protocol: Protocol::Tcp,
            custom_alias: None,
            status: AllocationStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        db.upsert_allocation(&alloc).unwrap();

        let proto_det = Arc::new(ProtocolDetector::new(HashMap::new()));
        let mock_dns = Arc::new(MockDnsProvider::default());
        let alloc_mgr = allocation_manager::AllocationManager::new(
            db.clone(),
            port_mgr.clone(),
            proto_det,
            mock_dns.clone(),
            Some("example.com".to_string()),
        );

        let mapping = alloc_mgr.provision_mapping(&node, &alloc, None, None).await.unwrap();
        assert_eq!(mapping.gateway_port, 30001);
        assert_eq!(mapping.fqdn, Some("srv-123.example.com".to_string()));

        // Verify DNS record was mocked
        let dns_records = mock_dns.records.lock().unwrap();
        assert_eq!(dns_records.get("srv-123.example.com").unwrap(), "198.51.100.1");
        drop(dns_records);

        // Next allocation should get 30002
        let port2 = port_mgr.allocate_port(&gw, Protocol::Tcp).unwrap();
        assert_eq!(port2, 30002);

        // Check port pool stats
        let pool = port_mgr.get_pool_status(&gw).unwrap();
        assert_eq!(pool.tcp_total, 6);
        assert_eq!(pool.tcp_allocated, 1); // Only 1 mapping is inserted in DB
        assert_eq!(pool.tcp_available, 4); // 6 total - 1 reserved - 1 allocated = 4
    }
}
