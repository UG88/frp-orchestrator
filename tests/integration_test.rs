use frp_controller::allocation_manager::AllocationManager;
use frp_controller::db::Database;
use frp_controller::port_manager::PortManager;
use frp_controller::protocol_detector::ProtocolDetector;
use frp_shared::dns::MockDnsProvider;
use frp_shared::models::{Allocation, AllocationStatus, Gateway, Node, Protocol};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_multi_server_allocation_and_non_interruption() {
    let db = Database::open_in_memory().unwrap();
    let port_mgr = PortManager::new(db.clone());

    let gw = Gateway {
        id: "gw-sg-main".to_string(),
        region: "singapore".to_string(),
        public_ip: "3.108.50.20".to_string(),
        control_port: 7000,
        tcp_port_range_start: 30000,
        tcp_port_range_end: 30050,
        udp_port_range_start: 30000,
        udp_port_range_end: 30050,
        reserved_ports: vec![30000],
        is_healthy: true,
        last_heartbeat: None,
        token: "gw-token-xyz".to_string(),
        created_at: chrono::Utc::now(),
    };
    db.upsert_gateway(&gw).unwrap();

    let node = Node {
        id: "node-sg-01".to_string(),
        name: "Singapore Node 01".to_string(),
        pterodactyl_node_id: 10,
        assigned_gateway_id: "gw-sg-main".to_string(),
        local_ip: "10.10.0.20".to_string(),
        is_healthy: true,
        last_heartbeat: None,
        agent_token: "agent-tok-123".to_string(),
        created_at: chrono::Utc::now(),
    };
    db.upsert_node(&node).unwrap();

    let dns = Arc::new(MockDnsProvider::default());
    let proto_detector = Arc::new(ProtocolDetector::new(HashMap::new()));
    let alloc_mgr = AllocationManager::new(
        db.clone(),
        port_mgr.clone(),
        proto_detector,
        dns.clone(),
        Some("play.example.com".to_string()),
    );

    // 1. Connect Server A (Java Minecraft - TCP only)
    let alloc_a = Allocation {
        id: "alloc-srv-a".to_string(),
        node_id: "node-sg-01".to_string(),
        server_id: Some("srv-a".to_string()),
        server_name: Some("Survival Java".to_string()),
        pterodactyl_allocation_id: 1001,
        local_ip: "10.10.0.20".to_string(),
        local_port: 25565,
        protocol: Protocol::Tcp,
        custom_alias: Some("survival".to_string()),
        status: AllocationStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.upsert_allocation(&alloc_a).unwrap();

    let map_a = alloc_mgr.provision_mapping(&node, &alloc_a, None, None).await.unwrap();
    assert_eq!(map_a.gateway_port, 30001);
    assert_eq!(map_a.protocol, Protocol::Tcp);
    assert_eq!(map_a.fqdn, Some("survival.play.example.com".to_string()));

    // 2. Connect Server B (Geyser Crossplay - Both TCP + UDP on same port)
    let alloc_b = Allocation {
        id: "alloc-srv-b".to_string(),
        node_id: "node-sg-01".to_string(),
        server_id: Some("srv-b".to_string()),
        server_name: Some("Geyser Crossplay Server".to_string()),
        pterodactyl_allocation_id: 1002,
        local_ip: "10.10.0.20".to_string(),
        local_port: 25567,
        protocol: Protocol::Auto, // Auto detection should detect Geyser -> Both
        custom_alias: Some("crossplay".to_string()),
        status: AllocationStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.upsert_allocation(&alloc_b).unwrap();

    let map_b = alloc_mgr.provision_mapping(&node, &alloc_b, Some("geyser-egg"), None).await.unwrap();
    assert_eq!(map_b.gateway_port, 30002);
    assert_eq!(map_b.protocol, Protocol::Both);
    assert_eq!(map_b.fqdn, Some("crossplay.play.example.com".to_string()));

    // Verify Server A is still intact and unaffected
    let existing_map_a = db.get_mapping_by_allocation("alloc-srv-a").unwrap().unwrap();
    assert_eq!(existing_map_a.gateway_port, 30001);
    assert!(existing_map_a.is_active);

    // 3. Connect Server C (Bedrock standalone - UDP only)
    let alloc_c = Allocation {
        id: "alloc-srv-c".to_string(),
        node_id: "node-sg-01".to_string(),
        server_id: Some("srv-c".to_string()),
        server_name: Some("Bedrock Pocket Server".to_string()),
        pterodactyl_allocation_id: 1003,
        local_ip: "10.10.0.20".to_string(),
        local_port: 19132,
        protocol: Protocol::Auto, // Auto detection should detect Bedrock -> UDP
        custom_alias: None,
        status: AllocationStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.upsert_allocation(&alloc_c).unwrap();

    let map_c = alloc_mgr.provision_mapping(&node, &alloc_c, Some("bedrock-egg"), None).await.unwrap();
    assert_eq!(map_c.protocol, Protocol::Udp);
    // Port 30001 UDP is still free (Server A only took TCP 30001)!
    assert_eq!(map_c.gateway_port, 30001);

    // 4. Teardown Server B without affecting A or C
    alloc_mgr.delete_mapping("alloc-srv-b").await.unwrap();
    assert!(db.get_mapping_by_allocation("alloc-srv-b").unwrap().is_none());

    // Verify Server A & C still active
    let check_a = db.get_mapping_by_allocation("alloc-srv-a").unwrap().unwrap();
    let check_c = db.get_mapping_by_allocation("alloc-srv-c").unwrap().unwrap();
    assert_eq!(check_a.gateway_port, 30001);
    assert_eq!(check_c.gateway_port, 30001);
}

#[tokio::test]
async fn test_duplicate_prevention_and_idempotency() {
    let db = Database::open_in_memory().unwrap();
    let port_mgr = PortManager::new(db.clone());

    let gw = Gateway {
        id: "gw-sg-test".to_string(),
        region: "singapore".to_string(),
        public_ip: "3.108.50.20".to_string(),
        control_port: 7000,
        tcp_port_range_start: 30000,
        tcp_port_range_end: 30010,
        udp_port_range_start: 30000,
        udp_port_range_end: 30010,
        reserved_ports: vec![30000],
        is_healthy: true,
        last_heartbeat: None,
        token: "token".to_string(),
        created_at: chrono::Utc::now(),
    };
    db.upsert_gateway(&gw).unwrap();

    let node = Node {
        id: "node-01".to_string(),
        name: "Node 1".to_string(),
        pterodactyl_node_id: 1,
        assigned_gateway_id: "gw-sg-test".to_string(),
        local_ip: "10.10.0.1".to_string(),
        is_healthy: true,
        last_heartbeat: None,
        agent_token: "tok".to_string(),
        created_at: chrono::Utc::now(),
    };
    db.upsert_node(&node).unwrap();

    let dns = Arc::new(MockDnsProvider::default());
    let proto_detector = Arc::new(ProtocolDetector::new(HashMap::new()));
    let alloc_mgr = AllocationManager::new(
        db.clone(),
        port_mgr.clone(),
        proto_detector,
        dns.clone(),
        Some("mc.example.com".to_string()),
    );

    let alloc1 = Allocation {
        id: "alloc-1".to_string(),
        node_id: "node-01".to_string(),
        server_id: Some("srv-1".to_string()),
        server_name: Some("Server 1".to_string()),
        pterodactyl_allocation_id: 101,
        local_ip: "10.10.0.1".to_string(),
        local_port: 25565,
        protocol: Protocol::Tcp,
        custom_alias: None,
        status: AllocationStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.upsert_allocation(&alloc1).unwrap();

    // 1. Provision once
    let map1 = alloc_mgr.provision_mapping(&node, &alloc1, None, None).await.unwrap();
    assert_eq!(map1.gateway_port, 30001);

    // 2. Provision again for same allocation -> must return existing mapping without duplication
    let map1_dup = alloc_mgr.provision_mapping(&node, &alloc1, None, None).await.unwrap();
    assert_eq!(map1_dup.id, map1.id);
    assert_eq!(map1_dup.gateway_port, 30001);

    // Verify only ONE mapping exists in database
    let all_maps = db.list_mappings().unwrap();
    assert_eq!(all_maps.len(), 1);

    // 3. Provision a second server -> must get next available port (30002) without conflict
    let alloc2 = Allocation {
        id: "alloc-2".to_string(),
        node_id: "node-01".to_string(),
        server_id: Some("srv-2".to_string()),
        server_name: Some("Server 2".to_string()),
        pterodactyl_allocation_id: 102,
        local_ip: "10.10.0.1".to_string(),
        local_port: 25566,
        protocol: Protocol::Tcp,
        custom_alias: None,
        status: AllocationStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db.upsert_allocation(&alloc2).unwrap();

    let map2 = alloc_mgr.provision_mapping(&node, &alloc2, None, None).await.unwrap();
    assert_eq!(map2.gateway_port, 30002);
    assert_ne!(map2.gateway_port, map1.gateway_port);

    let all_maps_after = db.list_mappings().unwrap();
    assert_eq!(all_maps_after.len(), 2);
}
